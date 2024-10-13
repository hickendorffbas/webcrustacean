mod color;
mod debug;
mod dom;
mod html_lexer;
mod html_parser;
#[cfg(test)] mod jsonify; //TODO: would also like to use it for debug, not sure how to configure that. feature flag on the crate maybe?
mod layout;
mod macros;
mod network;
mod platform;
mod renderer;
mod resource_loader;
mod script;
mod style;
mod ui;
mod ui_components;
#[cfg(test)] mod test_util; //TODO: is there a better (test-specific) place to define this?

use std::{
    cell::RefCell,
    cmp,
    env,
    rc::Rc,
    thread,
    time::{Duration, Instant},
};

use arboard::Clipboard;
use layout::{collect_content_nodes_in_walk_order, TextLayoutRect};
use sdl2::{
    event::Event as SdlEvent,
    keyboard::{Keycode, Mod as SdlKeyMod},
    mouse::MouseButton,
};
use threadpool::ThreadPool;

use crate::debug::debug_log_warn;
use crate::dom::Document;
use crate::platform::fonts::Font;
use crate::layout::{
    compute_layout,
    FullLayout,
    LayoutNode,
    rebuild_dirty_layout_childs,
    Rect,
};
use crate::network::url::Url;
use crate::platform::Platform;
use crate::resource_loader::{ResourceRequestJobTracker, ResourceThreadPool};
use crate::renderer::render;
use crate::script::{js_execution_context::JsExecutionContext, js_interpreter};
use crate::ui::{
    CONTENT_HEIGHT,
    CONTENT_TOP_LEFT_X,
    CONTENT_TOP_LEFT_Y,
    FocusTarget,
    HEADER_HEIGHT,
    History,
    MAIN_SCROLLBAR_HEIGHT,
    MAIN_SCROLLBAR_X_POS,
    UIState,
};
use crate::ui_components::{
    NavigationButton,
    TextField,
    Scrollbar,
};


//Config:
const TARGET_FPS: u32 = if cfg!(debug_assertions) { 20 } else { 60 };
const SCREEN_WIDTH: f32 = 1400.0;
const SCREEN_HEIGHT: f32 = 800.0;
const DEFAULT_LOCATION_TO_LOAD: &str = "about:home";
const SCROLL_SPEED: i32 = 25;
const NR_RESOURCE_LOADING_THREADS: usize = 4;


//Non-config constants:
const TARGET_MS_PER_FRAME: u128 = 1000 / TARGET_FPS as u128;



fn frame_time_check(start_instant: &Instant) {
    let millis_elapsed = start_instant.elapsed().as_millis();
    let sleep_time_millis = TARGET_MS_PER_FRAME as i64 - millis_elapsed as i64;
    if sleep_time_millis > 1 {
        //If we are more than a millisecond faster than what we need to reach the target FPS, we sleep
        thread::sleep(Duration::from_millis(sleep_time_millis as u64));
    } else {
        debug_log_warn(format!("we did not reach the target FPS, frametime: {}", millis_elapsed));
    }
}


fn handle_left_click(ui_state: &mut UIState, x: f32, y: f32, page_relative_mouse_y: f32, full_layout: &FullLayout) -> Option<Url> {
    let possible_url = ui::handle_possible_ui_click(ui_state, x, y);
    if possible_url.is_some() {
        return possible_url;
    }

    return full_layout.root_node.borrow().find_clickable(x, page_relative_mouse_y, ui_state.current_scroll_y);
}


pub struct MouseState {
    x: i32,
    y: i32,
    click_start_x: i32,
    click_start_y: i32,
    left_down: bool,
}


pub fn start_navigate(url: &Url, ui_state: &mut UIState, resource_thread_pool: &mut ResourceThreadPool) -> ResourceRequestJobTracker<String> {
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

    ui_state.currently_loading_page = true;

    //TODO: this code belongs in a history module somewhere as well...
    ui_state.forward_button.enabled = ui_state.history.list.len() > ui_state.history.position + 1;
    ui_state.back_button.enabled = ui_state.history.position > 0;

    ui_state.history.currently_navigating_from_history = false;
    let main_page_job_tracker = resource_loader::schedule_load_text(&url, resource_thread_pool); //TODO: should this be a different thread pool, or rename it?

    return main_page_job_tracker;
}


