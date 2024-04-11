pub mod fonts;

use image::DynamicImage;

use rusttype::{point, Scale};
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

    canvas: WindowCanvas,
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


pub fn init_platform(sdl_context: Sdl) -> Result<Platform, String> {
    let video_subsystem = sdl_context.video()
        .expect("Could not get the video subsystem");

    let image_context = SdlImage::init(SdlImage::InitFlag::PNG | SdlImage::InitFlag::JPG)?;

    let window = video_subsystem.window("Webcrustacean", SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");

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
