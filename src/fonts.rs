use std::collections::HashMap;

use rusttype::{point, Font as RustTypeFont, Scale};


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
    pub fn to_font_key(&self) -> FontKey {
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
    pub fn new() -> FontContext<'static> {

        let mut font_context = FontContext { font_data: HashMap::new() };

        //TODO: load the other font variants (bold, italic etc.)
        let font = RustTypeFont::try_from_bytes(&FONT_DATA).expect("Failure loading font data");
        font_context.font_data.insert(Font::default().to_font_key(), font);

        return font_context;
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Eq, PartialEq, Hash, Clone)]
pub enum FontFace {
    //TODO: we want bold and italic to be seperate fonts here (because they are loaded from different files)
    //      but it would be nice if the enum items then have properties with those flags (bool)
    TimesNewRomanRegular,
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
