use sdl2::rect::{Point as SdlPoint, Rect};
use sdl2::render::{TextureQuery, WindowCanvas};
use sdl2::pixels::Color as SdlColor;
use sdl2::ttf::Font as SdlFont;


//TODO: eventually I would to like to seperate platform (SDL) and the renderer


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone, Copy)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}
impl Position {
    fn to_sdl_point(&self) -> SdlPoint { //TODO: maybe want to put all SDL stuff in a separate place, and therefore not in this impl?
        return SdlPoint::new(self.x as i32, self.y as i32);
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Dimension {
    pub width: u32,
    pub height: u32,
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8
}
impl Color {
    pub fn to_sdl_color(&self) -> SdlColor {
        return SdlColor::RGB(self.r, self.g, self.b)
    }
    pub const fn new(p_r: u8, p_g: u8, p_b: u8) -> Color { Color { r: p_r, g: p_g, b: p_b } }

    pub const BLACK: Color = Color::new(0, 0, 0);
    pub const BLUE: Color = Color::new(0, 0, 255);
    #[allow(dead_code)] //TODO: remove when used
    pub const GREEN: Color = Color::new(0, 255, 0);
    #[allow(dead_code)] //TODO: remove when used
    pub const RED: Color = Color::new(255, 0, 0);
    pub const WHITE: Color = Color::new(255, 255, 255);
}


pub fn clear(canvas: &mut WindowCanvas, color: Color) {
    canvas.set_draw_color(color.to_sdl_color());
    canvas.clear();
}


#[allow(dead_code)] //TODO: eventually use, or remove
pub fn draw_line(canvas: &mut WindowCanvas, start: Position, end: Position, color: Color) {
    canvas.set_draw_color(color.to_sdl_color());
    canvas.draw_line(start.to_sdl_point(), end.to_sdl_point()).expect("error drawing line");
}


pub fn render_text(canvas: &mut WindowCanvas, text: &String, x: u32, y: u32, font: &SdlFont, color: SdlColor) {
    let sdl_surface = font
        .render(text)
        .blended(color)
        .expect("error while rendering text");

    let texture_creator = canvas.texture_creator();

    let texture = texture_creator
        .create_texture_from_surface(&sdl_surface)
        .expect("error while building surface");


    let TextureQuery { width, height, .. } = texture.query();

    let target = Rect::new(x as i32, y as i32, width, height);

    canvas.copy(&texture, None, Some(target))
        .expect("copying texture in canvas failed!");
}


pub fn get_text_dimension(text: &str, font: &SdlFont) -> Dimension {
    let result = font.size_of(text);
    if result.is_ok() {
        let (width, height) = result.ok().unwrap();
        return Dimension { width, height };
    } else {
        panic!("{:?}", result.err().unwrap()); //TODO: don't think this is a good way of reporting the error
    }
}
