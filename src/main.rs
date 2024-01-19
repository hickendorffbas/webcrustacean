mod color;
mod debug;
mod dom;
mod experiment_threading;
mod fonts;
mod html_lexer;
mod html_parser;
mod layout;
mod macros;
mod network;
mod platform;
mod renderer;
mod resource_loader;
mod style;
mod ui;
mod ui_components;
#[cfg(test)] mod test_util; //TODO: is there a better (test-specific) place to define this?

use std::cell::RefCell;
use std::cmp;
use std::env;
use std::rc::Rc;
use std::thread;
use std::time::{Duration, Instant};

use arboard::Clipboard;
use sdl2::keyboard::Mod as SdlKeyMod;
use sdl2::{
    event::Event as SdlEvent,
    keyboard::Keycode,
    mouse::MouseButton,
};

use crate::debug::debug_log_warn;
use crate::dom::Document;
use crate::experiment_threading::run_experiment;
use crate::fonts::Font;
use crate::layout::{
    FullLayout,
    LayoutNode,
    LayoutRect,
    Rect,
};
use crate::network::url::Url;
use crate::platform::Platform;
use crate::renderer::render;
use crate::ui::{CONTENT_HEIGHT, UIState};
use crate::ui_components::{TextField, NavigationButton};


use ui::History;

//Config:
const TARGET_FPS: u32 = if cfg!(debug_assertions) { 20 } else { 60 };
const SCREEN_WIDTH: f32 = 1400.0;
const SCREEN_HEIGHT: f32 = 800.0;
const DEFAULT_LOCATION_TO_LOAD: &str = "file:///home/bas/webcrustacean/testinput/doc.html";
const SCROLL_SPEED: i32 = 25;

//TODO: we probably should include the font files in the repo / build, since these paths might be different on different systems
#[cfg(target_os = "macos")] const FONT_PATH: &str = "/Library/Fonts/Managed/OpenSans-Light_744839258.ttf";
#[cfg(target_os = "linux")] const FONT_PATH: &str = "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf";
#[cfg(target_os = "windows")] const FONT_PATH: &str = "C:/Windows/Fonts/arial.ttf";


//Non-config constants:
const TARGET_MS_PER_FRAME: u128 = 1000 / TARGET_FPS as u128;



fn frame_time_check(start_instant: &Instant, currently_loading_new_page: bool) {
    let millis_elapsed = start_instant.elapsed().as_millis();
    let sleep_time_millis = TARGET_MS_PER_FRAME as i64 - millis_elapsed as i64;
    if sleep_time_millis > 1 {
        //If we are more than a millisecond faster than what we need to reach the target FPS, we sleep
        thread::sleep(Duration::from_millis(sleep_time_millis as u64));
    } else {
        if !currently_loading_new_page {
            debug_log_warn(format!("we did not reach the target FPS, frametime: {}", millis_elapsed));
        }
    }
}


fn handle_left_click(platform: &mut Platform, ui_state: &mut UIState, x: f32, y: f32, full_layout: &FullLayout) -> Option<Url> {
    let possible_url = ui::handle_possible_ui_click(platform, ui_state, x, y);
    if possible_url.is_some() {
        return possible_url;
    }

    return full_layout.root_node.borrow().find_clickable(x, y, ui_state.current_scroll_y);
}


pub struct MouseState {
    x: i32,
    y: i32,
    click_start_x: i32,
    click_start_y: i32,
    left_down: bool,
    //TODO: eventually we need a more generic way to refer to controls we are currently dragging...
    is_dragging_scrollblock: bool,
}


