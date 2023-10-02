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

const ADDRESSBAR_X: f32 = 100.0;
const ADDRESSBAR_Y: f32 = 10.0;
const ADDRESSBAR_WIDTH: f32 = SCREEN_WIDTH - 200.0;
const ADDRESSBAR_HEIGHT: f32 = 35.0;
const TEXT_OFFSET_FROM_ADDRESS_BAR_BORDER: f32 = 5.0;

const SCROLLBAR_HEIGHT: f32 = SCREEN_HEIGHT - HEADER_HEIGHT;
const SCROLLBAR_X_POS: f32 = SCREEN_WIDTH - SIDE_SCROLLBAR_WIDTH;



pub struct AddressbarState {
    pub has_focus: bool,
    pub cursor_visible: bool,
    pub cursor_text_position: usize,
    pub text: String,
}


pub struct UIState {
    pub addressbar: AddressbarState,
    pub current_scroll_y: f32,
}


pub fn render_ui(platform: &mut Platform, ui_state: &UIState, page_height: f32) {
    render_header(platform, ui_state);
    render_scrollbar(platform, ui_state.current_scroll_y, page_height);
}


pub fn mouse_on_scrollblock(mouse_state: &MouseState, current_scroll_y: f32, page_height: f32) -> bool {
    let (block_x, block_y, block_width, block_height) = compute_scrollblock_position(current_scroll_y, page_height);
    return mouse_state.x > block_x as i32 && mouse_state.x < (block_x + block_width) as i32
           &&
           mouse_state.y > block_y as i32 && mouse_state.y < (block_y + block_height) as i32
}


pub fn handle_keyboard_input(input: Option<&String>, is_backspace: bool, ui_state: &mut UIState) {
    if ui_state.addressbar.has_focus {
        if is_backspace {
            if ui_state.addressbar.cursor_text_position > 0 {
                ui_state.addressbar.text.remove(ui_state.addressbar.cursor_text_position - 1);
                ui_state.addressbar.cursor_text_position -= 1;
            }
        } else {
            ui_state.addressbar.text.insert_str(ui_state.addressbar.cursor_text_position, input.unwrap());
            ui_state.addressbar.cursor_text_position += 1;
        }
    }
}


pub fn handle_possible_ui_click(platform: &mut Platform, ui_state: &mut UIState, x: f32, y: f32) {
    ui_state.addressbar.has_focus = false;

    //TODO: I think this should also handle the scrollbar, but we now handle that in main still

    if x > ADDRESSBAR_X && x < (ADDRESSBAR_X + ADDRESSBAR_WIDTH) {
        if y > ADDRESSBAR_Y && y < (ADDRESSBAR_Y + ADDRESSBAR_HEIGHT) {
            ui_state.addressbar.has_focus = true;
        }
    }

    if ui_state.addressbar.has_focus {
        platform.enable_text_input();
    } else {
        platform.disable_text_input();
    }
}


pub fn convert_block_drag_to_page_scroll(ui_state: &UIState, scroll_block_amount_moved: f32, page_height: f32) -> f32 {
    let (_, _, _, block_height) = compute_scrollblock_position(ui_state.current_scroll_y, page_height);

    let movable_space = SCROLLBAR_HEIGHT - block_height;
    let relatively_moved = scroll_block_amount_moved / movable_space;
    return (page_height - CONTENT_HEIGHT) * relatively_moved;
}


fn render_header(platform: &mut Platform, ui_state: &UIState) {
    platform.fill_rect(0.0, 0.0, SCREEN_WIDTH, HEADER_HEIGHT, Color::WHITE);

    let font = Font::new(true, 14);
    platform.render_text(&"Bbrowser".to_owned(), 10.0, 10.0, &font, Color::BLACK);
    platform.draw_line(Position { x: 0.0, y: HEADER_HEIGHT - 1.0 },
                       Position { x: SCREEN_WIDTH, y: HEADER_HEIGHT - 1.0 },
                       Color::BLACK);

    render_address_bar(platform, ui_state);
}


fn render_address_bar(platform: &mut Platform, ui_state: &UIState) {
    platform.draw_square(ADDRESSBAR_X, ADDRESSBAR_Y, ADDRESSBAR_WIDTH, ADDRESSBAR_HEIGHT, Color::BLACK);

    let font = Font::new(false, 18);
    platform.render_text(&ui_state.addressbar.text, ADDRESSBAR_X + TEXT_OFFSET_FROM_ADDRESS_BAR_BORDER,
                         ADDRESSBAR_Y + TEXT_OFFSET_FROM_ADDRESS_BAR_BORDER, &font, Color::BLACK);

    if ui_state.addressbar.cursor_visible && ui_state.addressbar.has_focus {
        let char_position_mapping = compute_char_position_mapping(platform, &font, &ui_state.addressbar.text);
        let cursor_position = char_position_mapping[ui_state.addressbar.cursor_text_position] + ADDRESSBAR_X + TEXT_OFFSET_FROM_ADDRESS_BAR_BORDER;

        let cursor_top_bottom_margin = 2.0;
        let cursor_bottom_pos = (ADDRESSBAR_Y + ADDRESSBAR_HEIGHT) - cursor_top_bottom_margin;
        platform.draw_line(Position { x: cursor_position, y: ADDRESSBAR_Y + cursor_top_bottom_margin}, Position { x: cursor_position, y: cursor_bottom_pos },
                           Color::BLACK);
    }
}


fn compute_char_position_mapping(platform: &mut Platform, font: &Font, text: &String) -> Vec<f32> {
    //TODO: we take a very slow approach here. Not sure if we can do this faster, but we should probably at least cache it....

    let mut char_position_mapping = Vec::new();
    char_position_mapping.push(0.0);

    for i in 1..text.len()+1 {
        let (x_pos, _) = platform.get_text_dimension_str(&text[0..i], font);
        char_position_mapping.push(x_pos);
    }

    return char_position_mapping;
}


fn render_scrollbar(platform: &mut Platform, current_scroll_y: f32, page_height: f32) {
    //TODO: I don't like that we are using HEADER_HEIGHT etc. here. The scrollbar should only know where it should draw, and we should derive that from
    //      header hight etc. outside this function

    platform.fill_rect(SCROLLBAR_X_POS, HEADER_HEIGHT, SCREEN_WIDTH - SCROLLBAR_X_POS, SCROLLBAR_HEIGHT, UI_BASIC_COLOR);

    let (block_x, block_y, block_width, block_height) = compute_scrollblock_position(current_scroll_y, page_height);
    platform.fill_rect(block_x, block_y, block_width, block_height, UI_BASIC_DARKER_COLOR);
}


fn compute_scrollblock_position(current_scroll_y: f32, page_height: f32) -> (f32, f32, f32, f32) {
    let scrollbar_height_per_page_y = SCROLLBAR_HEIGHT / page_height;
    let relative_size_of_scroll_block = CONTENT_HEIGHT / page_height;

    //TODO: I do need to account that we don't scroll the bottom of the page all the way to the top
    let top_scroll_block_y = (scrollbar_height_per_page_y * current_scroll_y) + HEADER_HEIGHT;

    //TODO: we probably should clamp this to a minimum
    let scroll_block_height = relative_size_of_scroll_block * SCROLLBAR_HEIGHT;

    return (SCROLLBAR_X_POS, top_scroll_block_y, (SCREEN_WIDTH - SCROLLBAR_X_POS), scroll_block_height);
}
