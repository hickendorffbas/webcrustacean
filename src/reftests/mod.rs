use std::collections::HashMap;
use std::{
    fs::File,
    io::Read,
    path::Path,
    thread,
    time
};

use image::ImageBuffer;
use image::Rgba;
use sdl2::rect::Rect as SdlRect;
use threadpool::ThreadPool;

use crate::html_lexer;
use crate::html_parser;
use crate::layout;
use crate::network::url::Url;
use crate::platform::{Platform, self};
use crate::renderer;
use crate::resource_loader::{CookieStore, ResourceThreadPool};
use crate::ui::{
    UIState,
    CONTENT_TOP_LEFT_X,
    CONTENT_TOP_LEFT_Y,
};


const DEFAULT_SCREEN_WIDTH: f32 = 600.0;
const DEFAULT_SCREEN_HEIGHT: f32 = 600.0;
const WAIT_FOR_SDL2_MILLIS: u64 = 250;


fn read_file(file_path: &Path) -> String {
    let mut file = match File::open(&file_path) {
        Err(err) => panic!("Could not open {}: {}", file_path.display(), err),
        Ok(file) => file,
    };

    let mut string_data = String::new();
    match file.read_to_string(&mut string_data) {
        Err(why) => panic!("couldn't read {}: {}", file_path.display(), why),
        Ok(_) => {},
    }

    return string_data;
}


fn render_doc(filename: &str, platform: &mut Platform, save_output: bool) -> Vec<u8> {
    let html = read_file(Path::new(&filename));

    let url = Url::empty();

    let lex_result = html_lexer::lex_html(&html);
    let mut document = html_parser::parse(lex_result, &url);

    document.document_node.borrow_mut().post_construct(platform);
    document.update_all_dom_nodes(&mut ResourceThreadPool { pool: ThreadPool::new(1) }, &CookieStore { cookies_by_domain: HashMap::new() });

    let full_layout = layout::build_full_layout(&document, &platform.font_context);

    let mut ui_state = UIState::new(DEFAULT_SCREEN_WIDTH, DEFAULT_SCREEN_HEIGHT);
    ui_state.current_scroll_y = 0.0;
    ui_state.currently_loading_page = false;

    layout::compute_layout(&full_layout.root_node, CONTENT_TOP_LEFT_X, CONTENT_TOP_LEFT_Y, &platform.font_context,
                           ui_state.current_scroll_y, false, true, ui_state.window_dimensions.content_viewport_width);

    renderer::render(platform, &full_layout, &mut ui_state);

    thread::sleep(time::Duration::from_millis(WAIT_FOR_SDL2_MILLIS));

    let rect = SdlRect::new(0, 0, DEFAULT_SCREEN_WIDTH as u32, DEFAULT_SCREEN_HEIGHT as u32);
    let pixels = platform.canvas.read_pixels(rect, platform.canvas.default_pixel_format()).unwrap();

    if save_output {
        let img: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(DEFAULT_SCREEN_WIDTH as u32, DEFAULT_SCREEN_HEIGHT as u32, pixels.clone()).unwrap();
        let name = filename.split("/").last().unwrap().split(".").next().unwrap();
        img.save(Path::new(&("/home/bas/code/webcrustacean/reftest_dump_".to_owned() + name + ".bmp"))).map_err(|e| e.to_string()).expect("ERROR");
    }

    return pixels;
}


fn run_ref_test(test_number: usize, save_output: bool) {

    //TODO: platform init would be better to do once for all tests? Can we have state over tests even?
    let sdl_context = sdl2::init().unwrap();
    let mut platform = platform::init_platform(sdl_context, DEFAULT_SCREEN_WIDTH, DEFAULT_SCREEN_HEIGHT, true).unwrap();

    let file_name_a = String::from("reftest_data/") + test_number.to_string().as_str() + "a.html";
    let file_name_b = String::from("reftest_data/") + test_number.to_string().as_str() + "b.html";

    let render1 = render_doc(&file_name_a, &mut platform, save_output);
    let render2 = render_doc(&file_name_b, &mut platform, save_output);

    let renders_equal = render1 == render2;
    assert!(renders_equal);
}


#[test]
fn reftest_1() {
    //Check that whitespace in the html does not lead to extra whitespace on the rendered page
    run_ref_test(1, false);
}

