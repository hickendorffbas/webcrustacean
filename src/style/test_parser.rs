use crate::style::{
    AtRule,
    CssCombinator,
    CssFunction,
    CssProperty,
    CssValue,
    SelectorType,
};
use crate::style::css_lexer;
use crate::style::css_parser;


#[test]
fn test_parse_basic_style() {
    let css_text = "h3 { color:red; }";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_css(&tokens);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].property, CssProperty::Color);
    assert_eq!(result[0].value, CssValue::String("red".to_owned()));
    assert_eq!(result[0].selector.elements[0], (CssCombinator::None, SelectorType::Name, "h3".to_owned()));
}


#[test]
fn test_lex_basic_style_with_more_whitespace() {
    let css_text = "h3         {    color:red;      }";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_css(&tokens);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].property, CssProperty::Color);
    assert_eq!(result[0].value, CssValue::String("red".to_owned()));
    assert_eq!(result[0].selector.elements[0], (CssCombinator::None, SelectorType::Name, "h3".to_owned()));
}


#[test]
fn test_comments() {
    let css_text = "h3 {/* bla: bla; */ color:red; }";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_css(&tokens);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].property, CssProperty::Color);
    assert_eq!(result[0].value, CssValue::String("red".to_owned()));
    assert_eq!(result[0].selector.elements[0], (CssCombinator::None, SelectorType::Name, "h3".to_owned()));
}


#[test]
fn test_multiple_selectors() {
    let css_text = "h3, h4 {color:red}";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_css(&tokens);

    assert_eq!(result.len(), 2);

    assert_eq!(result[0].property, CssProperty::Color);
    assert_eq!(result[0].value, CssValue::String("red".to_owned()));
    assert_eq!(result[0].selector.elements[0], (CssCombinator::None, SelectorType::Name, "h3".to_owned()));

    assert_eq!(result[1].property, CssProperty::Color);
    assert_eq!(result[1].value, CssValue::String("red".to_owned()));
    assert_eq!(result[1].selector.elements[0], (CssCombinator::None, SelectorType::Name, "h4".to_owned()));
}


#[test]
fn test_basic_combinator() {
    let css_text = "h3 > p{color:red}";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_css(&tokens);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].property, CssProperty::Color);
    assert_eq!(result[0].value, CssValue::String("red".to_owned()));
    assert_eq!(result[0].selector.elements[0], (CssCombinator::Child, SelectorType::Name, "p".to_owned()));
    assert_eq!(result[0].selector.elements[1], (CssCombinator::None, SelectorType::Name, "h3".to_owned()));
}


#[test]
fn test_parse_id_style() {
    let css_text = "#ident { color:red; }";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_css(&tokens);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].property, CssProperty::Color);
    assert_eq!(result[0].value, CssValue::String("red".to_owned()));
    assert_eq!(result[0].selector.elements[0], (CssCombinator::None, SelectorType::Id, "ident".to_owned()));
}


#[test]
fn test_parse_function() {
    let css_text = "h3 { color:rgb(10 20 30); }";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_css(&tokens);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].property, CssProperty::Color);
    assert_eq!(result[0].value, CssValue::Function(CssFunction { name: "rgb".to_owned(),
                                                                 arguments: vec![CssValue::String("10".to_owned()),
                                                                                 CssValue::String("20".to_owned()),
                                                                                 CssValue::String("30".to_owned())] }));
    assert_eq!(result[0].selector.elements[0], (CssCombinator::None, SelectorType::Name, "h3".to_owned()));
}
//TODO: test a nested css function (for example calc())


#[test]
fn test_parse_hash_color() {
    let css_text = "a { color:#ffffff; }";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_css(&tokens);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].property, CssProperty::Color);
    assert_eq!(result[0].value, CssValue::Hash("ffffff".to_owned()));
    assert_eq!(result[0].selector.elements[0], (CssCombinator::None, SelectorType::Name, "a".to_owned()));
}


#[test]
fn test_parse_at_rule() {
    let css_text = "@media(max-width:500px) { .mw-parser-output { color: blue } }";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_css(&tokens);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].property, CssProperty::Color);
    assert_eq!(result[0].value, CssValue::String("blue".to_owned()));
    assert_eq!(result[0].selector.elements[0], (CssCombinator::None, SelectorType::Class, "mw-parser-output".to_owned()));
    assert_eq!(result[0].at_rules.as_ref().unwrap()[0], AtRule::Media("(max-width:500px)".to_owned()));
}


#[test]
fn test_rules_after_at_rule() {
    let css_text = "@media(max-width:500px) { .mw-parser-output { color: blue } } b{color: red}";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_css(&tokens);

    assert_eq!(result.len(), 2);
    assert_eq!(result[1].property, CssProperty::Color);
    assert_eq!(result[1].value, CssValue::String("red".to_owned()));
    assert_eq!(result[1].selector.elements[0], (CssCombinator::None, SelectorType::Name, "b".to_owned()));
    assert_eq!(result[1].at_rules.is_none(), true);
}


#[test]
fn test_rules_after_font_face() {
    let css_text = "@font-face { font-family: 'X'; font-style: normal; } b{color: red}";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_css(&tokens);

    let b_rule = result.iter().find(|rule| rule.selector.elements[0] == (CssCombinator::None, SelectorType::Name, "b".to_owned())).unwrap();
    assert_eq!(b_rule.property, CssProperty::Color);
    assert_eq!(b_rule.value, CssValue::String("red".to_owned()));
    assert_eq!(b_rule.at_rules.is_none(), true);
}


#[test]
fn test_declaration_after_invalid_value() {
    let css_text = "a{color:default:#ca1a12;background-color:red}";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_css(&tokens);

    let background_rule = result.iter().find(|rule| rule.property == CssProperty::BackgroundColor).unwrap();
    assert_eq!(background_rule.value, CssValue::String("red".to_owned()));
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
    assert_eq!(b_rule.property, CssProperty::Color);
    assert_eq!(b_rule.value, CssValue::String("red".to_owned()));
}


#[test]
fn test_selector_with_unknown_pseudoelement_is_dropped() {
    let css_text = ".a::-webkit-scrollbar, .b{display:none}";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_css(&tokens);

    let a_selector = (CssCombinator::None, SelectorType::Class, "a".to_owned());
    let b_selector = (CssCombinator::None, SelectorType::Class, "b".to_owned());
    assert!(result.iter().all(|rule| rule.selector.elements[0] != a_selector));
    assert!(result.iter().any(|rule| rule.selector.elements[0] == b_selector && rule.property == CssProperty::Display));
}


#[test]
fn test_function_arguments_keep_commas_and_nesting() {
    let css_text = "h3 { font-size: var(--a, var(--b, 16px)); }";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_css(&tokens);

    let font_size_rule = result.iter().find(|rule| rule.property == CssProperty::FontSize).unwrap();
    let inner_var = CssValue::Function(CssFunction { name: "var".to_owned(),
                                                     arguments: vec![CssValue::String("--b".to_owned()),
                                                                     CssValue::Comma,
                                                                     CssValue::String("16px".to_owned())] });
    assert_eq!(font_size_rule.value, CssValue::Function(CssFunction { name: "var".to_owned(),
                                                                      arguments: vec![CssValue::String("--a".to_owned()),
                                                                                      CssValue::Comma,
                                                                                      inner_var] }));
}
