use sdl2::pixels::Color as SdlColor;


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8
}
impl Color {
    pub fn to_sdl_color(&self) -> SdlColor {
        //TODO: this function beloings in the platform layer, not here
        return SdlColor::RGB(self.r, self.g, self.b)
    }
    pub const fn new(p_r: u8, p_g: u8, p_b: u8) -> Color { Color { r: p_r, g: p_g, b: p_b } }

    pub fn from_string(color_name: &String) -> Option<Color> {  //TODO: maby use result here, instead of option?

        //TODO: check here if we have a #00ff00 like color spec, and parse that...
        //      although maybe that shouldn't happen here, and we should just always deal with rgb values only for the platform...

        match color_name.as_str() {
            "black" => Some(Color::BLACK),
            "blue" => Some(Color::BLUE),
            "red" => Some(Color::RED),
            "green" => Some(Color::GREEN),
            "white" => Some(Color::WHITE),
            _ => None
        }
    }

    pub const BLACK: Color = Color::new(0, 0, 0);
    pub const BLUE: Color = Color::new(0, 0, 255);
    pub const GREEN: Color = Color::new(0, 255, 0);
    pub const RED: Color = Color::new(255, 0, 0);
    pub const WHITE: Color = Color::new(255, 255, 255);
}
