use crate::style::{
    AtRule,
    CssCombinator,
    CssProperty,
    Declaration,
    SelectorType,
    StyleRule,
};
use crate::style::css_lexer;
use crate::style::css_parser;
use crate::style::css_properties::{
    CssPropertyValue,
    FlexBasis,
    FontSize
};
use crate::color::Color;


fn parse_css(css: &str) -> Vec<StyleRule> {
    let tokens = css_lexer::lex_css(&css, 1, 1);
    return css_parser::parse_css(&tokens);
}


#[test]
fn test_parse_basic_style() {
    let result = parse_css("h3 { color:red; }");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].declaration, Declaration { value: CssPropertyValue::Color(Color::new(255, 0, 0)), important: false });
    assert_eq!(result[0].selector.elements[0], (CssCombinator::None, SelectorType::Name, "h3".to_owned()));
}


#[test]
fn test_lex_basic_style_with_more_whitespace() {
    let result = parse_css("h3         {    color:red;      }");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].declaration, Declaration { value: CssPropertyValue::Color(Color::new(255, 0, 0)), important: false });
    assert_eq!(result[0].selector.elements[0], (CssCombinator::None, SelectorType::Name, "h3".to_owned()));
}


#[test]
fn test_comments() {
    let result = parse_css("h3 {/* bla: bla; */ color:red; }");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].declaration, Declaration { value: CssPropertyValue::Color(Color::new(255, 0, 0)), important: false });
    assert_eq!(result[0].selector.elements[0], (CssCombinator::None, SelectorType::Name, "h3".to_owned()));
}


#[test]
fn test_multiple_selectors() {
    let result = parse_css("h3, h4 {color:red}");
    assert_eq!(result.len(), 2);

    assert_eq!(result[0].declaration, Declaration { value: CssPropertyValue::Color(Color::new(255, 0, 0)), important: false });
    assert_eq!(result[0].selector.elements[0], (CssCombinator::None, SelectorType::Name, "h3".to_owned()));

    assert_eq!(result[1].declaration, Declaration { value: CssPropertyValue::Color(Color::new(255, 0, 0)), important: false });
    assert_eq!(result[1].selector.elements[0], (CssCombinator::None, SelectorType::Name, "h4".to_owned()));
}


#[test]
fn test_basic_combinator() {
    let result = parse_css("h3 > p{color:red}");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].declaration, Declaration { value: CssPropertyValue::Color(Color::new(255, 0, 0)), important: false });
    assert_eq!(result[0].selector.elements[0], (CssCombinator::Child, SelectorType::Name, "p".to_owned()));
    assert_eq!(result[0].selector.elements[1], (CssCombinator::None, SelectorType::Name, "h3".to_owned()));
}


#[test]
fn test_parse_id_style() {
    let result = parse_css("#ident { color:red; }");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].declaration, Declaration { value: CssPropertyValue::Color(Color::new(255, 0, 0)), important: false });
    assert_eq!(result[0].selector.elements[0], (CssCombinator::None, SelectorType::Id, "ident".to_owned()));
}


#[test]
fn test_parse_hash_color() {
    let result = parse_css("a { color:#ffffff; }");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].declaration, Declaration { value: CssPropertyValue::Color(Color::WHITE), important: false });
    assert_eq!(result[0].selector.elements[0], (CssCombinator::None, SelectorType::Name, "a".to_owned()));
}


#[test]
fn test_parse_at_rule() {
    let result = parse_css("@media(max-width:500px) { .mw-parser-output { color: blue } }");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].declaration, Declaration { value: CssPropertyValue::Color(Color::BLUE), important: false });
    assert_eq!(result[0].selector.elements[0], (CssCombinator::None, SelectorType::Class, "mw-parser-output".to_owned()));
    assert_eq!(result[0].at_rules.as_ref().unwrap()[0], AtRule::Media("(max-width:500px)".to_owned()));
}


#[test]
fn test_rules_after_at_rule() {
    let result = parse_css("@media(max-width:500px) { .mw-parser-output { color: blue } } b{color: red}");
    assert_eq!(result.len(), 2);
    assert_eq!(result[1].declaration, Declaration { value: CssPropertyValue::Color(Color::new(255, 0, 0)), important: false });
    assert_eq!(result[1].selector.elements[0], (CssCombinator::None, SelectorType::Name, "b".to_owned()));
    assert_eq!(result[1].at_rules.is_none(), true);
}


#[test]
fn test_rules_after_font_face() {
    let result = parse_css("@font-face { font-family: 'X'; font-style: normal; } b{color: red}");
    let b_rule = result.iter().find(|rule| rule.selector.elements[0] == (CssCombinator::None, SelectorType::Name, "b".to_owned())).unwrap();
    assert_eq!(b_rule.declaration, Declaration { value: CssPropertyValue::Color(Color::new(255, 0, 0)), important: false });
    assert_eq!(b_rule.at_rules.is_none(), true);
}


