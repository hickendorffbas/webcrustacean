use crate::{
    MouseState,
    SCREEN_HEIGHT,
    SCREEN_WIDTH,
};
use crate::color::Color;
use crate::fonts::Font;
use crate::platform::{Platform, Position};


pub const HEADER_HEIGHT: f32 = 50.0;
pub const SIDE_SCROLLBAR_WIDTH: f32 = 20.0;
const UI_BASIC_COLOR: Color = Color::new(212, 208, 200);
const UI_BASIC_DARKER_COLOR: Color = Color::new(116, 107, 90);


pub const CONTENT_HEIGHT: f32 = SCREEN_HEIGHT - HEADER_HEIGHT;
pub const CONTENT_WIDTH: f32 = SCREEN_WIDTH - SIDE_SCROLLBAR_WIDTH;
pub const CONTENT_TOP_LEFT_X: f32 = 0.0;
pub const CONTENT_TOP_LEFT_Y: f32 = HEADER_HEIGHT;


pub struct UIState {
    pub addressbar_has_focus: bool,
    pub addressbar_text: String,
    pub current_scroll_y: f32,
}


pub fn render_ui(platform: &mut Platform, ui_state: &UIState, page_height: f32) {
    render_header(platform, ui_state);
    render_scrollbar(platform, ui_state.current_scroll_y, page_height);
}


pub fn mouse_on_scrollblock(mouse_state: &MouseState, current_scroll_y: f32, page_height: f32) -> bool {
    let (block_x, block_y, block_width, block_height) = compute_scrollblock_positions(current_scroll_y, page_height);
    return mouse_state.x > block_x as i32 && mouse_state.x < (block_x + block_width) as i32
           &&
           mouse_state.y > block_y as i32 && mouse_state.y < (block_y + block_height) as i32
}


pub fn handle_keyboard_input(input: Option<&String>, is_backspace: bool, ui_state: &mut UIState) {
    if ui_state.addressbar_has_focus {
        if is_backspace {
            let mut chars_iter = ui_state.addressbar_text.chars();
            chars_iter.next_back();
            ui_state.addressbar_text = chars_iter.collect::<String>()
        } else {
            ui_state.addressbar_text.push_str(input.unwrap());
        }
    }
}


fn render_header(platform: &mut Platform, ui_state: &UIState) {
    platform.fill_rect(0.0, 0.0, SCREEN_WIDTH as u32, HEADER_HEIGHT as u32, Color::WHITE);

    let font = Font::new(true, 14);
    platform.render_text(&"Bbrowser".to_owned(), 10.0, 10.0, &font, Color::BLACK);
    platform.draw_line(Position { x: 0.0, y: HEADER_HEIGHT - 1.0 },
                       Position { x: SCREEN_WIDTH, y: HEADER_HEIGHT - 1.0 },
                       Color::BLACK);

    render_address_bar(platform, ui_state);
}


fn render_address_bar(platform: &mut Platform, ui_state: &UIState) {
    let x = 100.0;
    let y = 10.0;
    let width = SCREEN_WIDTH - 200.0;
    let height = 35.0;

    draw_square(platform, x, y, width, height, Color::BLACK);

    let font = Font::new(false, 18);
    platform.render_text(&ui_state.addressbar_text, x + 5.0, y + 5.0, &font, Color::BLACK);
}


//TODO: fill rect is on platform, should this be too?
fn draw_square(platform: &mut Platform, x: f32, y: f32, width: f32, height: f32, color: Color) {
    let p1 = Position { x, y };
    let p2 = Position { x: width + x, y };
    let p3 = Position { x: width + x, y: height + y };
    let p4 = Position { x, y: height + y };

    platform.draw_line(p1, p2, color);
    platform.draw_line(p2, p3, color);
    platform.draw_line(p3, p4, color);
    platform.draw_line(p4, p1, color);
}


fn render_scrollbar(platform: &mut Platform, current_scroll_y: f32, page_height: f32) {
    //TODO: I don't like that we are using HEADER_HEIGHT etc. here. The scrollbar should only know where it should draw, and we should derive that from
    //      header hight etc. outside this function

    let scrollbar_start_x = SCREEN_WIDTH - SIDE_SCROLLBAR_WIDTH;
    let scrollbar_height = SCREEN_HEIGHT - HEADER_HEIGHT;
    platform.fill_rect(scrollbar_start_x, HEADER_HEIGHT, (SCREEN_WIDTH - scrollbar_start_x) as u32, scrollbar_height as u32, UI_BASIC_COLOR);

    let (block_x, block_y, block_width, block_height) = compute_scrollblock_positions(current_scroll_y, page_height);
    platform.fill_rect(block_x, block_y, block_width as u32, block_height as u32, UI_BASIC_DARKER_COLOR);
}


fn compute_scrollblock_positions(current_scroll_y: f32, page_height: f32) -> (f32, f32, f32, f32) {
    let scrollbar_start_x = SCREEN_WIDTH - SIDE_SCROLLBAR_WIDTH;
    let scrollbar_height = SCREEN_HEIGHT - HEADER_HEIGHT;

    let scrollbar_height_per_page_y = scrollbar_height / page_height;
    let relative_size_of_scroll_block = CONTENT_HEIGHT / page_height;

    //TODO: I do need to account that we don't scroll the bottom of the page all the way to the top
    let top_scroll_block_y = (scrollbar_height_per_page_y * current_scroll_y) + HEADER_HEIGHT;

    //TODO: we probably should clamp this to a minimum
    let scroll_block_height = relative_size_of_scroll_block * scrollbar_height;

    return (scrollbar_start_x, top_scroll_block_y, (SCREEN_WIDTH - scrollbar_start_x), scroll_block_height);
}
