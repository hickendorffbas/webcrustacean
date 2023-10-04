use crate::{platform::{Platform, Position}, fonts::Font, color::Color};

pub struct TextField {
    //TODO: this should contain something about font, font-size, margin's etc.

    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,

    pub has_focus: bool,
    pub cursor_visible: bool,
    pub cursor_text_position: usize,
    pub text: String,
}
impl TextField {
    pub fn render(&self, platform: &mut Platform) {
        let font = Font::new(false, 18);
        let text_offset_from_border = 5.0;

        platform.draw_square(self.x, self.y, self.width, self.height, Color::BLACK);
        platform.render_text(&self.text, self.x + text_offset_from_border, self.y + text_offset_from_border, &font, Color::BLACK);

        if self.cursor_visible && self.has_focus {

            //TODO: cache on the struct, and update when changing text (only change text via method on struct):
            let char_position_mapping = compute_char_position_mapping(platform, &font, &self.text);

            let cursor_position = char_position_mapping[self.cursor_text_position] + self.x + text_offset_from_border;
            let cursor_top_bottom_margin = 2.0;
            let cursor_bottom_pos = (self.y + self.height) - cursor_top_bottom_margin;
            platform.draw_line(Position { x: cursor_position, y: self.y + cursor_top_bottom_margin},
                               Position { x: cursor_position, y: cursor_bottom_pos },
                               Color::BLACK);
        }
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
