#![allow(unused_parens)]

mod debug;
mod fonts;
mod html_parser;
mod layout;
mod network;
mod renderer;

use std::collections::HashMap;
use std::env;
use std::fs;
use std::time::{Duration, Instant};

use crate::fonts::{Font, FontCache};
use crate::layout::{ClickBox, LayoutNode};
use crate::network::http_get;
use crate::renderer::{clear as renderer_clear, draw_line, Color, Position, render_text};

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


fn frame_time_check(start_instant: &Instant) {
    let millis_elapsed = start_instant.elapsed().as_millis();
    let sleep_time_millis = (TARGET_MS_PER_FRAME as i64 - millis_elapsed as i64);
    if (sleep_time_millis > 1) {
        //If we are more than a millisecond faster than what we need to reach the target FPS, we sleep
        ::std::thread::sleep(Duration::from_millis(sleep_time_millis as u64))
    } else {
        //println!("Warning: we did not reach the target FPS, frametime: {}", millis_elapsed); //TODO: temporarily disabled, re-enable later
    }
}


fn render(canvas: &mut WindowCanvas, layout_nodes: &Vec<LayoutNode>, font_cache: &mut FontCache) {
    //TODO: I'm not sure if I want the renderer to be the thing that takes the layoutnodes, I think so? In that case this method should move there.

    renderer_clear(canvas, Color::WHITE);

    for layout_node in layout_nodes {
        if (layout_node.text.is_none()) {
            continue;
        }

        let x = layout_node.position.x;
        let y = layout_node.position.y;

        let own_font = Font::new(layout_node.bold, layout_node.font_size); //TODO: we should just have a (reference to) the font on the layout node

        let font = font_cache.get_font(&own_font);

        render_text(canvas, layout_node.text.as_ref().unwrap(), x, y, &font);
    }

    //temp test:
    draw_line(canvas, Position::new(100, 100), Position::new(200, 200), Color::RED);
    draw_line(canvas, Position::new(100, 200), Position::new(200, 100), Color::BLUE);

    canvas.present();
}


fn handle_left_click(x : i32, y: i32, click_boxes: &Vec<ClickBox>) {
    //TODO: check clickboxes!
    println!("Mouse clicked: {} {}", x, y);
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
    if (loading_local_file) {
        let mut file_path = url[7..] //remove the "file://" prefix
                            .to_owned();
        println!("file_path: {:?}", file_path);
        file_contents = fs::read_to_string(file_path)
                                .expect("Something went wrong reading the file");
    } else {
        file_contents = http_get(String::from(url));
    }


    let root_nodes = html_parser::parse_document(&file_contents);
    let layout_nodes = layout::build_layout_list(&root_nodes, &mut font_cache);

    let click_boxes = layout::compute_click_boxes(&layout_nodes);

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
                    //handle_left_click(mouse_x, mouse_y, &click_boxes); //TODO: disabled because something is wrong with types?
                }
                _ => {}
            }
        }

        render(&mut canvas, &layout_nodes, &mut font_cache);
        frame_time_check(&start_instant);
    }

    Ok(())
}
