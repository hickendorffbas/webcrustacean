use std::collections::HashMap;

use crate::color::Color;
use crate::debug::debug_log_warn;
use crate::style::Declaration;
use crate::style::css_parser::{CssParsedValue, ParsedDeclaration};


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum CssProperty { //TODO: I think this should become CssParsedProperty, and move to mod.rs
    BackgroundColor,
    Color,
    Display,
    Flex,
    FlexBasis,
    FlexDirection,
    FlexGrow,
    FlexShrink,
    FontWeight,
    FontSize,
    FontStyle,
    TextDecoration,
}
impl CssProperty {
    pub fn from_string(value: &String) -> Option<CssProperty> {
        let prop = match value.as_str() {
            "background-color" => CssProperty::BackgroundColor,
            "color" => CssProperty::Color,
            "display" => CssProperty::Display,
            "flex" => CssProperty::Flex,
            "flex-basis" => CssProperty::FlexBasis,
            "flex-direction" => CssProperty::FlexDirection,
            "flex-grow" => CssProperty::FlexGrow,
            "flex-shrink" => CssProperty::FlexShrink,
            "font-weight" => CssProperty::FontWeight,
            "font-size" => CssProperty::FontSize,
            "font-style" => CssProperty::FontStyle,
            "text-decoration" => CssProperty::TextDecoration,
            _ => return None,
        };
        return Some(prop);
    }

    pub fn is_inherited(&self) -> bool {
        return match self {
            CssProperty::Color => true,
            CssProperty::FontSize => true,
            CssProperty::FontStyle => true,
            CssProperty::FontWeight => true,

            _ => false,
        };
    }
}


#[derive(PartialEq, Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum CssPropertyValue {
    BackgroundColor(Color),
    Color(Color),
    Display(Display),
    FlexBasis(FlexBasis),
    FlexDirection(FlexDirection),
    FlexGrow(f32),
    FlexShrink(f32),
    FontSize(FontSize),
    FontStyle(FontStyle),
    FontWeight(FontWeight),
    TextDecoration(TextDecoration),
}
impl CssPropertyValue {
    pub fn property(&self) -> CssProperty {
        match self {
            CssPropertyValue::BackgroundColor(_) => CssProperty::BackgroundColor,
            CssPropertyValue::Color(_) => CssProperty::Color,
            CssPropertyValue::Display(_) => CssProperty::Display,
            CssPropertyValue::FlexBasis(_) => CssProperty::FlexBasis,
            CssPropertyValue::FlexDirection(_) => CssProperty::FlexDirection,
            CssPropertyValue::FlexGrow(_) => CssProperty::FlexGrow,
            CssPropertyValue::FlexShrink(_) => CssProperty::FlexShrink,
            CssPropertyValue::FontSize(_) => CssProperty::FontSize,
            CssPropertyValue::FontStyle(_) => CssProperty::FontStyle,
            CssPropertyValue::FontWeight(_) => CssProperty::FontWeight,
            CssPropertyValue::TextDecoration(_) => CssProperty::TextDecoration,
        }
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Styles(HashMap<CssProperty, CssPropertyValue>);
impl Styles {
    pub fn new() -> Styles {
        return Styles(HashMap::new());
    }

    pub fn insert(&mut self, value: CssPropertyValue) {
        self.0.insert(value.property(), value);
    }

    pub fn inherit_from(&mut self, parent: &Styles) {
        for (property, value) in &parent.0 {
            if property.is_inherited() && !self.0.contains_key(property) {
                self.0.insert(*property, value.clone());
            }
        }
    }

    pub fn background_color(&self) -> Color {
        match self.0.get(&CssProperty::BackgroundColor) {
            Some(CssPropertyValue::BackgroundColor(color)) => *color,
            _ => Color::WHITE,
        }
    }

    pub fn color(&self) -> Color {
        match self.0.get(&CssProperty::Color) {
            Some(CssPropertyValue::Color(color)) => *color,
            _ => Color::BLACK,
        }
    }

    pub fn display(&self) -> Display {
        match self.0.get(&CssProperty::Display) {
            Some(CssPropertyValue::Display(display)) => display.clone(),
            _ => Display::Inline,
        }
    }

    pub fn flex_basis(&self) -> FlexBasis {
        match self.0.get(&CssProperty::FlexBasis) {
            Some(CssPropertyValue::FlexBasis(basis)) => basis.clone(),
            _ => FlexBasis::Auto,
        }
    }

    pub fn flex_direction(&self) -> FlexDirection {
        match self.0.get(&CssProperty::FlexDirection) {
            Some(CssPropertyValue::FlexDirection(direction)) => direction.clone(),
            _ => FlexDirection::Row,
        }
    }

    pub fn font_size(&self) -> FontSize {
        match self.0.get(&CssProperty::FontSize) {
            Some(CssPropertyValue::FontSize(font_size)) => font_size.clone(),
            _ => FontSize::Px(18.0),
        }
    }

    pub fn font_style(&self) -> FontStyle {
        match self.0.get(&CssProperty::FontStyle) {
            Some(CssPropertyValue::FontStyle(font_style)) => font_style.clone(),
            _ => FontStyle::Normal,
        }
    }

    pub fn font_weight(&self) -> FontWeight {
        match self.0.get(&CssProperty::FontWeight) {
            Some(CssPropertyValue::FontWeight(font_weight)) => font_weight.clone(),
            _ => FontWeight::Normal,
        }
    }

    #[allow(unused)] //TODO: underline does not currently work
    pub fn text_decoration(&self) -> TextDecoration {
        match self.0.get(&CssProperty::TextDecoration) {
            Some(CssPropertyValue::TextDecoration(text_decoration)) => text_decoration.clone(),
            _ => TextDecoration::None,
        }
    }
}


#[derive(PartialEq, Clone, Copy)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum Display {
    Block,
    Inline,
    Flex,
    None,
}
impl Display {
    pub fn from_string(value: &str) -> Option<Display> {
        return match value {
            "block" => Some(Display::Block),
            "inline" => Some(Display::Inline),
            "flex" => Some(Display::Flex),
            "none" => Some(Display::None),
            _ => None,
        };
    }

