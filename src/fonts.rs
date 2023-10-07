use std::collections::HashMap;

use crate::FONT_PATH;
use sdl2::ttf::{Font as SdlFont, Sdl2TtfContext};


#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Font {
    bold: bool,
    underline: bool,
    font_size: u16
}
impl Font {
    pub fn new(bold: bool, underline: bool, font_size: u16) -> Font {
        return Font {bold, underline, font_size};
    }
}


pub struct FontCache<'ttf_module, 'rwops> {
    pub ttf_context: &'ttf_module Sdl2TtfContext,
    pub mapping: HashMap<Font, SdlFont<'ttf_module, 'rwops>>
}
impl<'ttf_module, 'rwops> FontCache<'ttf_module, 'rwops> {
    pub fn get_font(&mut self, font: &Font) -> &SdlFont<'ttf_module, 'rwops> {

        if !self.mapping.contains_key(font) {
            let new_font = build_font(&self.ttf_context, font);
            self.mapping.insert(font.clone(), new_font);
        }
    
        return self.mapping.get(font).unwrap();
    }
}

fn build_font<'ttf_module, 'rwops>(ttf_context: &'ttf_module Sdl2TtfContext, font: &Font) -> SdlFont<'ttf_module, 'rwops> {
    let mut sdl_font : SdlFont = ttf_context.load_font(FONT_PATH, font.font_size)
        .expect("could not load font");

    if font.bold {
        sdl_font.set_style(sdl2::ttf::FontStyle::BOLD);
    }
    if font.underline {
        sdl_font.set_style(sdl2::ttf::FontStyle::UNDERLINE);
    }

    return sdl_font;
}
