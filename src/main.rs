mod debug;
mod dom;
mod fonts;
mod html_parser;
mod layout;
mod lexer;
mod network;
mod renderer;

use std::collections::HashMap;
use std::env;
use std::fs;
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::debug::debug_log_warn;
use crate::fonts::{Font, FontCache};
use crate::layout::{FullLayout, LayoutNode};
use crate::network::http_get;
use crate::renderer::{Color, clear as renderer_clear, render_text};

use sdl2::{
    event::Event as SdlEvent,
    keyboard::Keycode,
    mouse::MouseButton,
    render::WindowCanvas,
    Sdl
};



//Config:
const FONT_PATH: &str = "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf";
const FONT_SIZE: u16 = 20;
const TARGET_FPS: u32 = 60;
const SCREEN_WIDTH: u32 = 1000;
const SCREEN_HEIGHT: u32 = 700;
const LAYOUT_MARGIN_HORIZONTAL : u32 = 10;
const VERTICAL_ELEMENT_SPACING : u32 = 10;
const HORIZONTAL_ELEMENT_SPACING: u32 = 10;
const DEFAULT_LOCATION_TO_LOAD: &str = "file://testinput/doc.html";
 

//Non-config constants:
const TARGET_MS_PER_FRAME: u128 = 1000 / TARGET_FPS as u128;




fn build_canvas(sdl_context: &Sdl) -> WindowCanvas {
    let video_subsystem = sdl_context.video()
        .expect("Could not get the video subsystem");

    let window = video_subsystem.window("BBrowser", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");

    let canvas = window.into_canvas().build()
        .expect("could not make a canvas");

    return canvas;
}


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


fn render(canvas: &mut WindowCanvas, full_layout: &FullLayout, font_cache: &mut FontCache) {
    //TODO: I'm not sure if I want the renderer to be the thing that takes the layoutnodes,
    //      I think so? In that case this method should move there -> well, in the renderer, but currently the renderer contains all kinds of
    //      SDL specific stuff, that should move one layer further (platform or something)

    renderer_clear(canvas, Color::WHITE);
    render_layout_node(canvas, &full_layout.root_node, font_cache);

    canvas.present();
}


fn render_layout_node(canvas: &mut WindowCanvas, layout_node: &LayoutNode, font_cache: &mut FontCache) {
    if layout_node.text.is_some() {
        let own_font = Font::new(layout_node.font_bold, layout_node.font_size); //TODO: we should just have a (reference to) the font on the layout node
        let font = font_cache.get_font(&own_font);

        let x = layout_node.position.x;
        let y = layout_node.position.y;
        render_text(canvas, layout_node.text.as_ref().unwrap(), x, y, &font, layout_node.font_color.to_sdl_color());
    }

    if layout_node.children.is_some() {
        for child in layout_node.children.as_ref().unwrap() {
            render_layout_node(canvas, &child, font_cache);
        }
    }
}


fn handle_left_click(x: u32, y: u32, layout_tree: &FullLayout) {

    fn check_left_click_for_layout_node(x: u32, y: u32, layout_node: &Rc<LayoutNode>) {
        if layout_node.position.x > x || layout_node.position.y > y { //TODO: this check should take the width and height into account, but we don't have that on the layout node yet
            return;
        }

        if layout_node.optional_link_url.is_some() {
            println!("Link found: {}", layout_node.optional_link_url.as_ref().unwrap());
            return;
        }

        if layout_node.children.is_some() {
            for child in layout_node.children.as_ref().unwrap() {
                check_left_click_for_layout_node(x, y, &child);
            }
        }

    }

    check_left_click_for_layout_node(x, y, &layout_tree.root_node);
}


fn main() -> Result<(), String> {

    let args: Vec<String> = env::args().collect();

    let url: String;
    if args.len() < 2 {
        url = String::from(DEFAULT_LOCATION_TO_LOAD);
    } else {
        url = args[1].clone();
    }
    let loading_local_file = url.starts_with("file://");


    let sdl_context = sdl2::init()?;
    let mut canvas = build_canvas(&sdl_context);


    let ttf_context = sdl2::ttf::init()
                                .expect("could not initialize the font system");
    let mut font_cache = FontCache {ttf_context: &ttf_context, mapping: HashMap::new()};


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


    //TODO: temporary call to lexer below, for testing, output is not used yet:
    let _lex_result = lexer::lex_html(&file_contents);

    let document_node = html_parser::parse_document(&file_contents);
    let full_layout_tree = layout::build_full_layout(&document_node, &mut font_cache);

    let mut event_pump = sdl_context.event_pump()?;
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

        render(&mut canvas, &full_layout_tree, &mut font_cache);
        frame_time_check(&start_instant, currently_loading_new_page);
        currently_loading_new_page = false;
    }

    Ok(())
}
