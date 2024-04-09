use image::DynamicImage;

use sdl2::{
    image::{self as SdlImage, Sdl2ImageContext},
    keyboard::Keycode as SdlKeycode,
    pixels::{Color as SdlColor, PixelFormatEnum},
    rect::{Point as SdlPoint, Rect as SdlRect},
    render::{BlendMode, TextureAccess, WindowCanvas},
    Sdl,
    VideoSubsystem,
};

use crate::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::color::Color;
use crate::fonts::{self, Font, FontContext};


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


pub struct Platform<'a> {
    pub sdl_context: Sdl,
    pub font_context: FontContext<'a>,

    canvas: WindowCanvas,
    video_subsystem: VideoSubsystem,

    //the image_context is not used by our code, but needs to be kept alive in order to work with images in SDL2:
    _image_context: Sdl2ImageContext,
}
impl Platform<'_> {
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
        //TODO: this method should just move to the font context? YES
        fonts::render_text(self, text, color, font, x, y);
    }

    pub fn get_text_dimension(&mut self, text: &String, font: &Font) -> (f32, f32) {
        //TODO: this method should just move to the font context? YES
        return fonts::get_text_dimension(&self.font_context, text, font);
    }

    pub fn get_text_dimension_str(&mut self, text: &str, font: &Font) -> (f32, f32) {
        //TODO: this method should just move to the font context? YES
        return fonts::get_text_dimension_str(&self.font_context, text, font);
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

    pub fn render_image(&mut self, image: &DynamicImage, x: f32, y: f32) {
        let texture_creator = self.canvas.texture_creator(); //TODO: reuse the texture creator for the canvas by storing it on the context?

        let mut texture = texture_creator.create_texture(find_pixel_format(image), TextureAccess::Target, image.width(), image.height()).unwrap();

        let bytes_per_pixel = image.color().bytes_per_pixel();
        texture.update(None, image.as_bytes(), image.width() as usize * bytes_per_pixel as usize).unwrap();

        //self.canvas.set_blend_mode(BlendMode::Blend); //TODO: this does not work, but we need to fix blending somehow (for png alpha)

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


pub fn find_pixel_format(image: &DynamicImage) -> PixelFormatEnum {
    match image {
        DynamicImage::ImageLuma8(_) => todo!(),
        DynamicImage::ImageLumaA8(_) => todo!(),
        DynamicImage::ImageRgb8(_) => PixelFormatEnum::RGB24,
        DynamicImage::ImageRgba8(_) => PixelFormatEnum::ABGR8888,
        DynamicImage::ImageLuma16(_) => todo!(),
        DynamicImage::ImageLumaA16(_) => todo!(),
        DynamicImage::ImageRgb16(_) => todo!(),
        DynamicImage::ImageRgba16(_) => todo!(),
        DynamicImage::ImageRgb32F(_) => todo!(),
        DynamicImage::ImageRgba32F(_) => todo!(),
        _ => panic!("unexpected pixel format"), //TODO: what case is this describing?
    }
}


pub fn to_sdl_color(color: Color, alpha: u8) -> SdlColor {
    return SdlColor::RGBA(color.r, color.g, color.b, alpha);
}


pub fn init_platform<'a>(sdl_context: Sdl) -> Result<Platform<'a>, String> {
    let video_subsystem = sdl_context.video()
        .expect("Could not get the video subsystem");

    let image_context = SdlImage::init(SdlImage::InitFlag::PNG | SdlImage::InitFlag::JPG)?;

    let window = video_subsystem.window("Webcrustacean", SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");

    let canvas = window.into_canvas().build()
        .expect("could not make a canvas");

    let mut font_context = FontContext::empty();
    fonts::setup_font_context(&mut font_context);

    return Result::Ok(Platform {
        canvas,
        sdl_context,
        font_context: font_context,
        video_subsystem,
        _image_context: image_context,
    });
}
