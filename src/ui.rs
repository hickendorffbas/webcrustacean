use std::time::{SystemTime, UNIX_EPOCH};

use crate::network::url::Url;
use crate::{MouseState, SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::color::Color;
use crate::platform::{KeyCode, Platform, Position};
use crate::ui_components::{NavigationButton, TextField};


pub const HEADER_HEIGHT: f32 = 50.0;
pub const SIDE_SCROLLBAR_WIDTH: f32 = 20.0;
const UI_BASIC_COLOR: Color = Color::new(212, 208, 200);
const UI_BASIC_DARKER_COLOR: Color = Color::new(116, 107, 90);

pub const CONTENT_HEIGHT: f32 = SCREEN_HEIGHT - HEADER_HEIGHT;
pub const CONTENT_WIDTH: f32 = SCREEN_WIDTH - SIDE_SCROLLBAR_WIDTH;
pub const CONTENT_TOP_LEFT_X: f32 = 0.0;
pub const CONTENT_TOP_LEFT_Y: f32 = HEADER_HEIGHT;

const SCROLLBAR_HEIGHT: f32 = SCREEN_HEIGHT - HEADER_HEIGHT;
const SCROLLBAR_X_POS: f32 = SCREEN_WIDTH - SIDE_SCROLLBAR_WIDTH;


pub struct History {
    pub list: Vec<Url>,
    pub position: usize,
    pub currently_navigating_from_history: bool,
}


pub struct UIState {
    pub addressbar: TextField,
    pub current_scroll_y: f32,
    pub back_button: NavigationButton,
    pub forward_button: NavigationButton,
    pub history: History,
    pub currently_loading_page: bool,
    pub animation_tick: u32,
}


pub fn render_ui(platform: &mut Platform, ui_state: &mut UIState, page_height: f32) {
    update_animation_state(ui_state);
    render_header(platform, ui_state);
    render_scrollbar(platform, ui_state.current_scroll_y, page_height);
}


pub fn mouse_on_scrollblock(mouse_state: &MouseState, current_scroll_y: f32, page_height: f32) -> bool {
    let (block_x, block_y, block_width, block_height) = compute_scrollblock_position(current_scroll_y, page_height);
    return mouse_state.x > block_x as i32 && mouse_state.x < (block_x + block_width) as i32
           &&
           mouse_state.y > block_y as i32 && mouse_state.y < (block_y + block_height) as i32
}


pub fn handle_keyboard_input(platform: &mut Platform, input: Option<&String>, key_code: Option<KeyCode>, ui_state: &mut UIState) {
    if ui_state.addressbar.has_focus {
        ui_state.addressbar.handle_keyboard_input(platform, input, key_code);
    }
}


pub fn insert_text(platform: &mut Platform, ui_state: &mut UIState, text: &String) {
    if ui_state.addressbar.has_focus {
        ui_state.addressbar.insert_text(platform, text);
    }
}


pub fn current_focus_can_receive_text(ui_state: &UIState) -> bool {
    return ui_state.addressbar.has_focus;
}


pub fn handle_possible_ui_click(platform: &mut Platform, ui_state: &mut UIState, x: f32, y: f32) -> Option<Url> {

    //TODO: I think this should also handle the scrollbar, but we now handle that in main still

    ui_state.addressbar.click(x, y);
    let possible_url = ui_state.back_button.click(x, y, &mut ui_state.history);
    if possible_url.is_some() {
        return possible_url;
    }
    let possible_url = ui_state.forward_button.click(x, y, &mut ui_state.history);
    if possible_url.is_some() {
        return possible_url;
    }

    //The below code is currently a bit more generic than it needs to be, but this makes that the enable/disable doesn't break when we add other textfields...
    let any_text_field_has_focus = ui_state.addressbar.has_focus;

    if any_text_field_has_focus {
        platform.enable_text_input();
    } else {
        platform.disable_text_input();
    }

    return None;
}


pub fn convert_block_drag_to_page_scroll(ui_state: &UIState, scroll_block_amount_moved: f32, page_height: f32) -> f32 {
    let (_, _, _, block_height) = compute_scrollblock_position(ui_state.current_scroll_y, page_height);

    let movable_space = SCROLLBAR_HEIGHT - block_height;
    let relatively_moved = scroll_block_amount_moved / movable_space;
    return (page_height - CONTENT_HEIGHT) * relatively_moved;
}


fn render_header(platform: &mut Platform, ui_state: &UIState) {
    platform.fill_rect(0.0, 0.0, SCREEN_WIDTH, HEADER_HEIGHT, Color::WHITE, 255);

    platform.draw_line(Position { x: 0.0, y: HEADER_HEIGHT - 1.0 },
                       Position { x: SCREEN_WIDTH, y: HEADER_HEIGHT - 1.0 },
                       Color::BLACK);

    if ui_state.currently_loading_page {
        render_spinner(platform, ui_state);
    }

    ui_state.back_button.render(platform);
    ui_state.forward_button.render(platform);
    ui_state.addressbar.render(&ui_state, platform);
}


fn render_spinner(platform: &mut Platform, ui_state: &UIState) {
    let block_size = 5.0;
    let block_spacing = 15.0;
    let spinner_x_pos = ui_state.addressbar.x + ui_state.addressbar.width + 15.0;
    let spinner_y_pos = (ui_state.addressbar.y + ui_state.addressbar.height / 2.0) - (block_size / 2.0);

    let number_of_blocks = (ui_state.animation_tick % 1000) / 250;

    if number_of_blocks > 0 {
        platform.fill_rect(spinner_x_pos, spinner_y_pos, block_size, block_size, Color::BLACK, 255);
    }
    if number_of_blocks > 1 {
        platform.fill_rect(spinner_x_pos + block_spacing, spinner_y_pos, block_size, block_size, Color::BLACK, 255);
    }
    if number_of_blocks > 2 {
        platform.fill_rect(spinner_x_pos + (block_spacing * 2.0), spinner_y_pos, block_size, block_size, Color::BLACK, 255);
    }
}


fn render_scrollbar(platform: &mut Platform, current_scroll_y: f32, page_height: f32) {
    //TODO: I don't like that we are using HEADER_HEIGHT etc. here. The scrollbar should only know where it should draw, and we should derive that from
    //      header hight etc. outside this function

    platform.fill_rect(SCROLLBAR_X_POS, HEADER_HEIGHT, SCREEN_WIDTH - SCROLLBAR_X_POS, SCROLLBAR_HEIGHT, UI_BASIC_COLOR, 255);

    let (block_x, block_y, block_width, block_height) = compute_scrollblock_position(current_scroll_y, page_height);
    platform.fill_rect(block_x, block_y, block_width, block_height, UI_BASIC_DARKER_COLOR, 255);
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


fn update_animation_state(ui_state: &mut UIState) {
    let current_millis = SystemTime::now().duration_since(UNIX_EPOCH)
                            .expect("Time went backwards, please check if you entered a wormhole").as_millis();
    ui_state.animation_tick = (current_millis % 10_000) as u32;
}