pub fn navigate(url: &Url, ui_state: &mut UIState, platform: &mut Platform, document: &RefCell<Document>, full_layout: &RefCell<FullLayout>) {
    //TODO: we should wrap the history logic in a function or module somewhere...
    if !ui_state.history.currently_navigating_from_history {
        if ui_state.history.list.len() > (ui_state.history.position + 1) {
            let last_idx_to_keep = ui_state.history.position;
            for idx in ((last_idx_to_keep + 1)..ui_state.history.list.len()).rev() {
                ui_state.history.list.remove(idx);
            }
        }
        ui_state.history.list.push(url.clone());
        ui_state.history.position = ui_state.history.list.len() - 1;
        if ui_state.history.position > 0 {
            ui_state.back_button.enabled = true;
        }
    }

    //TODO: this code belongs in a history module somewhere as well...
    ui_state.forward_button.enabled = ui_state.history.list.len() > ui_state.history.position + 1;
    ui_state.back_button.enabled = ui_state.history.position > 0;

    ui_state.current_scroll_y = 0.0;

    ui_state.history.currently_navigating_from_history = false;
    let page_content = resource_loader::load_text(&url);
    let lex_result = html_lexer::lex_html(&page_content);
    document.replace(html_parser::parse(lex_result, url));

    #[cfg(feature="timings")] let start_layout_instant = Instant::now();
    full_layout.replace(layout::build_full_layout(&document.borrow(), platform, &url));
    debug_assert!(full_layout.borrow().root_node.borrow().rects.len() == 1);
    #[cfg(feature="timings")] println!("layout elapsed millis: {}", start_layout_instant.elapsed().as_millis());
}


fn build_selection_rect_on_layout_rect(layout_rect: &mut LayoutRect, selection_rect: &Rect, start_for_selection_rect_on_layout_rect: f32, start_idx_for_selection: usize) {
    let mut matching_offset = layout_rect.location.width;

    if layout_rect.text.is_some() {

        let mut end_idx_for_selection = 0;
        for (idx, offset) in layout_rect.char_position_mapping.as_ref().unwrap().iter().enumerate() {
            if layout_rect.location.x + offset > selection_rect.x + selection_rect.width {
                matching_offset = *offset;
                end_idx_for_selection = idx;
                break;
            }
        }

        let selection_rect_for_layout_rect = Rect { x: start_for_selection_rect_on_layout_rect,
                    y: layout_rect.location.y,
                    width: (layout_rect.location.x + matching_offset) - start_for_selection_rect_on_layout_rect,
                    height: layout_rect.location.height };
        layout_rect.selection_rect = Some(selection_rect_for_layout_rect);
        layout_rect.selection_char_range = Some( (start_idx_for_selection, end_idx_for_selection) );
        println!("UU: {start_idx_for_selection}, {end_idx_for_selection}")

    } else if layout_rect.image.is_some() {
        //Currently no selection is implemented for images
    } else {
        panic!("We should not get in this method with a rect without content");
    }
}


