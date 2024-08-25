
#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone, Copy, PartialEq)]
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

            if color_name.len() == 4 {
                let r = u8::from_str_radix(&color_name[1..2], 16).ok();
                let g = u8::from_str_radix(&color_name[2..3], 16).ok();
                let b = u8::from_str_radix(&color_name[3..4], 16).ok();

                if r.is_none() || g.is_none() || b.is_none() {
                    return None;
                }

                let r = r.unwrap() + (16 * r.unwrap());
                let g = g.unwrap() + (16 * g.unwrap());
                let b = b.unwrap() + (16 * b.unwrap());

                return Some(Color {r, g, b})
            }

            if color_name.len() == 7 {
                let r = u8::from_str_radix(&color_name[1..3], 16).ok();
                let g = u8::from_str_radix(&color_name[3..5], 16).ok();
                let b = u8::from_str_radix(&color_name[5..7], 16).ok();

                if r.is_none() || g.is_none() || b.is_none() {
                    return None;
                }

                return Some(Color {r: r.unwrap(), g: g.unwrap(), b: b.unwrap() })
            }

            return None;
        }

        //TODO: I still need to support hsl and rgb color values (as specified in html / css)

        return match color_name.as_str() {
            "aqua" => Some(Color::new(0, 255, 255)),
            "black" => Some(Color::BLACK),
            "blue" => Some(Color::new(0, 0, 255)),
            "fuchsia" => Some(Color::new(255, 0, 255)),
            "gray" => Some(Color::GRAY),
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
    pub const GRAY: Color = Color::new(128, 128, 128);
    pub const WHITE: Color = Color::new(255, 255, 255);
    pub const DEFAULT_SELECTION_COLOR: Color = Color::new(180, 213, 255);  //TODO: maybe belongs in the ui module? we have other colors there as well...
}
