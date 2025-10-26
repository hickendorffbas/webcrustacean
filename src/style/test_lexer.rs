use crate::style::css_lexer::{self, CssToken};


#[test]
fn test_parse_basic_style() {
    let css_text = "h3 { color: red; }";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);

    let expected_tokens = vec![
        CssToken::Selector("h3".to_owned()),
        CssToken::BlockStart,
        CssToken::Property("color".to_owned()),
        CssToken::Value("red".to_owned()),
        CssToken::BlockEnd,
    ];
    assert_eq!(tokens.len(), expected_tokens.len());

    for (token, expected_token) in tokens.iter().zip(expected_tokens.iter()) {
        assert_eq!(&token.css_token, expected_token);
    }
}


#[test]
fn test_comments() {
    let css_text = "h3 { /* bla: bla; */ color: red; }";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);

    let expected_tokens = vec![
        CssToken::Selector("h3".to_owned()),
        CssToken::BlockStart,
        CssToken::Property("color".to_owned()),
        CssToken::Value("red".to_owned()),
        CssToken::BlockEnd,
    ];
    assert_eq!(tokens.len(), expected_tokens.len());

    for (token, expected_token) in tokens.iter().zip(expected_tokens.iter()) {
        assert_eq!(&token.css_token, expected_token);
    }
}



#[test]
fn test_parse_basic_style_no_trailing_semicolon() {
    let css_text = "h3 { color: red; text-decoration: none }";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);

    let expected_tokens = vec![
        CssToken::Selector("h3".to_owned()),
        CssToken::BlockStart,
        CssToken::Property("color".to_owned()),
        CssToken::Value("red".to_owned()),
        CssToken::Property("text-decoration".to_owned()),
        CssToken::Value("none".to_owned()),
        CssToken::BlockEnd,
    ];
    assert_eq!(tokens.len(), expected_tokens.len());

    for (token, expected_token) in tokens.iter().zip(expected_tokens.iter()) {
        assert_eq!(&token.css_token, expected_token);
    }
}


#[test]
fn test_parsing_nested_style() {
    let css_text = "p { color: red; h3 { text-decoration: none; }}";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);

    let expected_tokens = vec![
        CssToken::Selector("p".to_owned()),
        CssToken::BlockStart,
        CssToken::Property("color".to_owned()),
        CssToken::Value("red".to_owned()),
        CssToken::Selector("h3".to_owned()),
        CssToken::BlockStart,
        CssToken::Property("text-decoration".to_owned()),
        CssToken::Value("none".to_owned()),
        CssToken::BlockEnd,
        CssToken::BlockEnd,
    ];
    assert_eq!(tokens.len(), expected_tokens.len());

    for (token, expected_token) in tokens.iter().zip(expected_tokens.iter()) {
        assert_eq!(&token.css_token, expected_token);
    }
}


#[test]
fn test_multiple_selectors() {
    let css_text = "h3, h4 { color: red; }";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);

    let expected_tokens = vec![
        CssToken::Selector("h3".to_owned()),
        CssToken::Comma,
        CssToken::Selector("h4".to_owned()),
        CssToken::BlockStart,
        CssToken::Property("color".to_owned()),
        CssToken::Value("red".to_owned()),
        CssToken::BlockEnd,
    ];
    assert_eq!(tokens.len(), expected_tokens.len());

    for (token, expected_token) in tokens.iter().zip(expected_tokens.iter()) {
        assert_eq!(&token.css_token, expected_token);
    }
}


#[test]
fn test_basic_combinator() {
    let css_text = "h3 > p { color: red; }";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);

    let expected_tokens = vec![
        CssToken::Selector("h3".to_owned()),
        CssToken::ChildCombinator,
        CssToken::Selector("p".to_owned()),
        CssToken::BlockStart,
        CssToken::Property("color".to_owned()),
        CssToken::Value("red".to_owned()),
        CssToken::BlockEnd,
    ];
    assert_eq!(tokens.len(), expected_tokens.len());

    for (token, expected_token) in tokens.iter().zip(expected_tokens.iter()) {
        assert_eq!(&token.css_token, expected_token);
    }
}
