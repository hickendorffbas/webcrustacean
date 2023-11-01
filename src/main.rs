mod color;
mod debug;
mod dom;
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
use std::{env, thread};
use std::time::{Duration, Instant};

use crate::debug::debug_log_warn;
use crate::dom::Document;
use crate::fonts::Font;
use crate::layout::ClickMapEntry;
use crate::network::url::Url;
use crate::platform::Platform;
use crate::renderer::render;
use crate::ui::{CONTENT_HEIGHT, UIState};
use crate::ui_components::{TextField, NavigationButton};

use sdl2::{
    event::Event as SdlEvent,
    keyboard::Keycode,
    mouse::MouseButton,
};


//Config:
const TARGET_FPS: u32 = if cfg!(debug_assertions) { 30 } else { 60 };
const SCREEN_WIDTH: f32 = 1000.0;
const SCREEN_HEIGHT: f32 = 700.0;
const DEFAULT_LOCATION_TO_LOAD: &str = "file:///home/bas/bbrowser/testinput/doc.html";
const SCROLL_SPEED: i32 = 25;


//TODO: detect OS automatically (compile time, using cfg) and set constants automatically
//Config for macOS:
//const FONT_PATH: &str = "/Library/Fonts/Managed/OpenSans-Light_744839258.ttf";


//Config for Linux
const FONT_PATH: &str = "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf";


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


fn handle_left_click(platform: &mut Platform, ui_state: &mut UIState, x: f32, y: f32, click_map: &Vec<ClickMapEntry>) -> Option<Url> {

    ui::handle_possible_ui_click(platform, ui_state, x, y);

    for click_map_entry in click_map {
        if click_map_entry.region.is_inside(x, y) {
            if click_map_entry.optional_link_url.is_some() {
                return click_map_entry.optional_link_url.clone();
            }
        }
    }

    return None;
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


pub fn load_url(url: &Url) -> RefCell<Document> {
    let page_content = resource_loader::load_text(&url);
    let lex_result = html_lexer::lex_html(&page_content);
    let dom_tree = html_parser::parse(lex_result, url);
    return dom_tree;
}


fn main() -> Result<(), String> {
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
    };

    let mut should_reload_from_url = false;
    let mut document = load_url(&url);
    let mut currently_loading_new_page = true;

    let mut previous_frame_page_height = 0.0;
    let mut previous_frame_click_map = Vec::new();

    let mut event_pump = platform.sdl_context.event_pump()?;
    'main_loop: loop {
        let start_loop_instant = Instant::now();

        if should_reload_from_url {
            #[cfg(feature="timings")] let start_page_load_instant = Instant::now();
            currently_loading_new_page = true;
            document = load_url(&url);
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
                        let page_scroll = ui::convert_block_drag_to_page_scroll(&mut ui_state, yrel as f32, previous_frame_page_height);
                        ui_state.current_scroll_y = clamp_scroll_position(ui_state.current_scroll_y + page_scroll, previous_frame_page_height);
                    }
                },
                SdlEvent::MouseButtonDown { mouse_btn: MouseButton::Left, x: mouse_x, y: mouse_y, .. } => {
                    mouse_state.x = mouse_x;
                    mouse_state.y = mouse_y;
                    mouse_state.click_start_x = mouse_x;
                    mouse_state.click_start_y = mouse_y;
                    mouse_state.left_down = true;

                    //TODO: its probably nicer to call a generic method in UI, to check any drags and update the mouse state
                    if ui::mouse_on_scrollblock(&mouse_state, ui_state.current_scroll_y, previous_frame_page_height) {
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
                        let optional_url = handle_left_click(&mut platform, &mut ui_state, mouse_x as f32, new_mouse_y, &previous_frame_click_map);

                        if optional_url.is_some() {
                            let url = optional_url.unwrap();
                            //TODO: this should be done via a nicer "navigate" method or something (also below when pressing enter in the addressbar
                            ui_state.addressbar.set_text(&mut platform, url.to_string());
                            document = load_url(&url);  // we should do this above in the next loop, just schedule the url for reload
                            currently_loading_new_page = true;
                        }
                    }
                },
                SdlEvent::MouseWheel { y, direction, .. } => {
                    match direction {
                        sdl2::mouse::MouseWheelDirection::Normal => {
                            //TODO: someday it might be nice to implement smooth scrolling (animate the movement over frames)
                            ui_state.current_scroll_y = clamp_scroll_position(ui_state.current_scroll_y - (y * SCROLL_SPEED) as f32, previous_frame_page_height);
                        },
                        sdl2::mouse::MouseWheelDirection::Flipped => {},
                        sdl2::mouse::MouseWheelDirection::Unknown(_) => debug_log_warn("Unknown mousewheel direction unknown!"),
                    }
                },
                SdlEvent::KeyUp { keycode, .. } => {
                    if keycode.is_some() {
                        let key_code = platform.convert_key_code(&keycode.unwrap());
                        ui::handle_keyboard_input(&mut platform, None, key_code, &mut ui_state);

                        if ui_state.addressbar.has_focus && keycode.unwrap().name() == "Return" {
                            //TODO: This is here for now because we need to load another page, not sure how to correctly trigger that from inside the component
                            url = Url::from(&ui_state.addressbar.text);
                            document = load_url(&url); // we should do this above in the next loop, just schedule the url for reload
                            currently_loading_new_page = true;
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

        /*** below we should not have any mutable ref to the DOM any more (because we take references in the layout tree) ***/

        #[cfg(feature="timings")] let start_layout_instant = Instant::now();
        let full_layout_tree = layout::build_full_layout(&document.borrow(), &mut platform, &url);
        debug_assert!(full_layout_tree.root_node.rects.borrow().len() == 1);
        previous_frame_page_height = full_layout_tree.page_height();
        previous_frame_click_map = layout::compute_click_map(&full_layout_tree, ui_state.current_scroll_y);
        #[cfg(feature="timings")] println!("layout elapsed millis: {}", start_layout_instant.elapsed().as_millis());

        #[cfg(feature="timings")] let start_render_instant = Instant::now();
        render(&mut platform, &full_layout_tree, &mut ui_state);
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