fn compute_selection_regions(layout_node: &Rc<RefCell<LayoutNode>>, selection_rect: &Rect, current_scroll_y: f32, nodes_in_selection_order: &Vec<Rc<RefCell<LayoutNode>>>) {
    let any_visible = layout_node.borrow().rects.iter().any(|rect| -> bool { rect.location.is_visible_on_y_location(current_scroll_y) });
    if !any_visible {
        return;
    }

    let any_rects_with_content = layout_node.borrow().rects.iter().any(|rect| -> bool { rect.text.is_some() || rect.image.is_some() });
    if any_rects_with_content {

        let selection_end_x = selection_rect.x + selection_rect.width;
        let selection_end_y = selection_rect.y + selection_rect.height;

        let mut selection_start_found = false;
        let mut start_for_selection_rect_on_layout_rect = 0.0;
        let mut start_idx_for_selection = 0;
        for mut layout_rect in RefCell::borrow_mut(layout_node).rects.iter_mut() {
            if layout_rect.location.is_inside(selection_rect.x, selection_rect.y) {
                selection_start_found = true;

                if layout_rect.text.is_some() {

                    let mut previous_offset = 0.0;
                    for (idx, offset) in layout_rect.char_position_mapping.as_ref().unwrap().iter().enumerate() {
                        if layout_rect.location.x + offset > selection_rect.x {
                            start_for_selection_rect_on_layout_rect = layout_rect.location.x + previous_offset;
                            start_idx_for_selection = idx - 1;
                            break;
                        }

                        previous_offset = *offset;
                    }

                } else if layout_rect.image.is_some() {
                    start_for_selection_rect_on_layout_rect = layout_rect.location.x;
                    start_idx_for_selection = 0;
                } else {
                    panic!("We should not get here with a rect without content");
                }

                //Handle the special case where both the top left and the bottom right of the selection rect are in the same layout rect:
                if layout_rect.location.is_inside(selection_end_x, selection_end_y) {
                    build_selection_rect_on_layout_rect(&mut layout_rect, selection_rect, start_for_selection_rect_on_layout_rect, start_idx_for_selection);
                    return;
                } else {
                    let selection_rect_for_layout_rect = Rect { x: start_for_selection_rect_on_layout_rect,
                                                                y: layout_rect.location.y,
                                                                width: layout_rect.location.width - start_for_selection_rect_on_layout_rect,
                                                                height: layout_rect.location.height };
                    layout_rect.selection_rect = Some(selection_rect_for_layout_rect);
                    if layout_rect.text.is_some() {
                        layout_rect.selection_char_range = Some( (start_idx_for_selection, layout_rect.text.as_ref().unwrap().len()) );
                    }
                }
            } else if selection_start_found {
                // Now we check for other rects on the same layout node that might contain the bottom right point:
                if layout_rect.location.is_inside(selection_end_x, selection_end_y) {
                    let start_selection_pos = layout_rect.location.x;
                    build_selection_rect_on_layout_rect(&mut layout_rect, selection_rect, start_selection_pos, 0);
                    return;
                } else {
                    //This rect is in between the start and end node, so we fully set it as selected:
                    let selection_rect_for_layout_rect = Rect { x: layout_rect.location.x, y: layout_rect.location.y,
                                                                width: layout_rect.location.width, height: layout_rect.location.height };
                    layout_rect.selection_rect = Some(selection_rect_for_layout_rect);
                    layout_rect.selection_char_range = Some( (0, layout_rect.text.as_ref().unwrap().len()) );
                }
            }
        }
        if selection_start_found {
            //Now we are going to walk the layout nodes to find the node where the selection ends, and all nodes in between

            let mut starting_node_found = false;
            for next_selection_node in nodes_in_selection_order {

                if !starting_node_found {
                    if next_selection_node.borrow().internal_id == layout_node.borrow().internal_id {
                        starting_node_found = true;
                    }
                    continue;
                } else {
                    for mut layout_rect in RefCell::borrow_mut(next_selection_node).rects.iter_mut() {
                        if layout_rect.location.is_inside(selection_end_x, selection_end_y) {
                            let start_selection_pos = layout_rect.location.x;
                            build_selection_rect_on_layout_rect(&mut layout_rect, selection_rect, start_selection_pos, 0);
                            return;
                        } else {
                            //This node is in between the start and end node, so we fully set it as selected:
                            let selection_rect_for_layout_rect = Rect { x: layout_rect.location.x, y: layout_rect.location.y,
                                                                        width: layout_rect.location.width, height: layout_rect.location.height };
                            layout_rect.selection_rect = Some(selection_rect_for_layout_rect);

                            if layout_rect.text.is_some() {
                                layout_rect.selection_char_range = Some( (0, layout_rect.text.as_ref().unwrap().len()) );
                            }
                        }
                    }
                }
            }
        }

    } else {
        if layout_node.borrow().children.is_some() {
            for child in layout_node.borrow().children.as_ref().unwrap() {
                compute_selection_regions(&child, selection_rect, current_scroll_y, nodes_in_selection_order);
            }
        }
    }
}


