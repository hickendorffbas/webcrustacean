use crate::color::Color;
use crate::debug::debug_log_warn;
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
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,

    pub has_focus: bool,
    pub cursor_text_position: usize, // this position means the string index it is _before_, so starts at 0, and is max string.len()
    pub text: String,

    pub font: Font,
    pub char_position_mapping: Vec<f32>,
}
impl TextField {
    pub fn render(&self, ui_state: &UIState, platform: &mut Platform) {
        platform.draw_square(self.x, self.y, self.width, self.height, Color::BLACK, 255);
        platform.render_text(&self.text, self.x + TEXT_FIELD_OFFSET_FROM_BORDER, self.y + TEXT_FIELD_OFFSET_FROM_BORDER, &self.font, Color::BLACK);

        if self.has_focus {

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
        self.text = text;

        if self.cursor_text_position > self.text.len() {
            self.cursor_text_position = self.text.len();
        }

        self.char_position_mapping = platform.font_context.compute_char_position_mapping(&self.font, &self.text);
    }

    pub fn insert_text(&mut self, platform: &mut Platform, text: &String) {
        for char in text.chars() {
            self.text.insert(self.cursor_text_position, char);
            self.cursor_text_position += 1;
        }
        self.char_position_mapping = platform.font_context.compute_char_position_mapping(&self.font, &self.text);
    }

    pub fn click(&mut self, x: f32, y: f32)  {
        let is_inside = x > self.x && x < (self.x + self.width) &&
                        y > self.y && y < (self.y + self.height);
        self.has_focus = is_inside;

        if is_inside {
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
        }
    }

    pub fn handle_keyboard_input(&mut self, platform: &mut Platform, input: Option<&String>, key_code: Option<KeyCode>) {
        if input.is_some() {
            self.insert_text(platform, &input.unwrap());
            return;
        }

        if key_code.is_some() {
            match key_code.unwrap() {
                KeyCode::BACKSPACE => {
                    if self.cursor_text_position > 0 {
                        self.text.remove(self.cursor_text_position - 1);  //TODO: this does not work with unicode, but we probably have many more places here that don't
                        self.cursor_text_position -= 1;
                        self.char_position_mapping = platform.font_context.compute_char_position_mapping(&self.font, &self.text);
                    }
                },
                KeyCode::LEFT => {
                    if self.cursor_text_position > 0 {
                        self.cursor_text_position -= 1;
                    }
                },
                KeyCode::RETURN => {
                    //This is currently handled outside of the component
                    //TODO: (but shouldn't I think)
                },
                KeyCode::RIGHT => {
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
