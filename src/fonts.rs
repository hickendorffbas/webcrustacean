use std::collections::HashMap;

use rusttype::{point, Font as RustTypeFont, Scale};

use crate::{color::Color, platform::Platform};



//TODO: all stuff of this file belongs in platform, make platform a folder instead of a file



//TODO: we need to add the bold, italic etc. versions and thefore name this more specific
static FONT_DATA: [u8; include_bytes!("../ubuntu_fonts/Ubuntu-Regular.ttf").len()] =
    *include_bytes!("../ubuntu_fonts/Ubuntu-Regular.ttf");


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Eq, PartialEq, Hash, Clone)]
pub struct Font {
    pub face: FontFace,
    pub bold: bool,
    pub italic: bool,
    pub size: u16
}
impl Font {
    pub fn default() -> Font {
        return Font { face: FontFace::TimesNewRomanRegular, bold: false, italic: false, size: 18 };
    }
    fn to_font_key(&self) -> FontKey {
        return FontKey { face: self.face.clone(), bold: self.bold, italic: self.italic };
    }
}


#[derive(Eq, PartialEq, Hash)]
pub struct FontKey {
    face: FontFace,
    bold: bool,
    italic: bool,
}


pub struct FontContext<'a> {
    pub font_data: HashMap<FontKey, RustTypeFont<'a>>,
}
impl FontContext<'static>  {
    pub fn empty() -> FontContext<'static> {
        return FontContext { font_data: HashMap::new() };
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Eq, PartialEq, Hash, Clone)]
pub enum FontFace {
    //TODO: we want bold and italic to be seperate fonts here (because they are loaded from different files)
    //      but it would be nice if the enum items then have properties with those flags (bool)
    TimesNewRomanRegular,
}


pub fn setup_font_context(font_context: &mut FontContext){
    //TODO: load the other font variants (bold, italic etc.)
    let font = RustTypeFont::try_from_bytes(&FONT_DATA).expect("Failure loading font data");
    font_context.font_data.insert(Font::default().to_font_key(), font);
}


pub fn render_text(platform: &mut Platform, text: &str, color: Color, font: &Font, x: f32, y: f32) {

    if text.len() == 0 {
        return;
    }

    let rust_type_font = &platform.font_context.font_data[&font.to_font_key()];

    let scale = Scale::uniform(font.size as f32);
    let v_metrics = rust_type_font.v_metrics(scale);
    let glyphs: Vec<_> = rust_type_font.layout(text, scale, point(0.0, v_metrics.ascent)).collect();

    platform.enable_blending(); //TODO: what if we always have blending on? Maybe more expensive?

    //TODO: to speed this up, we would need to save the resulting bitmap including alpha somewhere
    for glyph in glyphs {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            glyph.draw(|g_x, g_y, g_v| {

                let absolute_x = g_x as i32 + bounding_box.min.x + x as i32;
                let absolute_y = g_y as i32 + bounding_box.min.y + y as i32;

                //TODO: it is probably slow to set pixels individually on the full surface, instead
                //      of render a smaller surface first. But lets first move to openGL instead of
                //      optimizing for SDL, the optimization might be different on openGL
                platform.set_pixel(absolute_x, absolute_y, color, (g_v * 255.0) as u8);

            });
        }
    }

    platform.disable_blending();

}


pub fn get_text_dimension(font_context: &FontContext, text: &String, font: &Font) -> (f32, f32) {
    return get_text_dimension_str(font_context, text.as_str(), font);
}


pub fn get_text_dimension_str(font_context: &FontContext, text: &str, font: &Font) -> (f32, f32) {
    if text == "" {
        return (0.0, 0.0);
    }

    let rust_type_font = &font_context.font_data[&font.to_font_key()];

    let scale = Scale::uniform(font.size as f32);
    let v_metrics = rust_type_font.v_metrics(scale);

    let glyphs_height = (v_metrics.ascent - v_metrics.descent + v_metrics.line_gap).ceil();
    let glyphs_width = rust_type_font.layout(text, scale, point(0.0, 0.0)).last()
            .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
            .unwrap_or(0.0);

    return (glyphs_width, glyphs_height);
}
