use std::collections::HashMap;

use rusttype::{
    Font as RustTypeFont,
    Scale,
};


static UNBUNTU_FONT_REGULAR: [u8; include_bytes!("../../ubuntu_fonts/Ubuntu-Regular.ttf").len()] =
    *include_bytes!("../../ubuntu_fonts/Ubuntu-Regular.ttf");
static UNBUNTU_FONT_BOLD: [u8; include_bytes!("../../ubuntu_fonts/Ubuntu-Bold.ttf").len()] =
    *include_bytes!("../../ubuntu_fonts/Ubuntu-Bold.ttf");
static UNBUNTU_FONT_ITALIC: [u8; include_bytes!("../../ubuntu_fonts/Ubuntu-Italic.ttf").len()] =
    *include_bytes!("../../ubuntu_fonts/Ubuntu-Italic.ttf");
static UNBUNTU_FONT_BOLD_ITALIC: [u8; include_bytes!("../../ubuntu_fonts/Ubuntu-BoldItalic.ttf").len()] =
    *include_bytes!("../../ubuntu_fonts/Ubuntu-BoldItalic.ttf");


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


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Eq, PartialEq, Hash, Clone)]
pub enum FontFace {
    TimesNewRomanRegular,
}


#[derive(Eq, PartialEq, Hash)]
pub struct FontKey {
    face: FontFace,
    bold: bool,
    italic: bool,
}


pub struct FontContext {
    pub font_data: HashMap<FontKey, RustTypeFont<'static>>,
}
impl FontContext {
    pub fn new() -> FontContext {
        let mut font_context = FontContext { font_data: HashMap::new() };

        let font = RustTypeFont::try_from_bytes(&UNBUNTU_FONT_REGULAR).expect("Failure loading font data");
        font_context.font_data.insert(FontKey { face: FontFace::TimesNewRomanRegular, bold: false, italic: false }, font);

        let font = RustTypeFont::try_from_bytes(&UNBUNTU_FONT_BOLD).expect("Failure loading font data");
        font_context.font_data.insert(FontKey { face: FontFace::TimesNewRomanRegular, bold: true, italic: false }, font);

        let font = RustTypeFont::try_from_bytes(&UNBUNTU_FONT_ITALIC).expect("Failure loading font data");
        font_context.font_data.insert(FontKey { face: FontFace::TimesNewRomanRegular, bold: false, italic: true }, font);

        let font = RustTypeFont::try_from_bytes(&UNBUNTU_FONT_BOLD_ITALIC).expect("Failure loading font data");
        font_context.font_data.insert(FontKey { face: FontFace::TimesNewRomanRegular, bold: true, italic: true }, font);

        return font_context;
    }

    pub fn get_text_dimension(&self, text: &String, font: &Font) -> (f32, f32) {
        return self.get_text_dimension_str(text.as_str(), font);
    }

    pub fn get_text_dimension_str(&self, text: &str, font: &Font) -> (f32, f32) {
        if text == "" {
            return (0.0, 0.0);
        }

        let rust_type_font = &self.font_data[&font.to_font_key()];

        let scale = Scale::uniform(font.size as f32);
        let v_metrics = rust_type_font.v_metrics(scale);

        let glyphs_height = (v_metrics.ascent - v_metrics.descent + v_metrics.line_gap).ceil();
        let glyphs_width = compute_char_end_positions(rust_type_font, scale, text).last().copied().unwrap_or(0.0);

        return (glyphs_width, glyphs_height);
    }

    pub fn compute_char_position_mapping(&self, font: &Font, text: &String) -> Vec<f32> {
        //This returns the relative ending x positions of each character in the text

        let rust_type_font = &self.font_data[&font.to_font_key()];
        let scale = Scale::uniform(font.size as f32);
        let char_position_mapping = compute_char_end_positions(rust_type_font, scale, text);

        debug_assert!(text.chars().count() == char_position_mapping.len());
        return char_position_mapping;
    }

}


fn compute_char_end_positions(rust_type_font: &RustTypeFont<'static>, scale: Scale, text: &str) -> Vec<f32> {
    //We don't use rusttype's layout() for measuring, because it computes the pixel bounding box of every glyph by parsing its full outline,
    //which is very slow and computes much more than widths. This does the same computation as layout() for widths, so the results are identical.

    let mut char_end_positions = Vec::new();
    let mut position = 0.0;
    let mut last_glyph_id = None;

    for character in text.chars() {
        let glyph = rust_type_font.glyph(character).scaled(scale);
        if let Some(last_glyph_id) = last_glyph_id {
            position += rust_type_font.pair_kerning(scale, last_glyph_id, glyph.id());
        }

        position += glyph.h_metrics().advance_width;
        char_end_positions.push(position);

        last_glyph_id = Some(glyph.id());
    }

    return char_end_positions;
}
