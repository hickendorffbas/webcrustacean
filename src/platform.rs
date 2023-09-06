use std::collections::HashMap;

use image::DynamicImage;

use sdl2::{
    pixels::PixelFormatEnum,
    rect::{Point as SdlPoint, Rect},
    render::{TextureQuery, TextureAccess},
    render::WindowCanvas,
    Sdl,
    ttf::Sdl2TtfContext, VideoSubsystem,
};

use crate::color::Color;
use crate::{SCREEN_WIDTH, SCREEN_HEIGHT};
use crate::fonts::{FontCache, Font};


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone, Copy)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}
impl Position {
    pub fn to_sdl_point(&self) -> SdlPoint {
        return SdlPoint::new(self.x as i32, self.y as i32);
    }
}


pub struct Platform<'a> {
    //TODO: would be nice to not have all of these public eventually
    pub sdl_context: Sdl,
    pub font_cache: FontCache<'a, 'a>,
    pub canvas: WindowCanvas,
    pub video_subsystem: VideoSubsystem,
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

    pub fn render_text(&mut self, text: &String, x: f32, y: f32, font: &Font, color: Color) {
        if text.len() == 0 {
            return;
        }

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

    pub fn fill_rect(&mut self, x: f32, y: f32, width: u32, height: u32, color: Color) {
        self.canvas.set_draw_color(color.to_sdl_color());

        let rect = Rect::new(x as i32, y as i32, width, height);
        self.canvas.fill_rect(rect).expect("error drawing rect");
    }

    pub fn render_image(&mut self, image: &DynamicImage, x: f32, y: f32) {

        let texture_creator = self.canvas.texture_creator(); //TODO: reuse the texture creator for the canvas by storing it on the context?

        //TODO: the pixel format below is not always correct, derive it from the image
        let mut texture = texture_creator.create_texture(PixelFormatEnum::RGB24, TextureAccess::Target, image.width(), image.height()).unwrap();

        let bytes_per_pixel = 3; //TODO: not always correct, can we derive this from the img object? Probably based on the pixel type

        texture.update(None, image.as_bytes(), image.width() as usize * bytes_per_pixel).unwrap();

        self.canvas.copy(&texture, None, Some(Rect::new(x as i32, y as i32, image.width(), image.height()))).expect("error rendering image");
    }
    pub fn enable_text_input(&self) {
        self.video_subsystem.text_input().start();
    }
    pub fn disable_text_input(&self) {
        self.video_subsystem.text_input().stop();
    }
}


pub fn init_platform<'a>(sdl_context: Sdl, ttf_context: &Sdl2TtfContext) -> Result<Platform, String> {
    let video_subsystem = sdl_context.video()
        .expect("Could not get the video subsystem");

    let window = video_subsystem.window("BBrowser", SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");

    let canvas = window.into_canvas().build()
        .expect("could not make a canvas");

    return Result::Ok(Platform {
        canvas,
        sdl_context,
        font_cache: FontCache {ttf_context: ttf_context, mapping: HashMap::new()},
        video_subsystem,
    });
}
