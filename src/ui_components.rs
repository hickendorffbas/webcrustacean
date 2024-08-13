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
use crate::ui::{History, UIState};


const TEXT_FIELD_OFFSET_FROM_BORDER: f32 = 5.0;
const CURSOR_BLINK_SPEED_MILLIS: u32 = 500;

pub struct TextField {
    //TODO: it would be nice to have a distinction between properties of the component, and state (for example, select_on_first_click is a property,
    //      while selection_start_x is state. Maybe make a constructor?)
    pub x: f32,
    pub y: f32,
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

    pub fn set_text(&mut self, platform: &mut Platform, text: String) { //TODO: use this everywhere...
        self.clear_selection();
        self.text = text;

        if self.cursor_text_position > self.text.len() {
            self.cursor_text_position = self.text.len();
        }

        self.char_position_mapping = platform.font_context.compute_char_position_mapping(&self.font, &self.text);
    }

    pub fn insert_text(&mut self, platform: &mut Platform, text: &String) {
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

    pub fn click(&mut self, x: f32, y: f32)  {
        if !self.is_inside(x, y) {
            self.has_focus = false;
            return;
        }

        if self.select_on_first_click && !self.has_focus {
            self.selection_start_idx = 0;
            self.selection_end_idx = self.text.len() - 1;
            self.selection_start_x = self.x + TEXT_FIELD_OFFSET_FROM_BORDER;
            self.selection_end_x = self.x + TEXT_FIELD_OFFSET_FROM_BORDER + self.char_position_mapping.iter().last().unwrap();
            self.has_focus = true;
            return;
        }

        self.has_focus = true;

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
                    self.selection_start_idx = if idx == 0 { 0 } else { idx - 1 };
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

    fn remove_selected_text(&mut self, platform: &mut Platform) {
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

    pub fn handle_keyboard_input(&mut self, platform: &mut Platform, input: Option<&String>, key_code: Option<KeyCode>) {
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
                    //This is currently handled outside of the component
                    //TODO: (but shouldn't I think)
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
