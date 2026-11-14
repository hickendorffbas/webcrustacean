use crate::style::{
    AtRule,
    CssCombinator,
    CssProperty,
    Declaration,
    SelectorType,
};
use crate::style::css_lexer;
use crate::style::css_parser;
use crate::style::css_properties::{CssPropertyValue, FlexBasis};
use crate::color::Color;


#[test]
fn test_parse_basic_style() {
    let css_text = "h3 { color:red; }";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_css(&tokens);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].declaration, Declaration { value: CssPropertyValue::Color(Color::new(255, 0, 0)), important: false });
    assert_eq!(result[0].selector.elements[0], (CssCombinator::None, SelectorType::Name, "h3".to_owned()));
}


#[test]
fn test_lex_basic_style_with_more_whitespace() {
    let css_text = "h3         {    color:red;      }";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_css(&tokens);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].declaration, Declaration { value: CssPropertyValue::Color(Color::new(255, 0, 0)), important: false });
    assert_eq!(result[0].selector.elements[0], (CssCombinator::None, SelectorType::Name, "h3".to_owned()));
}


#[test]
fn test_comments() {
    let css_text = "h3 {/* bla: bla; */ color:red; }";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_css(&tokens);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].declaration, Declaration { value: CssPropertyValue::Color(Color::new(255, 0, 0)), important: false });
    assert_eq!(result[0].selector.elements[0], (CssCombinator::None, SelectorType::Name, "h3".to_owned()));
}


#[test]
fn test_multiple_selectors() {
    let css_text = "h3, h4 {color:red}";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_css(&tokens);

    assert_eq!(result.len(), 2);

    assert_eq!(result[0].declaration, Declaration { value: CssPropertyValue::Color(Color::new(255, 0, 0)), important: false });
    assert_eq!(result[0].selector.elements[0], (CssCombinator::None, SelectorType::Name, "h3".to_owned()));

    assert_eq!(result[1].declaration, Declaration { value: CssPropertyValue::Color(Color::new(255, 0, 0)), important: false });
    assert_eq!(result[1].selector.elements[0], (CssCombinator::None, SelectorType::Name, "h4".to_owned()));
}


#[test]
fn test_basic_combinator() {
    let css_text = "h3 > p{color:red}";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_css(&tokens);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].declaration, Declaration { value: CssPropertyValue::Color(Color::new(255, 0, 0)), important: false });
    assert_eq!(result[0].selector.elements[0], (CssCombinator::Child, SelectorType::Name, "p".to_owned()));
    assert_eq!(result[0].selector.elements[1], (CssCombinator::None, SelectorType::Name, "h3".to_owned()));
}


#[test]
fn test_parse_id_style() {
    let css_text = "#ident { color:red; }";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_css(&tokens);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].declaration, Declaration { value: CssPropertyValue::Color(Color::new(255, 0, 0)), important: false });
    assert_eq!(result[0].selector.elements[0], (CssCombinator::None, SelectorType::Id, "ident".to_owned()));
}


#[test]
fn test_parse_hash_color() {
    let css_text = "a { color:#ffffff; }";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_css(&tokens);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].declaration, Declaration { value: CssPropertyValue::Color(Color::WHITE), important: false });
    assert_eq!(result[0].selector.elements[0], (CssCombinator::None, SelectorType::Name, "a".to_owned()));
}


#[test]
fn test_parse_at_rule() {
    let css_text = "@media(max-width:500px) { .mw-parser-output { color: blue } }";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_css(&tokens);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].declaration, Declaration { value: CssPropertyValue::Color(Color::BLUE), important: false });
    assert_eq!(result[0].selector.elements[0], (CssCombinator::None, SelectorType::Class, "mw-parser-output".to_owned()));
    assert_eq!(result[0].at_rules.as_ref().unwrap()[0], AtRule::Media("(max-width:500px)".to_owned()));
}


#[test]
fn test_rules_after_at_rule() {
    let css_text = "@media(max-width:500px) { .mw-parser-output { color: blue } } b{color: red}";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_css(&tokens);

    assert_eq!(result.len(), 2);
    assert_eq!(result[1].declaration, Declaration { value: CssPropertyValue::Color(Color::new(255, 0, 0)), important: false });
    assert_eq!(result[1].selector.elements[0], (CssCombinator::None, SelectorType::Name, "b".to_owned()));
    assert_eq!(result[1].at_rules.is_none(), true);
}


