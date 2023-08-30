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
mod style;
mod ui;
#[cfg(test)] mod test_util; //TODO: is there a better (test-specific) place to define this?

use std::env;
use std::fs;
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::debug::debug_log_warn;
use crate::fonts::Font;
use crate::layout::{FullLayout, LayoutNode};
use crate::network::http_get;
use crate::ui::CONTENT_HEIGHT;

use renderer::render;
use sdl2::{
    event::Event as SdlEvent,
    keyboard::Keycode,
    mouse::MouseButton,
};



//Config:
const FONT_PATH: &str = "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf";
const TARGET_FPS: u32 = if cfg!(debug_assertions) { 30 } else { 60 };
const SCREEN_WIDTH: f32 = 1000.0;
const SCREEN_HEIGHT: f32 = 700.0;
const DEFAULT_LOCATION_TO_LOAD: &str = "file://testinput/doc.html";
const SCROLL_SPEED: i32 = 25;

//Non-config constants:
const TARGET_MS_PER_FRAME: u128 = 1000 / TARGET_FPS as u128;



fn frame_time_check(start_instant: &Instant, currently_loading_new_page: bool) {
    let millis_elapsed = start_instant.elapsed().as_millis();
    let sleep_time_millis = TARGET_MS_PER_FRAME as i64 - millis_elapsed as i64;
    if sleep_time_millis > 1 {
        //If we are more than a millisecond faster than what we need to reach the target FPS, we sleep
        ::std::thread::sleep(Duration::from_millis(sleep_time_millis as u64));
    } else {
        if !currently_loading_new_page {
            debug_log_warn(format!("we did not reach the target FPS, frametime: {}", millis_elapsed));
        }
    }
}



fn handle_left_click(x: u32, y: u32, layout_tree: &FullLayout) {

    fn check_left_click_for_layout_node(x: u32, y: u32, layout_node: &Rc<LayoutNode>) {

        let any_inside = layout_node.rects.borrow().iter().any(|rect| -> bool {rect.location.borrow().is_inside(x, y)});
        if !any_inside {
            return;
        }

        if layout_node.optional_link_url.is_some() {
            println!("Link found: {}", layout_node.optional_link_url.as_ref().unwrap());
            return;
        }

        if layout_node.children.is_some() {
            for child in layout_node.children.as_ref().unwrap() {
                if child.visible {
                    check_left_click_for_layout_node(x, y, &child);
                }
            }
        }
    }

    check_left_click_for_layout_node(x, y, &layout_tree.root_node);
}


pub struct MouseState {
    x: i32,
    y: i32,
    click_start_x: i32,
    click_start_y: i32,
    left_down: bool,
}


fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let ttf_context = sdl2::ttf::init()
                                .expect("could not initialize the font system");
    let mut platform = platform::init_platform(sdl_context, &ttf_context).unwrap();

    let args: Vec<String> = env::args().collect();

    let url: String;
    if args.len() < 2 {
        url = String::from(DEFAULT_LOCATION_TO_LOAD);
    } else {
        url = args[1].clone();
    }
    let loading_local_file = url.starts_with("file://");

    let file_contents: String;
    if loading_local_file {
        let file_path = url[7..] //remove the "file://" prefix
                        .to_owned();
        file_contents = fs::read_to_string(file_path)
                                .expect("Something went wrong reading the file");
    } else {
        file_contents = http_get(String::from(url));
    }

    let mut currently_loading_new_page = true;

    let lex_result = html_lexer::lex_html(&file_contents);
    let dom_tree = html_parser::parse(lex_result);
    let full_layout_tree = layout::build_full_layout(&dom_tree, &mut platform);

    let mut current_scroll_y = 0.0;

    debug_assert!(full_layout_tree.root_node.rects.borrow().len() == 1);
    let current_page_height = full_layout_tree.root_node.rects.borrow().iter().next().unwrap().location.borrow().height();


    let mut mouse_state = MouseState { x: 0, y: 0, click_start_x: 0, click_start_y: 0, left_down: false };

    let mut event_pump = platform.sdl_context.event_pump()?;
    'main_loop: loop {
        let start_instant = Instant::now();

        for event in event_pump.poll_iter() {
            match event {
                SdlEvent::Quit {..} |
                SdlEvent::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'main_loop;
                },
                SdlEvent::MouseMotion { x: mouse_x, y: mouse_y, yrel, .. } => {
                    mouse_state.x = mouse_x;
                    mouse_state.y = mouse_y;

                    if mouse_state.left_down && ui::mouse_started_on_scrollblock(&mouse_state, current_scroll_y, current_page_height) {
                        current_scroll_y = clamp_scroll_position(current_scroll_y + yrel as f32, current_page_height);
                    }
                },
                SdlEvent::MouseButtonDown { mouse_btn: MouseButton::Left, x: mouse_x, y: mouse_y, .. } => {
                    mouse_state.x = mouse_x;
                    mouse_state.y = mouse_y;
                    mouse_state.click_start_x = mouse_x;
                    mouse_state.click_start_y = mouse_y;
                    mouse_state.left_down = true;
                },
                SdlEvent::MouseButtonUp { mouse_btn: MouseButton::Left, x: mouse_x, y: mouse_y, .. } => {
                    mouse_state.x = mouse_x;
                    mouse_state.y = mouse_y;
                    mouse_state.left_down = false;

                    let abs_movement = (mouse_state.x - mouse_state.click_start_x).abs() + (mouse_state.y - mouse_state.click_start_y).abs();
                    let was_dragging = abs_movement > 1;

                    if !was_dragging {
                        handle_left_click(mouse_x as u32, (mouse_y as f32 + current_scroll_y) as u32, &full_layout_tree);
                    }
                },
                SdlEvent::MouseWheel { y, direction, .. } => {
                    match direction {
                        sdl2::mouse::MouseWheelDirection::Normal => {
                            //TODO: someday it might be nice to implement smooth scrolling (animate the movement over frames)
                            current_scroll_y = clamp_scroll_position(current_scroll_y - (y * SCROLL_SPEED) as f32, current_page_height);
                        },
                        sdl2::mouse::MouseWheelDirection::Flipped => {},
                        sdl2::mouse::MouseWheelDirection::Unknown(_) => debug_log_warn("Unknown mousewheel direction unknown!"),
                    }
                },
                _ => {}
            }
        }

        render(&mut platform, &full_layout_tree, current_scroll_y);
        frame_time_check(&start_instant, currently_loading_new_page);
        currently_loading_new_page = false;
    }

    Ok(())
}

fn clamp_scroll_position(current_scroll_y: f32, current_page_height: f32) -> f32 {
    if current_scroll_y < 0.0 {
        return 0.0;
    }
    let max_scroll_y = (current_page_height + 1.0) - CONTENT_HEIGHT;
    if current_scroll_y > max_scroll_y {
        return max_scroll_y;
    }
    return current_scroll_y;
}
