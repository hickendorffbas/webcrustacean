use std::collections::HashMap;

use sdl2::rect::{Point as SdlPoint, Rect};
use sdl2::render::TextureQuery;
use sdl2::{
    render::WindowCanvas,
    Sdl,
    ttf::Sdl2TtfContext,
};

use crate::color::Color;
use crate::{SCREEN_WIDTH, SCREEN_HEIGHT};
use crate::fonts::{FontCache, Font};


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone, Copy)]
pub struct Position {  //TODO: this is more general than just the renderer, but maybe also not needed if we start using LayoutBox
    pub x: u32,
    pub y: u32,
}
impl Position {
    //TODO: this method belongs in the platform!
    pub fn to_sdl_point(&self) -> SdlPoint { //TODO: maybe want to put all SDL stuff in a separate place, and therefore not in this impl?
        return SdlPoint::new(self.x as i32, self.y as i32);
    }
}


pub struct Platform<'a> {
    pub sdl_context: Sdl,
    pub font_cache: FontCache<'a, 'a>,
    canvas: WindowCanvas,
}
impl Platform<'_> {
    pub fn present(&mut self) {
        self.canvas.present();
    }
    pub fn render_clear(&mut self, color: Color) {
        self.canvas.set_draw_color(color.to_sdl_color());
        self.canvas.clear();
    }
    pub fn draw_line(&mut self, start: Position, end: Position, color: Color) {
        self.canvas.set_draw_color(color.to_sdl_color());
        self.canvas.draw_line(start.to_sdl_point(), end.to_sdl_point()).expect("error drawing line");
    }
    pub fn render_text(&mut self, text: &String, x: u32, y: u32, font: &Font, color: Color) {
        let sdl_font = self.font_cache.get_font(font);

        let sdl_surface = sdl_font
            .render(text)
            .blended(color.to_sdl_color())
            .expect("error while rendering text");
    
        let texture_creator = self.canvas.texture_creator(); //TODO: I don't think I need to create this every time, we can probably keep it on the struct
        let texture = texture_creator
            .create_texture_from_surface(&sdl_surface)
            .expect("error while building surface");
    
        let TextureQuery { width, height, .. } = texture.query();
        let target = Rect::new(x as i32, y as i32, width, height);
    
        self.canvas.copy(&texture, None, Some(target))
            .expect("copying texture in canvas failed!");
    }
    pub fn get_text_dimension(&mut self, text: &String, font: &Font) -> (f32, f32) {
        let sdl_font = self.font_cache.get_font(font);
        let result = sdl_font.size_of(text);
        let (width, height) = result.expect("error measuring size of text");
        return (width as f32, height as f32);
    }    
}


pub fn init_platform<'a>(sdl_context: Sdl, ttf_context: &Sdl2TtfContext) -> Result<Platform, String> {
    let video_subsystem = sdl_context.video()
        .expect("Could not get the video subsystem");

    let window = video_subsystem.window("BBrowser", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");

    let canvas = window.into_canvas().build()
        .expect("could not make a canvas");

    return Result::Ok(Platform {
        canvas,
        sdl_context,
        font_cache: FontCache {ttf_context: ttf_context, mapping: HashMap::new()},
    });
}

