use crate::{platform::{Platform, Position, KeyCode}, fonts::Font, color::Color};

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


fn compute_char_position_mapping(platform: &mut Platform, font: &Font, text: &String) -> Vec<f32> {
    //TODO: we take a very slow approach here. Not sure if we can do this faster.

    let mut char_position_mapping = Vec::new();
    char_position_mapping.push(0.0);

    for i in 1..text.len()+1 {
        let (x_pos, _) = platform.get_text_dimension_str(&text[0..i], font);
        char_position_mapping.push(x_pos);
    }

    return char_position_mapping;
}
