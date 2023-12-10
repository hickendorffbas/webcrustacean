use crate::color::Color;
use crate::debug::debug_log_warn;
use crate::fonts::Font;
use crate::network::url::Url;
use crate::platform::{
    KeyCode,
    Platform,
    Position
};
use crate::ui::History;


pub struct TextField {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,

    pub has_focus: bool,
    pub cursor_visible: bool,
    pub cursor_text_position: usize,
    pub text: String,

    pub font: Font,
    pub char_position_mapping: Vec<f32>,
}
impl TextField {
    pub fn render(&self, platform: &mut Platform) {
        let text_offset_from_border = 5.0;

        platform.draw_square(self.x, self.y, self.width, self.height, Color::BLACK);
        platform.render_text(&self.text, self.x + text_offset_from_border, self.y + text_offset_from_border, &self.font, Color::BLACK);

        if self.cursor_visible && self.has_focus {

            let cursor_position = self.char_position_mapping[self.cursor_text_position] + self.x + text_offset_from_border;
            let cursor_top_bottom_margin = 2.0;
            let cursor_bottom_pos = (self.y + self.height) - cursor_top_bottom_margin;
            platform.draw_line(Position { x: cursor_position, y: self.y + cursor_top_bottom_margin},
                               Position { x: cursor_position, y: cursor_bottom_pos },
                               Color::BLACK);
        }
    }
    pub fn set_text(&mut self, platform: &mut Platform, text: String) { //TODO: use this everywhere...
        self.text = text;

        if self.cursor_text_position > self.text.len() {
            self.cursor_text_position = self.text.len();
        }

        self.char_position_mapping = compute_char_position_mapping(platform, &self.font, &self.text);
    }
    pub fn click(&mut self, x: f32, y: f32)  {
        let is_inside = x > self.x && x < (self.x + self.width) &&
                        y > self.y && y < (self.y + self.height);
        self.has_focus = is_inside;

        if is_inside {
            let mut found = false;
            for (idx, x_position) in self.char_position_mapping.iter().enumerate() {
                if x_position + self.x > x {
                    self.cursor_text_position = idx - 1;
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
        let mut text_changed = false;

        if input.is_some() {
            self.text.insert_str(self.cursor_text_position, input.unwrap());
            self.cursor_text_position += 1;
            text_changed = true;
        }

        if key_code.is_some() {
            match key_code.unwrap() {
                KeyCode::BACKSPACE => {
                    if self.cursor_text_position > 0 {
                        self.text.remove(self.cursor_text_position - 1);
                        self.cursor_text_position -= 1;
                    }
                    text_changed = true;
                },
                KeyCode::LEFT => {
                    if self.cursor_text_position > 0 {
                        self.cursor_text_position -= 1;
                    }
                },
                KeyCode::RETURN => {
                    //This is currently handled outside of the component
                },
                KeyCode::RIGHT => {
                    if self.cursor_text_position < self.text.len() {
                        self.cursor_text_position += 1;
                    }
                },
            }
        }

        if text_changed {
            self.char_position_mapping = compute_char_position_mapping(platform, &self.font, &self.text);
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


//TODO: this should not be public, but we use it in layout now as well, so we should move this to a general place, maybe platform?
pub fn compute_char_position_mapping(platform: &mut Platform, font: &Font, text: &String) -> Vec<f32> {
    //TODO: we take a very slow approach here. Not sure if we can do this faster.

    let mut char_position_mapping = Vec::new();
    char_position_mapping.push(0.0);

    for (idx, _) in text.char_indices() { //taking indices, because we need to iterate chars, but index by byte. We do want to use the slice, to
                                          //prevent allocating a new string for each iteration of the loop.
        let (x_pos, _) = platform.get_text_dimension_str(&text[0..idx], font);
        char_position_mapping.push(x_pos);
    }

    return char_position_mapping;
}
