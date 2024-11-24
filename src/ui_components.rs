use std::sync::atomic::{AtomicUsize, Ordering};

use crate::color::Color;
use crate::debug::debug_log_warn;
use crate::layout::Rect;
use crate::network::url::Url;
use crate::platform::{
    fonts::Font,
    KeyCode,
    Platform,
    Position
};
use crate::ui::{
    History,
    UI_BASIC_COLOR,
    UI_BASIC_DARKER_COLOR,
    UIState
};


const TEXT_FIELD_OFFSET_FROM_BORDER: f32 = 5.0;
const CURSOR_BLINK_SPEED_MILLIS: u32 = 500;

static NEXT_COMPONENT_ID: AtomicUsize = AtomicUsize::new(1);
pub fn get_next_component_id() -> usize { NEXT_COMPONENT_ID.fetch_add(1, Ordering::Relaxed) }


#[cfg_attr(debug_assertions, derive(Debug))]
pub enum PageComponent {
    Button(Button),
    TextField(TextField),
}
impl PageComponent {
    pub fn get_id(&self) -> usize {
        match self {
            PageComponent::Button(button) => button.id,
            PageComponent::TextField(text_field) => text_field.id,
        }
    }
    pub fn click(&mut self, x: f32, y: f32) {
        match self {
            PageComponent::Button(button) => button.click(),
            PageComponent::TextField(text_field) => text_field.click(x, y),
        }
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct TextField {
    pub id: usize,

    pub x: f32, //NOTE: x and y are the absolute positions in the window, not content positions in the page.
    pub y: f32, //TODO: we need to make sure x and y are updated when the page is scrolled (for <input> TextFields) and disable / hide them when outside of the window
    pub width: f32,
    pub height: f32,

    pub has_focus: bool,
    pub cursor_text_position: usize, // this position means the string index it is _before_, so starts at 0, and has length string.len()
    pub text: String,

    pub select_on_first_click: bool,
    pub selection_start_x: f32,
    pub selection_end_x: f32,
    pub selection_start_idx: usize,
    pub selection_end_idx: usize,

    pub font: Font,
    pub char_position_mapping: Vec<f32>,
}
impl TextField {
    pub fn new(x: f32, y: f32, width: f32, height: f32, select_on_first_click: bool) -> TextField {
        //we currently don't allow the text to be set in this constructor, because we then also need the platform in the constructor to compute char position mappings
        //   and that would mean that the DOM construction needs the platform (font context) too. Which is not nice.
        //TODO: it would be nicer to have the font_context (and other contexts) in some kind of global
        //      -> yes, we are going to make a lazy_static PLATFORM variable
        let font = Font::default();
        return TextField { id: get_next_component_id(), x, y, width, height, has_focus: false, cursor_text_position: 0, text: String::new(), select_on_first_click,
                           selection_start_x: 0.0, selection_end_x: 0.0, selection_start_idx: 0, selection_end_idx: 0, font, char_position_mapping: Vec::new() };
    }
    pub fn render(&self, ui_state: &UIState, platform: &mut Platform) {
        platform.draw_square(self.x, self.y, self.width, self.height, Color::BLACK, 255);

        if self.selection_start_x != self.selection_end_x {
            let start_x = if self.selection_start_x < self.selection_end_x { self.selection_start_x } else { self.selection_end_x };
            let end_x = if self.selection_start_x < self.selection_end_x { self.selection_end_x } else { self.selection_start_x };

            let y_start = self.y + TEXT_FIELD_OFFSET_FROM_BORDER;
            let height = self.height - (TEXT_FIELD_OFFSET_FROM_BORDER * 2.0);
            platform.fill_rect(start_x, y_start, end_x - start_x, height, Color::DEFAULT_SELECTION_COLOR, 255);
        }

        platform.render_text(&self.text, self.x + TEXT_FIELD_OFFSET_FROM_BORDER, self.y + TEXT_FIELD_OFFSET_FROM_BORDER, &self.font, Color::BLACK);

        if self.has_focus && !self.has_selection_active() {

            //TODO: also we need to make sure we reset the cycle whenever the cursor is moved, so it stays visible while using the arrow keys quickly
            let cursor_visible = ui_state.animation_tick % (CURSOR_BLINK_SPEED_MILLIS * 2) > CURSOR_BLINK_SPEED_MILLIS;
            if cursor_visible {
                let relative_cursor_position = if self.cursor_text_position == 0 {
                    0.0
                } else {
                    self.char_position_mapping[self.cursor_text_position - 1]
                };

                let cursor_position = relative_cursor_position + self.x + TEXT_FIELD_OFFSET_FROM_BORDER;
                let cursor_top_bottom_margin = 2.0;
                let cursor_bottom_pos = (self.y + self.height) - cursor_top_bottom_margin;
                platform.draw_line(Position { x: cursor_position, y: self.y + cursor_top_bottom_margin},
                                   Position { x: cursor_position, y: cursor_bottom_pos },
                                   Color::BLACK);
            }
        }
    }

    pub fn update_position(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.x = x;
        self.y = y;
        self.width = width;
        self.height = height;
    }

    pub fn set_text(&mut self, platform: &Platform, text: String) { //TODO: use this everywhere...
        self.clear_selection();
        self.text = text;

        if self.cursor_text_position > self.text.len() {
            self.cursor_text_position = self.text.len();
        }

        self.char_position_mapping = platform.font_context.compute_char_position_mapping(&self.font, &self.text);
    }

    pub fn insert_text(&mut self, platform: &Platform, text: &String) {
        if self.has_selection_active() {
            self.remove_selected_text(platform);
        }
        for char in text.chars() {
            self.text.insert(self.cursor_text_position, char);
            self.cursor_text_position += 1;
        }
        self.char_position_mapping = platform.font_context.compute_char_position_mapping(&self.font, &self.text);
    }

    pub fn is_inside(&self, x: f32, y: f32) -> bool {
        return x > self.x && x < (self.x + self.width) &&
               y > self.y && y < (self.y + self.height);
    }

    pub fn click(&mut self, x: f32, _: f32) {

        if self.select_on_first_click && !self.has_focus {  //TODO: this no longer works because focus is already set on mouse down
            self.selection_start_idx = 0;
            self.selection_end_idx = self.text.len() - 1;
            self.selection_start_x = self.x + TEXT_FIELD_OFFSET_FROM_BORDER;
            self.selection_end_x = self.x + TEXT_FIELD_OFFSET_FROM_BORDER + self.char_position_mapping.iter().last().unwrap();
            self.has_focus = true;
            return;
        }

        self.has_focus = true; //TODO: this should happen already in the mouse down

        let mut found = false;
        for (idx, x_position) in self.char_position_mapping.iter().enumerate() {
            if x_position + self.x + TEXT_FIELD_OFFSET_FROM_BORDER > x {
                self.cursor_text_position = idx;
                found = true;
                break;
            }
        }
        if !found {
            self.cursor_text_position = self.text.len();
        }

        self.clear_selection();
    }

    pub fn update_selection(&mut self, selection_rect: &Rect) {
        let min_x = selection_rect.x;
        let max_x = min_x + selection_rect.width;
        let text_start_x = self.x + TEXT_FIELD_OFFSET_FROM_BORDER;

        if (min_x > self.x && min_x < (self.x + self.width)) || (max_x > self.x && max_x < (self.x + self.width))  {

            let mut found = false;
            for (idx, x_position) in self.char_position_mapping.iter().enumerate() {
                if *x_position + text_start_x > min_x {
                    let char_offset = if idx == 0 { 0.0 } else { self.char_position_mapping[idx - 1] };
                    self.selection_start_x = text_start_x + char_offset;
                    self.selection_start_idx = idx;
                    found = true;
                    break;
                }
            }
            if !found {
                self.clear_selection();
                self.has_focus = false;
                return;
            }

            let mut found = false;
            for (idx, x_position) in self.char_position_mapping.iter().enumerate() {
                if *x_position + text_start_x > max_x {
                    self.selection_end_x = text_start_x + *x_position;
                    self.selection_end_idx = idx;
                    found = true;
                    break;
                }
            }
            if !found {
                self.selection_end_x = text_start_x + self.char_position_mapping[self.text.len() - 1];
                self.selection_end_idx = self.text.len() - 1;
            }

            self.has_focus = true;
        } else {
            self.clear_selection();
            self.has_focus = false;
        }
    }

    pub fn clear_selection(&mut self) {
        self.selection_start_x = 0.0;
        self.selection_end_x = 0.0;
        self.selection_start_idx = 0;
        self.selection_end_idx = 0;
    }

    pub fn get_selected_text(&self) -> String {
        let selection_size = self.selection_end_idx - self.selection_start_idx + 1;
        return self.text.chars().skip(self.selection_start_idx).take(selection_size).collect::<String>();
    }

    fn remove_selected_text(&mut self, platform: &Platform) {
        if self.has_selection_active() {
            for _ in self.selection_start_idx..(self.selection_end_idx+1) {
                self.text.remove(self.selection_start_idx);
            }
            self.cursor_text_position = self.selection_start_idx;
            self.char_position_mapping = platform.font_context.compute_char_position_mapping(&self.font, &self.text);
            self.clear_selection();
        }
    }

    pub fn has_selection_active(&self) -> bool {
        return self.selection_start_idx != self.selection_end_idx;
    }

    pub fn handle_keyboard_input(&mut self, platform: &Platform, input: Option<&String>, key_code: Option<KeyCode>) {
        if input.is_some() {
            self.insert_text(platform, &input.unwrap());
            return;
        }

        if key_code.is_some() {
            match key_code.unwrap() {
                KeyCode::BACKSPACE => {
                    if self.has_selection_active() {
                        self.remove_selected_text(platform);
                    } else if self.cursor_text_position > 0 {
                        self.text.remove(self.cursor_text_position - 1);  //TODO: this does not work with unicode, but we probably have many more places here that don't
                        self.cursor_text_position -= 1;
                        self.char_position_mapping = platform.font_context.compute_char_position_mapping(&self.font, &self.text);
                    }
                },
                KeyCode::LEFT => {
                    self.clear_selection();
                    if self.cursor_text_position > 0 {
                        self.cursor_text_position -= 1;
                    }
                },
                KeyCode::RETURN => {
                    //This is currently handled outside of the component (for the address bar), we need to rework this now we have more text inputs on a page possibly
                },
                KeyCode::RIGHT => {
                    self.clear_selection();
                    if self.cursor_text_position < self.text.len() {
                        self.cursor_text_position += 1;
                    }
                },
            }
        }
    }
}


const BUTTON_TEXT_OFFSET_FROM_BORDER: f32 = 5.0;

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Button {
    pub id: usize,
    pub x: f32, //NOTE: x and y are the absolute positions in the window, not content positions in the page.
    pub y: f32, //TODO: we need to make sure x and y are updated when the page is scrolled (for <input> TextFields) and disable / hide them when outside of the window
    pub width: f32,
    pub height: f32,
    #[allow(dead_code)] pub has_focus: bool,  //TODO: set in the correct cases, and use (to trigger on enter)
    pub text: String,
    pub font: Font,
}
impl Button {
    pub fn new(x: f32, y: f32, width: f32, height: f32, text: String) -> Button {
        //TODO: for now the width with not neccesarily be compatible with the text, but when we have a PLATFORM global we can fix this with the width of the text
        //TODO: it would be nicer to have the font_context (and other contexts) in some kind of global
        //      -> yes, we are going to make a lazy_static PLATFORM variable
        return Button { id: get_next_component_id(), x, y, width, height, has_focus: false, text, font: Font::default()};
    }

    pub fn render(&self, platform: &mut Platform) {
        platform.draw_square(self.x, self.y, self.width, self.height, Color::BLACK, 255);
        platform.render_text(&self.text, self.x + BUTTON_TEXT_OFFSET_FROM_BORDER, self.y + BUTTON_TEXT_OFFSET_FROM_BORDER, &self.font, Color::BLACK);
    }

    pub fn click(&mut self) {
        //We don't implement any actual behavior here for now, since the actual click is handled by the dom node. Later we could add animation here to show
        //   the button actually being pressed.
    }

    pub fn is_inside(&self, x: f32, y: f32) -> bool {
        return x > self.x && x < (self.x + self.width) &&
               y > self.y && y < (self.y + self.height);
    }

    pub fn update_position(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.x = x;
        self.y = y;
        self.width = width;
        self.height = height;
    }
}


pub struct NavigationButton {
    pub x: f32,
    pub y: f32,
    pub forward: bool, //false means its back
    pub enabled: bool, //TODO: this one is not yet set based on the history, needs to be fixed
}
impl NavigationButton {
    pub fn render(&self, platform: &mut Platform) {

        if self.forward {
            //The forward button
            let color = if self.enabled { Color::BLACK } else { Color::GRAY };
            let center_point = Position {x: self.x + 25.0, y: self.y + 10.0 };
            platform.draw_line(Position {x: self.x + 0.0, y: self.y + 10.0 }, center_point,  color);
            platform.draw_line(center_point, Position {x: self.x + 15.0, y: self.y + 20.0 }, color);
            platform.draw_line(center_point, Position {x: self.x + 15.0, y: self.y + 0.0 },  color);
        } else {
            //The back button
            let color = if self.enabled { Color::BLACK } else { Color::GRAY };
            let center_point = Position {x: self.x + 0.0, y: self.y + 10.0 };
            platform.draw_line(center_point, Position {x: self.x + 25.0, y: self.y + 10.0 }, color);
            platform.draw_line(center_point, Position {x: self.x + 10.0, y: self.y + 20.0 }, color);
            platform.draw_line(center_point, Position {x: self.x + 10.0, y: self.y + 0.0  }, color);
        }

    }
    //TODO: handle mouseover (make some mouseover background change color or something to make clearer that its a button)

    pub fn click(&mut self, x: f32, y: f32, history: &mut History) -> Option<Url> {
        //TODO: the x and y are now starting where we draw the arrow. We should make the component bigger so it covers the whole click region
        //      but then we also need to change the co-ordinates we use to draw

        //TODO: the is_inside check should live outside this component, and the click() should only be called once inside...
        let is_inside = x > (self.x - 10.0) && x < (self.x + 30.0) &&
                        y > (self.y - 10.0) && y < (self.y + 30.0);

        if is_inside && self.enabled {
            if self.forward {
                if history.list.len() > (history.position + 1) {
                    history.currently_navigating_from_history = true;
                    history.position = history.position + 1;
                    return Some(history.list.get(history.position).unwrap().clone());
                } else {
                    debug_log_warn("history button should have been disabled")
                }

            } else {
                if history.position > 0 {
                    history.currently_navigating_from_history = true;
                    history.position = history.position - 1;
                    return Some(history.list.get(history.position).unwrap().clone());
                } else {
                    debug_log_warn("history button should have been disabled")
                }

            }
        }

        return None;
    }
}


const MINIMUM_SCOLLBLOCK_HEIGHT: f32 = 25.0;

pub struct Scrollbar {
    //NOTE: for now this is only a vertical scrollbar
    //TODO: make it generic for direction, or add another component for horizontal scrolling

    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,

    pub block_height: f32,
    pub block_y: f32,

    pub content_size: f32,
    pub content_visible_height: f32,

    pub enabled: bool,
}
impl Scrollbar {
    pub fn render(&self, platform: &mut Platform) {
        platform.fill_rect(self.x, self.y, self.width, self.height, UI_BASIC_COLOR, 255);
        if self.enabled {
            platform.fill_rect(self.x, self.block_y, self.width, self.block_height, UI_BASIC_DARKER_COLOR, 255);
        }
    }

    pub fn scroll(&mut self, moved_y: f32, content_scroll_y: f32) -> f32 {
        if !self.enabled {
            return content_scroll_y;
        }

        let movable_space = self.height - self.block_height;
        let relatively_moved = moved_y / movable_space;
        let content_scroll_y_diff = (self.content_size - self.content_visible_height) * relatively_moved;
        return self.update_scroll(content_scroll_y + content_scroll_y_diff);
    }

    pub fn update_content_size(&mut self, new_content_size: f32, content_scroll_y: f32) -> f32 {
        self.content_size = new_content_size;

        self.enabled = self.content_size > self.content_visible_height;
        let relative_size_of_scroll_block = f32::min(self.content_visible_height / self.content_size, 1.0);
        self.block_height = f32::max(relative_size_of_scroll_block * self.height, MINIMUM_SCOLLBLOCK_HEIGHT);

        return self.update_scroll(content_scroll_y);
    }

    pub fn update_scroll(&mut self, content_scroll_y: f32) -> f32 {
        let new_content_scroll_y = self.clamp_scroll_position(content_scroll_y);

        let scrollblock_distance_per_page_y = (self.height - self.block_height) / (self.content_size - self.content_visible_height);
        self.block_y = scrollblock_distance_per_page_y * new_content_scroll_y + self.y;

        return new_content_scroll_y;
    }

    pub fn is_on_scrollblock(&self, x: f32, y: f32) -> bool {
        return self.x       <= x && (self.x + self.width)              >= x &&
               self.block_y <= y && (self.block_y + self.block_height) >= y;
    }

    fn clamp_scroll_position(&self, content_scroll_y: f32) -> f32 {
        if content_scroll_y < 0.0 {
            return 0.0;
        }
        let mut max_scroll_y = (self.content_size + 1.0) - self.content_visible_height;
        if max_scroll_y < 0.0 {
            max_scroll_y = 0.0;
        }
        if content_scroll_y > max_scroll_y {
            return max_scroll_y;
        }
        return content_scroll_y;
    }
}