#[test]
fn test_declaration_after_invalid_value() {
    let result = parse_css("a{color:default:#ca1a12;background-color:red}");
    let background_rule = result.iter().find(|rule| rule.declaration.value.property() == CssProperty::BackgroundColor).unwrap();
    assert_eq!(background_rule.declaration.value, CssPropertyValue::BackgroundColor(Color::new(255, 0, 0)));
}


#[test]
fn test_selector_type_resets_after_class() {
    let result = parse_css(".a li, b {color:red}");
    assert_eq!(result[0].selector.elements[0], (CssCombinator::Descendent, SelectorType::Name, "li".to_owned()));
    assert_eq!(result[0].selector.elements[1], (CssCombinator::None, SelectorType::Class, "a".to_owned()));
    assert_eq!(result[1].selector.elements[0], (CssCombinator::None, SelectorType::Name, "b".to_owned()));
}


#[test]
fn test_rules_after_functional_pseudoclass() {
    let result = parse_css("li:nth-last-child(2){display:none} b{color:red}");
    let b_rule = result.iter().find(|rule| rule.selector.elements[0] == (CssCombinator::None, SelectorType::Name, "b".to_owned())).unwrap();
    assert_eq!(b_rule.declaration, Declaration { value: CssPropertyValue::Color(Color::new(255, 0, 0)), important: false });
}


#[test]
fn test_selector_with_unknown_pseudoelement_is_dropped() {
    let result = parse_css(".a::-webkit-scrollbar, .b{display:none}");
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
    let result = parse_css("div { color: green !important; }");
    let color_rule = result.iter().find(|rule| rule.declaration.value.property() == CssProperty::Color).unwrap();
    assert_eq!(color_rule.declaration.value, CssPropertyValue::Color(Color::new(0, 128, 0)));
    assert!(color_rule.declaration.important);
}


#[test]
fn test_flex_shorthand_with_single_number() {
    let result = parse_css(".a { flex: 1; }");
    let has_value = |value: CssPropertyValue| result.iter().any(|rule| rule.declaration.value == value);
    assert!(has_value(CssPropertyValue::FlexGrow(1.0)));
    assert!(has_value(CssPropertyValue::FlexShrink(1.0)));
    assert!(has_value(CssPropertyValue::FlexBasis(FlexBasis::Percentage(0.0))));
}


#[test]
fn test_number_with_leading_dot() {
    let result = parse_css(".a { font-size: .875em; }");
    let font_size_rule = result.iter().find(|rule| rule.declaration.value.property() == CssProperty::FontSize).unwrap();
    assert_eq!(font_size_rule.declaration.value, CssPropertyValue::FontSize(FontSize::Em(0.875)));
}


#[test]
fn test_number_with_decimals() {
    let result = parse_css(".a { font-size: 1.5em; }");
    let font_size_rule = result.iter().find(|rule| rule.declaration.value.property() == CssProperty::FontSize).unwrap();
    assert_eq!(font_size_rule.declaration.value, CssPropertyValue::FontSize(FontSize::Em(1.5)));
}


#[test]
fn test_negative_number_with_leading_dot() {
    let result = parse_css(".a { flex-grow: -.5; }");
    let flex_grow_rule = result.iter().find(|rule| rule.declaration.value.property() == CssProperty::FlexGrow).unwrap();
    assert_eq!(flex_grow_rule.declaration.value, CssPropertyValue::FlexGrow(-0.5));
}


#[test]
fn test_nested_rule_is_scoped_to_parent_selector() {
    let result = parse_css(".outer { .inner { color: red; } }");
    let color_rule = result.iter().find(|rule| rule.declaration.value.property() == CssProperty::Color).unwrap();
    assert_eq!(color_rule.selector.elements, vec![
        (CssCombinator::Descendent, SelectorType::Class, "inner".to_owned()),
        (CssCombinator::None, SelectorType::Class, "outer".to_owned()),
    ]);
}


#[test]
fn test_compound_selector_is_not_parsed_as_descendant() {
    let result = parse_css("p div.a { color: red; }");
    let color_rule = result.iter().find(|rule| rule.declaration.value.property() == CssProperty::Color).unwrap();
    assert_eq!(color_rule.selector.elements, vec![
        (CssCombinator::None, SelectorType::Class, "a".to_owned()),
        (CssCombinator::Descendent, SelectorType::Name, "div".to_owned()),
        (CssCombinator::None, SelectorType::Name, "p".to_owned()),
    ]);
}


#[test]
fn test_nesting_selector_compounds_with_parent() {
    let result = parse_css(".a { &.b { color: red; } }");
    let color_rule = result.iter().find(|rule| rule.declaration.value.property() == CssProperty::Color).unwrap();
    assert_eq!(color_rule.selector.elements, vec![
        (CssCombinator::None, SelectorType::Class, "b".to_owned()),
        (CssCombinator::None, SelectorType::Class, "a".to_owned()),
    ]);
}
