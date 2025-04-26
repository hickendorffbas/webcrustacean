pub mod fonts;

use image::RgbaImage;
use rusttype::{point, Scale};
use sdl2::{
    image::{self as SdlImage, Sdl2ImageContext},
    keyboard::Keycode as SdlKeycode,
    pixels::{Color as SdlColor, PixelFormatEnum},
    rect::{Point as SdlPoint, Rect as SdlRect},
    render::{BlendMode, WindowCanvas},
    Sdl,
    VideoSubsystem,
};

use crate::color::Color;
use crate::platform::fonts::{Font, FontContext};


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


pub enum KeyCode {
    BACKSPACE,
    LEFT,
    RETURN,
    RIGHT
}


pub struct Platform {
    pub sdl_context: Sdl,
    pub font_context: FontContext,

    #[cfg(test)] pub canvas: WindowCanvas,
    #[cfg(not(test))] canvas: WindowCanvas,

    video_subsystem: VideoSubsystem,

    //the image_context is not used by our code, but needs to be kept alive in order to work with images in SDL2:
    _image_context: Sdl2ImageContext,
}
impl Platform {
    pub fn present(&mut self) {
        self.canvas.present();
    }

    pub fn render_clear(&mut self, color: Color) {
        self.canvas.set_draw_color(to_sdl_color(color, 255));
        self.canvas.clear();
    }

    pub fn draw_line(&mut self, start: Position, end: Position, color: Color) {
        self.canvas.set_draw_color(to_sdl_color(color, 255));
        self.canvas.draw_line(start.to_sdl_point(), end.to_sdl_point()).expect("error drawing line");
    }

    pub fn render_text(&mut self, text: &String, x: f32, y: f32, font: &Font, color: Color) {
        if text.len() == 0 {
            return;
        }

        let rust_type_font = &self.font_context.font_data[&font.to_font_key()];

        let scale = Scale::uniform(font.size as f32);
        let v_metrics = rust_type_font.v_metrics(scale);
        let glyphs: Vec<_> = rust_type_font.layout(text, scale, point(0.0, v_metrics.ascent)).collect();

        self.enable_blending(); //TODO: what if we always have blending on? Maybe more expensive?

        //TODO: to speed this up, we would need to save the resulting bitmap including alpha somewhere
        for glyph in glyphs {
            if let Some(bounding_box) = glyph.pixel_bounding_box() {
                glyph.draw(|g_x, g_y, g_v| {

                    let absolute_x = g_x as i32 + bounding_box.min.x + x as i32;
                    let absolute_y = g_y as i32 + bounding_box.min.y + y as i32;

                    //TODO: it is probably slow to set pixels individually on the full surface, instead
                    //      of render a smaller surface first. But lets first move to openGL instead of
                    //      optimizing for SDL, the optimization might be different on openGL
                    self.set_pixel(absolute_x, absolute_y, color, (g_v * 255.0) as u8);

                });
            }
        }

        self.disable_blending();
    }

    pub fn enable_blending(&mut self) {
        self.canvas.set_blend_mode(BlendMode::Blend);
    }

    pub fn disable_blending(&mut self) {
        self.canvas.set_blend_mode(BlendMode::None);
    }

    pub fn fill_rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color, alpha: u8) {
        self.canvas.set_draw_color(to_sdl_color(color, alpha));

        let rect = SdlRect::new(x as i32, y as i32, width as u32, height as u32);
        self.canvas.fill_rect(rect).expect("error filling rect");
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, color: Color, alpha: u8) {
        self.canvas.set_draw_color(to_sdl_color(color, alpha)); //TODO: we might want to extract this out of the platform calls, and make it a platform call
                                                                //      by itself, so we don't need to call it as much...
        self.canvas.draw_point(SdlPoint::new(x, y)).expect("error drawing point");
    }

    pub fn draw_square(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color, alpha: u8) {
        self.canvas.set_draw_color(to_sdl_color(color, alpha));

        let rect = SdlRect::new(x as i32, y as i32, width as u32, height as u32);
        self.canvas.draw_rect(rect).expect("error drawing square");
    }

    pub fn render_image(&mut self, image: &RgbaImage, x: f32, y: f32) {
        let texture_creator = self.canvas.texture_creator(); //TODO: reuse the texture creator for the canvas by storing it on the context?

        let raw_pixels = image.clone().into_raw();
        let mut texture = texture_creator.create_texture_streaming(PixelFormatEnum::ABGR8888, image.width(), image.height()).unwrap();

        texture.update(None, &raw_pixels, (image.width() * 4) as usize).unwrap();

        self.canvas.copy(&texture, None, Some(SdlRect::new(x as i32, y as i32, image.width(), image.height()))).expect("error rendering image");
    }
    pub fn enable_text_input(&self) {
        self.video_subsystem.text_input().start();
    }
    pub fn disable_text_input(&self) {
        self.video_subsystem.text_input().stop();
    }
    pub fn convert_key_code(&self, keycode: &SdlKeycode) -> Option<KeyCode> {
        return match keycode.name().as_str() {
            "Backspace" => Some(KeyCode::BACKSPACE),
            "Left" => Some(KeyCode::LEFT),
            "Return" => Some(KeyCode::RETURN),
            "Right" => Some(KeyCode::RIGHT),
            _ => None,
        }
    }
}


pub fn to_sdl_color(color: Color, alpha: u8) -> SdlColor {
    return SdlColor::RGBA(color.r, color.g, color.b, alpha);
}


pub fn init_platform(sdl_context: Sdl, screen_width: f32, screen_height: f32, headless: bool) -> Result<Platform, String> {
    let video_subsystem = sdl_context.video()
        .expect("Could not get the video subsystem");

    let image_context = SdlImage::init(SdlImage::InitFlag::PNG | SdlImage::InitFlag::JPG)?;

    let mut window_builder = video_subsystem.window("Webcrustacean", screen_width as u32, screen_height as u32);
    window_builder.position_centered().resizable();

    if headless {
        window_builder.hidden();
    }

    let window = window_builder.build().expect("could not initialize video subsystem");

    let canvas = window.into_canvas().build()
        .expect("could not make a canvas");

    return Result::Ok(Platform {
        canvas,
        sdl_context,
        font_context: FontContext::new(),
        video_subsystem,
        _image_context: image_context,
    });
}