    pub fn is_block_level(&self) -> bool {
        return match self {
            Display::Block | Display::Flex => true,
            _ => false,
        };
    }

    pub fn is_inline_level(&self) -> bool {
        return match self {
            Display::Inline => true,
            _ => false,
        };
    }
}



#[derive(PartialEq, Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum FlexBasis {
    Auto,
    Length(f32),
    Percentage(f32),
}


#[derive(PartialEq, Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}


#[derive(PartialEq, Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum FontSize {
    Px(f32),
    Em(f32),
    Percentage(f32),
}


#[derive(PartialEq, Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum FontStyle {
    Normal,
    Italic,
}


#[derive(PartialEq, Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum FontWeight {
    Normal,
    Bold,
}


#[derive(PartialEq, Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum TextDecoration {
    None,
    Underline,
}


fn split_dimension(text: &str) -> Option<(f32, &str)> {
    let mut number_end = 0;

    for (idx, character) in text.char_indices() {
        let is_sign = idx == 0 && (character == '-' || character == '+');
        if !is_sign && !character.is_ascii_digit() && character != '.' {
            break;
        }
        number_end = idx + character.len_utf8();
    }

    let number = text[..number_end].parse::<f32>().ok()?;
    return Some((number, &text[number_end..]));
}


pub fn type_declaration(parsed_declaration: ParsedDeclaration) -> Vec<Declaration> {
    let mut values = Vec::new();

    match parsed_declaration.property {
        CssProperty::BackgroundColor => {
            match &parsed_declaration.value {
                CssParsedValue::String(color_name) => {
                    if let Some(color) = Color::from_string(&color_name) {
                        values.push(CssPropertyValue::BackgroundColor(color));
                    }
                },
                CssParsedValue::Hash(hash_value) => {
                    //TODO: this could use an entry point into color that just parses a hash value
                    if let Some(color) = Color::from_string(&(String::from("#") + &hash_value)) {
                        values.push(CssPropertyValue::BackgroundColor(color));
                    }
                },
                _ => {},
            }
        },
        CssProperty::Color => {
            match &parsed_declaration.value {
                CssParsedValue::String(color_name) => {
                    if let Some(color) = Color::from_string(&color_name) {
                        values.push(CssPropertyValue::Color(color));
                    }
                },
                CssParsedValue::Hash(hash_value) => {
                    //TODO: this could use an entry point into color that just parses a hash value
                    if let Some(color) = Color::from_string(&(String::from("#") + &hash_value)) {
                        values.push(CssPropertyValue::Color(color));
                    }
                },
                _ => {},
            }
        },
        CssProperty::Display => {
            match &parsed_declaration.value {
                CssParsedValue::String(display_name) => {
                    if let Some(parsed) = Display::from_string(&display_name) {
                        values.push(CssPropertyValue::Display(parsed));
                    }
                },
                CssParsedValue::List(_) => {
                    //TODO: this is valid, but needs support for inner and outer types, for now we are just dropping the values
                },
                _ => {},
            }
        },
        CssProperty::Flex => {
            unreachable!(); //This can't be reached due to shorthand expansion, which should happen before this function
        },
        CssProperty::FlexBasis => {
            match &parsed_declaration.value {
                CssParsedValue::String(flex_basis) => {
                    if flex_basis == "auto" {
                        values.push(CssPropertyValue::FlexBasis(FlexBasis::Auto));
                    } else {
                        if let Some((amount, dimension)) = split_dimension(flex_basis) {
                            match dimension {
                                "px" => values.push(CssPropertyValue::FlexBasis(FlexBasis::Length(amount))),
                                "%" => values.push(CssPropertyValue::FlexBasis(FlexBasis::Percentage(amount))),
                                "" => { if amount == 0.0 { values.push(CssPropertyValue::FlexBasis(FlexBasis::Length(0.0))) } }
                                _ => {},
                            }
                        }
                    }
                },
                _ => {},
            }   
        },
        CssProperty::FlexDirection => {
            match &parsed_declaration.value {
                CssParsedValue::String(flex_direction) => {
                    match flex_direction.as_str() {
                        "row" => values.push(CssPropertyValue::FlexDirection(FlexDirection::Row)),
                        "column" => values.push(CssPropertyValue::FlexDirection(FlexDirection::Column)),
                        "row-reverse" => values.push(CssPropertyValue::FlexDirection(FlexDirection::RowReverse)),
                        "column-reverse" => values.push(CssPropertyValue::FlexDirection(FlexDirection::ColumnReverse)),
                        _ => {},
                    }
                    
                },
                _ => {},
            }
        },
        CssProperty::FlexGrow => {
            match &parsed_declaration.value {
                CssParsedValue::String(number) => {
                    if let Ok(num) = number.parse::<f32>() {
                        values.push(CssPropertyValue::FlexGrow(num));
                    }
                },
                _ => {},
            }
        },
        CssProperty::FlexShrink => {
            match &parsed_declaration.value {
                CssParsedValue::String(number) => {
                    if let Ok(num) = number.parse::<f32>() {
                        values.push(CssPropertyValue::FlexShrink(num));
                    }
                },
                _ => {},
            }
        },
        CssProperty::FontWeight => {
            match &parsed_declaration.value {
                CssParsedValue::String(font_weight) => {
                    match font_weight.as_str() {
                        "normal" => values.push(CssPropertyValue::FontWeight(FontWeight::Normal)),
                        "bold" => values.push(CssPropertyValue::FontWeight(FontWeight::Bold)),
                        _ => {},
                    }
                },
                _ => {},
            }
        },
        CssProperty::FontSize => {
            match &parsed_declaration.value {
                CssParsedValue::String(font_size) => {
                    if let Some((amount, dimension)) = split_dimension(font_size) {
                        match dimension {
                            "px" => values.push(CssPropertyValue::FontSize(FontSize::Px(amount))),
                            "em" => values.push(CssPropertyValue::FontSize(FontSize::Em(amount))),
                            "%" => values.push(CssPropertyValue::FontSize(FontSize::Percentage(amount))),
                            "" => { if amount == 0.0 { values.push(CssPropertyValue::FontSize(FontSize::Px(0.0))) } }
                            _ => {},
                        }
                    }
                },
                _ => {},
            }  
        },
        CssProperty::FontStyle => {
            match &parsed_declaration.value {
                CssParsedValue::String(font_weight) => {
                    match font_weight.as_str() {
                        "normal" => values.push(CssPropertyValue::FontStyle(FontStyle::Normal)),
                        "italic" => values.push(CssPropertyValue::FontStyle(FontStyle::Italic)),
                        _ => {},
                    }
                },
                _ => {},
            }
        },
        CssProperty::TextDecoration => {
            match &parsed_declaration.value {
                CssParsedValue::String(font_weight) => {
                    match font_weight.as_str() {
                        "none" => values.push(CssPropertyValue::TextDecoration(TextDecoration::None)),
                        "underline" => values.push(CssPropertyValue::TextDecoration(TextDecoration::Underline)),
                        _ => {},
                    }
                },
                _ => {},
            }
        }
    };

    let mut declarations = Vec::new();

    if values.is_empty() {
        match &parsed_declaration.value {
            //TODO: these warnings should show up in the devtools window log when we have it

            CssParsedValue::String(value_name) => { debug_log_warn(format!("Ignored unknown property value: {}", value_name)); },
            CssParsedValue::Hash(hash_value) => { debug_log_warn(format!("Ignored unknown property value: #{}", hash_value)); },
            _ => { debug_log_warn(format!("Ignored unknown property value")); },
        }
    }

    for value in values {
        declarations.push(Declaration { value, important: parsed_declaration.important })
    }

    return declarations;
}
