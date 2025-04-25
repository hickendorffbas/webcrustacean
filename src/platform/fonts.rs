use std::collections::HashMap;

use rusttype::{
    point,
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
        let glyphs_width = rust_type_font.layout(text, scale, point(0.0, 0.0)).last()
                .map(|g| g.position().x + g.unpositioned().h_metrics().advance_width)
                .unwrap_or(0.0);

        return (glyphs_width, glyphs_height);
    }

    pub fn compute_char_position_mapping(&self, font: &Font, text: &String) -> Vec<f32> {
        //This returns the relative ending x positions of each character in the text

        let mut char_position_mapping = Vec::new();

        let rust_type_font = &self.font_data[&font.to_font_key()];

        let scale = Scale::uniform(font.size as f32);
        let v_metrics = rust_type_font.v_metrics(scale);
        let glyphs: Vec<_> = rust_type_font.layout(text, scale, point(0.0, v_metrics.ascent)).collect();

        for glyph in glyphs {
            char_position_mapping.push(glyph.position().x + glyph.unpositioned().h_metrics().advance_width);
        }

        debug_assert!(text.chars().count() == char_position_mapping.len());
        return char_position_mapping;
    }

}