#[test]
fn test_rules_after_font_face() {
    let css_text = "@font-face { font-family: 'X'; font-style: normal; } b{color: red}";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_css(&tokens);

    let b_rule = result.iter().find(|rule| rule.selector.elements[0] == (CssCombinator::None, SelectorType::Name, "b".to_owned())).unwrap();
    assert_eq!(b_rule.declaration, Declaration { value: CssPropertyValue::Color(Color::new(255, 0, 0)), important: false });
    assert_eq!(b_rule.at_rules.is_none(), true);
}


#[test]
fn test_declaration_after_invalid_value() {
    let css_text = "a{color:default:#ca1a12;background-color:red}";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_css(&tokens);

    let background_rule = result.iter().find(|rule| rule.declaration.value.property() == CssProperty::BackgroundColor).unwrap();
    assert_eq!(background_rule.declaration.value, CssPropertyValue::BackgroundColor(Color::new(255, 0, 0)));
}


#[test]
fn test_selector_type_resets_after_class() {
    let css_text = ".a li, b {color:red}";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_css(&tokens);

    assert_eq!(result[0].selector.elements[0], (CssCombinator::Descendent, SelectorType::Name, "li".to_owned()));
    assert_eq!(result[0].selector.elements[1], (CssCombinator::None, SelectorType::Class, "a".to_owned()));
    assert_eq!(result[1].selector.elements[0], (CssCombinator::None, SelectorType::Name, "b".to_owned()));
}


#[test]
fn test_rules_after_functional_pseudoclass() {
    let css_text = "li:nth-last-child(2){display:none} b{color:red}";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_css(&tokens);

    let b_rule = result.iter().find(|rule| rule.selector.elements[0] == (CssCombinator::None, SelectorType::Name, "b".to_owned())).unwrap();
    assert_eq!(b_rule.declaration, Declaration { value: CssPropertyValue::Color(Color::new(255, 0, 0)), important: false });
}


#[test]
fn test_selector_with_unknown_pseudoelement_is_dropped() {
    let css_text = ".a::-webkit-scrollbar, .b{display:none}";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_css(&tokens);

    let a_selector = (CssCombinator::None, SelectorType::Class, "a".to_owned());
    let b_selector = (CssCombinator::None, SelectorType::Class, "b".to_owned());
    assert!(result.iter().all(|rule| rule.selector.elements[0] != a_selector));
    assert!(result.iter().any(|rule| rule.selector.elements[0] == b_selector && rule.declaration.value.property() == CssProperty::Display));
}


#[test]
fn test_declaration_list_without_selector_or_braces() {
    let css_text = "color: blue; background-color: yellow";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_declaration_list(&tokens);

    let color = result.iter().find(|declaration| declaration.value.property() == CssProperty::Color).unwrap();
    assert_eq!(color.value, CssPropertyValue::Color(Color::BLUE));

    let background_color = result.iter().find(|declaration| declaration.value.property() == CssProperty::BackgroundColor).unwrap();
    assert_eq!(background_color.value, CssPropertyValue::BackgroundColor(Color::new(255, 255, 0)));
}


#[test]
fn test_important_is_parsed_as_flag() {
    let css_text = "div { color: green !important; }";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_css(&tokens);

    let color_rule = result.iter().find(|rule| rule.declaration.value.property() == CssProperty::Color).unwrap();
    assert_eq!(color_rule.declaration.value, CssPropertyValue::Color(Color::new(0, 128, 0)));
    assert!(color_rule.declaration.important);
}


#[test]
fn test_flex_shorthand_with_single_number() {
    let css_text = ".a { flex: 1; }";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_css(&tokens);

    let has_value = |value: CssPropertyValue| result.iter().any(|rule| rule.declaration.value == value);
    assert!(has_value(CssPropertyValue::FlexGrow(1.0)));
    assert!(has_value(CssPropertyValue::FlexShrink(1.0)));
    assert!(has_value(CssPropertyValue::FlexBasis(FlexBasis::Percentage(0.0))));
}
