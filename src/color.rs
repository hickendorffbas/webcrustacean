
#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8
}
impl Color {
    pub const fn new(p_r: u8, p_g: u8, p_b: u8) -> Color { Color { r: p_r, g: p_g, b: p_b } }

    pub fn from_string(color_name: &String) -> Option<Color> {

        let first_char = color_name.chars().next();
        if first_char.is_none() {
            return None;
        }

        if first_char.unwrap() == '#' {
            let r = u8::from_str_radix(&color_name[1..3], 16).ok();
            let g = u8::from_str_radix(&color_name[3..5], 16).ok();
            let b = u8::from_str_radix(&color_name[5..7], 16).ok();

            if r.is_none() || g.is_none() || b.is_none() {
                return None;
            }

            return Some(Color {r: r.unwrap(), g: g.unwrap(), b: b.unwrap() })
        }

        //TODO: I still need to support hsl and rgb color values (as specified in html / css)

        match color_name.as_str() {
            "aqua" => Some(Color::new(0, 255, 255)),
            "black" => Some(Color::BLACK),
            "blue" => Some(Color::new(0, 0, 255)),
            "fuchsia" => Some(Color::new(255, 0, 255)),
            "gray" => Some(Color::new(128, 128, 128)),
            "green" => Some(Color::new(0, 255, 0)),
            "lime" => Some(Color::new(0, 255, 0)),
            "maroon" => Some(Color::new(128, 0, 0)),
            "navy" => Some(Color::new(0, 0, 128)),
            "olive" => Some(Color::new(128, 128, 0)),
            "purple" => Some(Color::new(128, 0, 128)),
            "red" => Some(Color::new(255, 0, 0)),
            "silver" => Some(Color::new(192, 192, 192)),
            "teal" => Some(Color::new(0, 128, 128)),
            "white" => Some(Color::WHITE),
            "yellow" => Some(Color::new(255, 255, 0)),
            _ => None
        }
    }

    //Below we only define Colors we use in other parts of the code in a hardcoded way:
    pub const BLACK: Color = Color::new(0, 0, 0);
    pub const WHITE: Color = Color::new(255, 255, 255);
}
