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

use renderer::render;
use sdl2::{
    event::Event as SdlEvent,
    keyboard::Keycode,
    mouse::MouseButton,
};



//Config:
const FONT_PATH: &str = "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf";
const TARGET_FPS: u32 = if cfg!(debug_assertions) { 30 } else { 60 };
const SCREEN_WIDTH: u32 = 1000;
const SCREEN_HEIGHT: u32 = 700;
const DEFAULT_LOCATION_TO_LOAD: &str = "file://testinput/doc.html";


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


    let mut event_pump = platform.sdl_context.event_pump()?;
    'main_loop: loop {
        let start_instant = Instant::now();

        for event in event_pump.poll_iter() {
            match event {
                SdlEvent::Quit {..} |
                SdlEvent::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'main_loop;
                },
                SdlEvent::MouseButtonUp { mouse_btn: MouseButton::Left, x: mouse_x, y: mouse_y, .. } => {
                    handle_left_click(mouse_x as u32, mouse_y as u32, &full_layout_tree);
                }
                _ => {}
            }
        }

        render(&mut platform, &full_layout_tree);
        frame_time_check(&start_instant, currently_loading_new_page);
        currently_loading_new_page = false;
    }

    Ok(())
}