fn finish_navigate(url: &Url, ui_state: &mut UIState, page_content: &String, document: &RefCell<Document>, full_layout: &RefCell<FullLayout>,
                   platform: &mut Platform, resource_thread_pool: &mut ResourceThreadPool) {
    let lex_result = html_lexer::lex_html(&page_content);
    document.replace(html_parser::parse(lex_result, url, resource_thread_pool, &mut JsExecutionContext::new()));

    //for now we run scripts here, because we don't want to always run them fully in the main loop, and we need to have the DOM before we run
    //but I'm not sure this is really the correct place
    let mut interpreter = js_interpreter::JsInterpreter::new();
    interpreter.run_scripts_in_document(document);

    #[cfg(feature="timings")] let start_layout_instant = Instant::now();
    full_layout.replace(layout::build_full_layout(&document.borrow(), &platform.font_context, &url));

    ui_state.current_scroll_y = 0.0;
    ui_state.currently_loading_page = false;

    compute_layout(&full_layout.borrow().root_node, &document.borrow().style_context, CONTENT_TOP_LEFT_X, CONTENT_TOP_LEFT_Y, &platform.font_context, false, true);

    #[cfg(feature="timings")] println!("layout elapsed millis: {}", start_layout_instant.elapsed().as_millis());
}


fn build_selection_rect_on_text_layout_rect(text_layout_rect: &mut TextLayoutRect, selection_rect: &Rect, start_for_selection_rect_on_layout_rect: f32,
                                            start_idx_for_selection: usize) {
    let mut matching_offset = text_layout_rect.location.width;

    let mut end_idx_for_selection = 0;
    for (idx, offset) in text_layout_rect.char_position_mapping.iter().enumerate() {
        if text_layout_rect.location.x + offset > selection_rect.x + selection_rect.width {
            matching_offset = *offset;
            end_idx_for_selection = idx;
            break;
        }
    }

    let selection_rect_for_layout_rect = Rect { x: start_for_selection_rect_on_layout_rect,
                y: text_layout_rect.location.y,
                width: (text_layout_rect.location.x + matching_offset) - start_for_selection_rect_on_layout_rect,
                height: text_layout_rect.location.height };
    text_layout_rect.selection_rect = Some(selection_rect_for_layout_rect);
    text_layout_rect.selection_char_range = Some( (start_idx_for_selection, end_idx_for_selection) );
}


