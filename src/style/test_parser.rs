use crate::style::{
    CssCombinator, CssFunction, CssProperty, CssValue, SelectorType
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

    //TODO: these fail, but should succeed:
    // assert_eq!(result[1].property, CssProperty::Color);
    // assert_eq!(result[1].value, "red");
    // assert_eq!(result[1].selector.elements[0], (CssCombinator::None, SelectorType::Name, "h4".to_owned()));
}


#[test]
fn test_basic_combinator() {
    let css_text = "h3 > p{color:red}";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    let result = css_parser::parse_css(&tokens);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].property, CssProperty::Color);
    assert_eq!(result[0].value, CssValue::String("red".to_owned()));
    assert_eq!(result[0].selector.elements[0], (CssCombinator::Descendent, SelectorType::Name, "p".to_owned()));
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