fn main() -> Result<(), String> {

    run_experiment();
    return Ok(());




    let sdl_context = sdl2::init()?;
    let ttf_context = sdl2::ttf::init()
                                .expect("could not initialize the font system");
    let mut platform = platform::init_platform(sdl_context, &ttf_context).unwrap();

    let args: Vec<String> = env::args().collect();

    let mut url = if args.len() < 2 {
        Url::from(&DEFAULT_LOCATION_TO_LOAD.to_owned())
    } else {
        Url::from(&args[1])
    };

    let mut mouse_state = MouseState { x: 0, y: 0, click_start_x: 0, click_start_y: 0, left_down: false, is_dragging_scrollblock: false };
    let addressbar_text = url.to_string();

    let mut addressbar_text_field = TextField {
        x: 100.0,
        y: 10.0,
        width: SCREEN_WIDTH - 200.0,
        height: 35.0,
        has_focus: false,
        cursor_visible: false,
        cursor_text_position: 0,
        text: String::new(),
        font: Font::new(false, false, 18),
        char_position_mapping: Vec::new(),
    };
    addressbar_text_field.set_text(&mut platform, addressbar_text);

    let mut ui_state = UIState {
        addressbar: addressbar_text_field,
        current_scroll_y: 0.0,
        back_button: NavigationButton { x: 15.0, y: 15.0, forward: false, enabled: false },
        forward_button: NavigationButton { x: 55.0, y: 15.0, forward: true, enabled: false },
        history: History { list: Vec::new(), position: 0, currently_navigating_from_history: false },
    };

    let mut should_reload_from_url = false;

    let document = RefCell::from(Document::new_empty());
    let full_layout_tree = RefCell::from(FullLayout::new_empty());

    navigate(&url, &mut ui_state, &mut platform, &document, &full_layout_tree);
    let mut currently_loading_new_page = true;

    let mut event_pump = platform.sdl_context.event_pump()?;
    'main_loop: loop {
        let start_loop_instant = Instant::now();

        if should_reload_from_url {
            #[cfg(feature="timings")] let start_page_load_instant = Instant::now();
            currently_loading_new_page = true;
            navigate(&url, &mut ui_state, &mut platform, &document, &full_layout_tree);
            should_reload_from_url = false;
            #[cfg(feature="timings")] println!("page load elapsed millis: {}", start_page_load_instant.elapsed().as_millis());
        }

        #[cfg(feature="timings")] let start_event_pump_instant = Instant::now();
        for event in event_pump.poll_iter() {
            match event {
                SdlEvent::Quit {..} | SdlEvent::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'main_loop;
                },
                SdlEvent::MouseMotion { x: mouse_x, y: mouse_y, yrel, .. } => {
                    mouse_state.x = mouse_x;
                    mouse_state.y = mouse_y;

                    if mouse_state.is_dragging_scrollblock {
                        let page_scroll = ui::convert_block_drag_to_page_scroll(&mut ui_state, yrel as f32, full_layout_tree.borrow().page_height());
                        ui_state.current_scroll_y = clamp_scroll_position(ui_state.current_scroll_y + page_scroll, full_layout_tree.borrow().page_height());
                    }

                    else if mouse_state.left_down {
                        let top_left_x = cmp::min(mouse_state.click_start_x, mouse_x) as f32;
                        let top_left_y = cmp::min(mouse_state.click_start_y, mouse_y) as f32 + ui_state.current_scroll_y;
                        let bottom_right_x = cmp::max(mouse_state.click_start_x, mouse_x) as f32;
                        let bottom_right_y = cmp::max(mouse_state.click_start_y, mouse_y) as f32 + ui_state.current_scroll_y;
                        let selection_rect = Rect { x: top_left_x, y: top_left_y, width: bottom_right_x - top_left_x, height: bottom_right_y - top_left_y };

                        RefCell::borrow_mut(&full_layout_tree.borrow_mut().root_node).reset_selection();
                        let full_layout_tree = full_layout_tree.borrow();
                        compute_selection_regions(&full_layout_tree.root_node, &selection_rect, ui_state.current_scroll_y, &full_layout_tree.nodes_in_selection_order);
                    }

                },
                SdlEvent::MouseButtonDown { mouse_btn: MouseButton::Left, x: mouse_x, y: mouse_y, .. } => {
                    mouse_state.x = mouse_x;
                    mouse_state.y = mouse_y;
                    mouse_state.click_start_x = mouse_x;
                    mouse_state.click_start_y = mouse_y;
                    mouse_state.left_down = true;

                    RefCell::borrow_mut(&full_layout_tree.borrow_mut().root_node).reset_selection();

                    //TODO: its probably nicer to call a generic method in UI, to check any drags and update the mouse state
                    if ui::mouse_on_scrollblock(&mouse_state, ui_state.current_scroll_y, full_layout_tree.borrow().page_height()) {
                        mouse_state.is_dragging_scrollblock = true;
                    } else {
                        mouse_state.is_dragging_scrollblock = false;
                    }
                },
                SdlEvent::MouseButtonUp { mouse_btn: MouseButton::Left, x: mouse_x, y: mouse_y, .. } => {
                    mouse_state.x = mouse_x;
                    mouse_state.y = mouse_y;
                    mouse_state.left_down = false;
                    mouse_state.is_dragging_scrollblock = false;

                    let abs_movement = (mouse_state.x - mouse_state.click_start_x).abs() + (mouse_state.y - mouse_state.click_start_y).abs();
                    let was_dragging = abs_movement > 1;

                    if !was_dragging {
                        let new_mouse_y = mouse_y as f32 + ui_state.current_scroll_y;
                        let optional_url = handle_left_click(&mut platform, &mut ui_state, mouse_x as f32, new_mouse_y, &full_layout_tree.borrow());

                        if optional_url.is_some() {
                            let url = optional_url.unwrap();
                            //TODO: this should be done via a nicer "navigate" method or something (also below when pressing enter in the addressbar
                            ui_state.addressbar.set_text(&mut platform, url.to_string());
                            navigate(&url, &mut ui_state, &mut platform, &document, &full_layout_tree);  // we should do this above in the next loop, just schedule the url for reload
                            currently_loading_new_page = true;
                        }
                    }
                },
                SdlEvent::MouseWheel { y, direction, .. } => {
                    match direction {
                        sdl2::mouse::MouseWheelDirection::Normal => {
                            //TODO: someday it might be nice to implement smooth scrolling (animate the movement over frames)
                            ui_state.current_scroll_y = clamp_scroll_position(ui_state.current_scroll_y - (y * SCROLL_SPEED) as f32, full_layout_tree.borrow().page_height());
                        },
                        sdl2::mouse::MouseWheelDirection::Flipped => {},
                        sdl2::mouse::MouseWheelDirection::Unknown(_) => debug_log_warn("Unknown mousewheel direction!"),
                    }
                },
                SdlEvent::KeyUp { keycode, keymod, .. } => {
                    if keycode.is_some() {
                        let key_code = platform.convert_key_code(&keycode.unwrap());
                        ui::handle_keyboard_input(&mut platform, None, key_code, &mut ui_state);

                        if ui_state.addressbar.has_focus && keycode.unwrap().name() == "Return" {
                            //TODO: This is here for now because we need to load another page, not sure how to correctly trigger that from inside the component
                            url = Url::from(&ui_state.addressbar.text);
                            navigate(&url, &mut ui_state, &mut platform, &document, &full_layout_tree); // we should do this above in the next loop, just schedule the url for reload
                            currently_loading_new_page = true;
                        }

                        if keymod.contains(SdlKeyMod::LCTRLMOD) {
                            if keycode.unwrap().name() == "C" {
                                let mut text_for_clipboard = String::new();
                                full_layout_tree.borrow().root_node.borrow().get_selected_text(&mut text_for_clipboard);
                                if !text_for_clipboard.is_empty() {
                                    let mut clipboard = Clipboard::new().unwrap();
                                    clipboard.set_text(text_for_clipboard).expect("Unhandled clipboard error");
                                }
                            }

                            if keycode.unwrap().name() == "V" {
                                if ui::current_focus_can_receive_text(&ui_state) {
                                    let clipboard_text = Clipboard::new().unwrap().get_text().expect("Unhandled clipboard error");
                                    ui::insert_text(&mut platform, &mut ui_state, &clipboard_text);
                                }
                            }
                        }


                    }
                },
                SdlEvent::TextInput { text, .. } => {
                    ui::handle_keyboard_input(&mut platform, Some(&text), None, &mut ui_state);
                }
                _ => {}
            }
        }
        #[cfg(feature="timings")] println!("event pump elapsed millis: {}", start_event_pump_instant.elapsed().as_millis());

        #[cfg(feature="timings")] let start_render_instant = Instant::now();
        render(&mut platform, &full_layout_tree.borrow(), &mut ui_state);
        #[cfg(feature="timings")] println!("render elapsed millis: {}", start_render_instant.elapsed().as_millis());

        frame_time_check(&start_loop_instant, currently_loading_new_page);
        currently_loading_new_page = false;
    }

    Ok(())
}


fn clamp_scroll_position(current_scroll_y: f32, current_page_height: f32) -> f32 {
    if current_scroll_y < 0.0 {
        return 0.0;
    }
    let mut max_scroll_y = (current_page_height + 1.0) - CONTENT_HEIGHT;
    if max_scroll_y < 0.0 {
        max_scroll_y = 0.0;
    }
    if current_scroll_y > max_scroll_y {
        return max_scroll_y;
    }
    return current_scroll_y;
}