fn compute_selection_regions(layout_node: &Rc<RefCell<LayoutNode>>, selection_rect: &Rect, current_scroll_y: f32, nodes_in_selection_order: &Vec<Rc<RefCell<LayoutNode>>>) {
    if !layout_node.borrow().visible_on_y_location(current_scroll_y) {
        return;
    }

    let mut selection_start_found = false;
    let selection_end_x = selection_rect.x + selection_rect.width;
    let selection_end_y = selection_rect.y + selection_rect.height;

    match &mut layout_node.borrow_mut().content {
        layout::LayoutNodeContent::TextLayoutNode(ref mut text_layout_node) => {
            let mut start_x_for_selection_rect_on_layout_rect = 0.0;
            let mut start_idx_for_selection = 0;
            for mut layout_rect in text_layout_node.rects.iter_mut() {
                if layout_rect.location.is_inside(selection_rect.x, selection_rect.y) {
                    selection_start_found = true;

                    let mut previous_offset = 0.0;
                    for (idx, offset) in layout_rect.char_position_mapping.iter().enumerate() {
                        if layout_rect.location.x + offset > selection_rect.x {
                            start_x_for_selection_rect_on_layout_rect = layout_rect.location.x + previous_offset;
                            start_idx_for_selection = idx;
                            break;
                        }

                        previous_offset = *offset;
                    }

                    //Handle the special case where both the top left and the bottom right of the selection rect are in the same layout rect:
                    if layout_rect.location.is_inside(selection_end_x, selection_end_y) {
                        build_selection_rect_on_text_layout_rect(&mut layout_rect, selection_rect, start_x_for_selection_rect_on_layout_rect, start_idx_for_selection);
                        return;
                    } else {
                        let selection_rect_for_layout_rect = Rect { x: start_x_for_selection_rect_on_layout_rect,
                                                                    y: layout_rect.location.y,
                                                                    width: layout_rect.location.width - start_x_for_selection_rect_on_layout_rect,
                                                                    height: layout_rect.location.height };
                        layout_rect.selection_rect = Some(selection_rect_for_layout_rect);
                        layout_rect.selection_char_range = Some( (start_idx_for_selection, layout_rect.text.len()) );

                    }
                } else if selection_start_found {
                    // Now we check for other rects on the same layout node that might contain the bottom right point:
                    if layout_rect.location.is_inside(selection_end_x, selection_end_y) {
                        let start_selection_pos = layout_rect.location.x;
                        build_selection_rect_on_text_layout_rect(&mut layout_rect, selection_rect, start_selection_pos, 0);
                        return;
                    } else {
                        //This rect is in between the start and end node, so we fully set it as selected:
                        let selection_rect_for_layout_rect = Rect { x: layout_rect.location.x, y: layout_rect.location.y,
                                                                    width: layout_rect.location.width, height: layout_rect.location.height };
                        layout_rect.selection_rect = Some(selection_rect_for_layout_rect);
                        layout_rect.selection_char_range = Some( (0, layout_rect.text.len()) );
                    }
                }
            }
        },
        layout::LayoutNodeContent::ImageLayoutNode(_) => todo!(),  //TODO: implement (selection on an image can just be a bool, since you cannot select part of an image)
        layout::LayoutNodeContent::ButtonLayoutNode(_) => todo!(),  //TODO: implement
        layout::LayoutNodeContent::TextInputLayoutNode(_) => todo!(),  //TODO: implement
        layout::LayoutNodeContent::BoxLayoutNode(_) => {
            //Note: this is a no-op for now, since there is nothing to select in a box node itself (just in its children)
        },
        layout::LayoutNodeContent::NoContent => todo!(),  //TODO: implement
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

                match &mut next_selection_node.borrow_mut().content {
                    layout::LayoutNodeContent::TextLayoutNode(ref mut text_layout_node) => {

                        for mut layout_rect in text_layout_node.rects.iter_mut() {
                            if layout_rect.location.is_inside(selection_end_x, selection_end_y) {
                                let start_selection_pos = layout_rect.location.x;
                                build_selection_rect_on_text_layout_rect(&mut layout_rect, selection_rect, start_selection_pos, 0);
                                return;
                            } else {
                                //This node is in between the start and end node, so we fully set it as selected:
                                let selection_rect_for_layout_rect = Rect { x: layout_rect.location.x, y: layout_rect.location.y,
                                                                            width: layout_rect.location.width, height: layout_rect.location.height };
                                layout_rect.selection_rect = Some(selection_rect_for_layout_rect);
                                layout_rect.selection_char_range = Some( (0, layout_rect.text.len()) );
                            }
                        }
                    },
                    layout::LayoutNodeContent::ImageLayoutNode(_) => todo!(),  //TODO: implement
                    layout::LayoutNodeContent::ButtonLayoutNode(_) => todo!(),  //TODO: implement
                    layout::LayoutNodeContent::TextInputLayoutNode(_) => todo!(),  //TODO: implement
                    layout::LayoutNodeContent::BoxLayoutNode(_) => todo!(),  //TODO: implement
                    layout::LayoutNodeContent::NoContent => todo!(),  //TODO: implement
                }
            }
        }
    }

    if layout_node.borrow().children.is_some() {
        for child in layout_node.borrow().children.as_ref().unwrap() {
            compute_selection_regions(&child, selection_rect, current_scroll_y, nodes_in_selection_order);
        }
    }
}


fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let mut platform = platform::init_platform(sdl_context).unwrap();

    let mut resource_thread_pool = ResourceThreadPool { pool: ThreadPool::new(NR_RESOURCE_LOADING_THREADS) };

    let args: Vec<String> = env::args().collect();

    let mut url = if args.len() < 2 {
        Url::from(&DEFAULT_LOCATION_TO_LOAD.to_owned())
    } else {
        Url::from(&args[1])
    };


    let mut mouse_state = MouseState { x: 0, y: 0, click_start_x: 0, click_start_y: 0, left_down: false };
    let addressbar_text = url.to_string();

    let mut addressbar_text_field = TextField {
        x: 100.0,
        y: 10.0,
        width: SCREEN_WIDTH - 200.0,
        height: 35.0,
        has_focus: false,
        cursor_text_position: 0,
        text: String::new(),
        select_on_first_click: true,
        selection_start_x: 0.0,
        selection_end_x: 0.0,
        font: Font::default(),
        char_position_mapping: Vec::new(),
        selection_start_idx: 0,
        selection_end_idx: 0,
    };
    addressbar_text_field.set_text(&mut platform, addressbar_text);

    //TODO: this setting up of components should happen in the ui module eventually
    let main_scrollbar = Scrollbar {
        x: MAIN_SCROLLBAR_X_POS,
        y: HEADER_HEIGHT,
        width: SCREEN_WIDTH,
        height: MAIN_SCROLLBAR_HEIGHT,
        content_size: 0.0,
        content_visible_height: CONTENT_HEIGHT,
        block_height: MAIN_SCROLLBAR_HEIGHT,
        block_y: HEADER_HEIGHT,
        enabled: false,
    };

    let mut ui_state = UIState {
        addressbar: addressbar_text_field,
        current_scroll_y: 0.0,
        back_button: NavigationButton { x: 15.0, y: 15.0, forward: false, enabled: false },
        forward_button: NavigationButton { x: 55.0, y: 15.0, forward: true, enabled: false },
        history: History { list: Vec::new(), position: 0, currently_navigating_from_history: false },
        currently_loading_page: false,
        animation_tick: 0,
        focus_target: FocusTarget::None,
        main_scrollbar: main_scrollbar,
    };

    let document = RefCell::from(Document::new_empty());
    let full_layout_tree = RefCell::from(FullLayout::new_empty());

    let mut main_page_job_tracker = start_navigate(&url, &mut ui_state, &mut resource_thread_pool);
    let mut currently_loading_new_page = true;

    let mut event_pump = platform.sdl_context.event_pump()?;
    'main_loop: loop {
        let start_loop_instant = Instant::now();

        if currently_loading_new_page {
            let try_recv_result = main_page_job_tracker.receiver.try_recv();
            if try_recv_result.is_ok() {
                finish_navigate(&url, &mut ui_state, &try_recv_result.ok().unwrap(), &document, &full_layout_tree, &mut platform, &mut resource_thread_pool);
                currently_loading_new_page = false;
            }
        }

        ui_state.current_scroll_y = ui_state.main_scrollbar.update_content_size(full_layout_tree.borrow().page_height(), ui_state.current_scroll_y);

        #[cfg(feature="timings")] let start_event_pump_instant = Instant::now();
        for event in event_pump.poll_iter() {
            match event {
                SdlEvent::Quit {..} | SdlEvent::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'main_loop;
                },
                SdlEvent::MouseMotion { x: mouse_x, y: mouse_y, yrel, .. } => {
                    mouse_state.x = mouse_x;
                    mouse_state.y = mouse_y;

                    if ui_state.focus_target == FocusTarget::ScrollBlock {
                        ui_state.current_scroll_y = ui_state.main_scrollbar.scroll(yrel as f32, ui_state.current_scroll_y);

                    } else if mouse_state.left_down {
                        let top_left_x = cmp::min(mouse_state.click_start_x, mouse_x) as f32;
                        let top_left_y = cmp::min(mouse_state.click_start_y, mouse_y) as f32 + ui_state.current_scroll_y;
                        let bottom_right_x = cmp::max(mouse_state.click_start_x, mouse_x) as f32;
                        let bottom_right_y = cmp::max(mouse_state.click_start_y, mouse_y) as f32 + ui_state.current_scroll_y;
                        let selection_rect = Rect { x: top_left_x, y: top_left_y, width: bottom_right_x - top_left_x, height: bottom_right_y - top_left_y };

                        if ui_state.focus_target == FocusTarget::MainContent {
                            RefCell::borrow_mut(&full_layout_tree.borrow_mut().root_node).reset_selection();
                            let full_layout_tree = full_layout_tree.borrow();
                            compute_selection_regions(&full_layout_tree.root_node, &selection_rect, ui_state.current_scroll_y, &full_layout_tree.nodes_in_selection_order);
                        } else if ui_state.focus_target == FocusTarget::AddressBar {
                            ui_state.addressbar.update_selection(&selection_rect)
                        } else {
                            //TODO: its not clear to me yet how we clear the selection on the main content
                            ui_state.addressbar.clear_selection();
                        }
                    }
                },
                SdlEvent::MouseButtonDown { mouse_btn: MouseButton::Left, x: mouse_x, y: mouse_y, .. } => {
                    mouse_state.x = mouse_x;
                    mouse_state.y = mouse_y;
                    mouse_state.click_start_x = mouse_x;
                    mouse_state.click_start_y = mouse_y;
                    mouse_state.left_down = true;

                    RefCell::borrow_mut(&full_layout_tree.borrow_mut().root_node).reset_selection();

                    ui::handle_possible_ui_mouse_down(&mut platform, &mut ui_state, mouse_x as f32, mouse_y as f32);
                },
                SdlEvent::MouseButtonUp { mouse_btn: MouseButton::Left, x: mouse_x, y: mouse_y, .. } => {
                    mouse_state.x = mouse_x;
                    mouse_state.y = mouse_y;
                    mouse_state.left_down = false;

                    if ui_state.focus_target == FocusTarget::ScrollBlock {
                        ui_state.focus_target = FocusTarget::None;
                    }

                    let abs_movement = (mouse_state.x - mouse_state.click_start_x).abs() + (mouse_state.y - mouse_state.click_start_y).abs();
                    let was_dragging = abs_movement > 1;

                    if !was_dragging {
                        let page_relative_mouse_y = mouse_y as f32 + ui_state.current_scroll_y;
                        let optional_url = handle_left_click(&mut ui_state, mouse_x as f32, mouse_y as f32, page_relative_mouse_y, &full_layout_tree.borrow());

                        if optional_url.is_some() {
                            let url = optional_url.unwrap();
                            //TODO: this should be done via a nicer "navigate" method or something (also below when pressing enter in the addressbar)
                            ui_state.addressbar.set_text(&mut platform, url.to_string());
                            main_page_job_tracker = start_navigate(&url, &mut ui_state, &mut resource_thread_pool); //TODO: we should do this above in the next loop, just schedule the url for reload
                            currently_loading_new_page = true;
                        }
                    }
                },
                SdlEvent::MouseWheel { y, direction, .. } => {
                    match direction {
                        sdl2::mouse::MouseWheelDirection::Normal => {
                            //TODO: someday it might be nice to implement smooth scrolling (animate the movement over frames)
                            let new_page_scroll_y = ui_state.current_scroll_y - (y * SCROLL_SPEED) as f32;
                            ui_state.current_scroll_y = ui_state.main_scrollbar.update_scroll(new_page_scroll_y);
                        },
                        sdl2::mouse::MouseWheelDirection::Flipped => {},
                        sdl2::mouse::MouseWheelDirection::Unknown(_) => debug_log_warn("Unknown mousewheel direction!"),
                    }
                },
                SdlEvent::KeyDown { keycode, keymod, .. } => {
                    if keycode.is_some() {
                        let key_code = platform.convert_key_code(&keycode.unwrap());
                        ui::handle_keyboard_input(&mut platform, None, key_code, &mut ui_state);

                        if ui_state.addressbar.has_focus && keycode.unwrap().name() == "Return" {
                            //TODO: This is here for now because we need to load another page, not sure how to correctly trigger that from inside the component
                            url = Url::from(&ui_state.addressbar.text);
                            main_page_job_tracker = start_navigate(&url, &mut ui_state, &mut resource_thread_pool);
                            currently_loading_new_page = true;
                        }

                        if keymod.contains(SdlKeyMod::LCTRLMOD) {
                            if keycode.unwrap().name() == "C" {
                                let mut text_for_clipboard = String::new();
                                full_layout_tree.borrow().root_node.borrow().get_selected_text(&mut text_for_clipboard);
                                if text_for_clipboard.is_empty() && ui_state.addressbar.has_selection_active() {
                                    text_for_clipboard = ui_state.addressbar.get_selected_text();
                                }

                                if !text_for_clipboard.is_empty() {
                                    let mut clipboard = Clipboard::new().unwrap();
                                    clipboard.set_text(text_for_clipboard).expect("Unhandled clipboard error");
                                }
                            }

                            if keycode.unwrap().name() == "V" {
                                match ui_state.focus_target {
                                    FocusTarget::AddressBar => {
                                        let clipboard_text = Clipboard::new().unwrap().get_text().expect("Unhandled clipboard error");
                                        ui_state.addressbar.insert_text(&mut platform, &clipboard_text);
                                    },
                                    _ => {},
                                }
                            }
                        }


                    }
                },
                SdlEvent::TextInput { text, .. } => {
                    ui::handle_keyboard_input(&mut platform, Some(&text), None, &mut ui_state);
                },
                _ => {},
            }
        }
        #[cfg(feature="timings")] println!("event pump elapsed millis: {}", start_event_pump_instant.elapsed().as_millis());

        let document_has_dirty_nodes = document.borrow_mut().update_all_dom_nodes(&mut resource_thread_pool);

        if document_has_dirty_nodes {
            rebuild_dirty_layout_childs(&full_layout_tree.borrow().root_node, &document.borrow(), &platform.font_context, &url);

            let mut nodes_in_selection_order = Vec::new();
            collect_content_nodes_in_walk_order(&full_layout_tree.borrow().root_node, &mut nodes_in_selection_order);
            full_layout_tree.borrow_mut().nodes_in_selection_order = nodes_in_selection_order;

            compute_layout(&full_layout_tree.borrow().root_node, &document.borrow().style_context, CONTENT_TOP_LEFT_X, CONTENT_TOP_LEFT_Y,
                           &platform.font_context, false, false);
        }

        #[cfg(feature="timings")] let start_render_instant = Instant::now();
        render(&mut platform, &full_layout_tree.borrow(), &mut ui_state);
        #[cfg(feature="timings")] println!("render elapsed millis: {}", start_render_instant.elapsed().as_millis());

        frame_time_check(&start_loop_instant);
    }

    Ok(())
}
