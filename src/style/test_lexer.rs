use crate::style::css_lexer::{self, CssToken};


#[test]
fn test_lex_basic_style() {
    let css_text = "h3 { color:red; }";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);

    let expected_tokens = vec![
        CssToken::Identifier("h3".to_owned()),
        CssToken::Whitespace,
        CssToken::OpenBrace,
        CssToken::Whitespace,
        CssToken::Identifier("color".to_owned()),
        CssToken::Colon,
        CssToken::Identifier("red".to_owned()),
        CssToken::Semicolon,
        CssToken::Whitespace,
        CssToken::CloseBrace,
    ];
    assert_eq!(tokens.len(), expected_tokens.len());

    for (token, expected_token) in tokens.iter().zip(expected_tokens.iter()) {
        assert_eq!(&token.css_token, expected_token);
    }
}


#[test]
fn test_lex_basic_style_with_more_whitespace() {
    let css_text = "h3         {    color:red;      }";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);

    let expected_tokens = vec![
        CssToken::Identifier("h3".to_owned()),
        CssToken::Whitespace,
        CssToken::OpenBrace,
        CssToken::Whitespace,
        CssToken::Identifier("color".to_owned()),
        CssToken::Colon,
        CssToken::Identifier("red".to_owned()),
        CssToken::Semicolon,
        CssToken::Whitespace,
        CssToken::CloseBrace,
    ];
    assert_eq!(tokens.len(), expected_tokens.len());

    for (token, expected_token) in tokens.iter().zip(expected_tokens.iter()) {
        assert_eq!(&token.css_token, expected_token);
    }
}



#[test]
fn test_comments() {
    let css_text = "h3 {/* bla: bla; */ color:red; }";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);

    let expected_tokens = vec![
        CssToken::Identifier("h3".to_owned()),
        CssToken::Whitespace,
        CssToken::OpenBrace,
        CssToken::Whitespace,
        CssToken::Identifier("color".to_owned()),
        CssToken::Colon,
        CssToken::Identifier("red".to_owned()),
        CssToken::Semicolon,
        CssToken::Whitespace,
        CssToken::CloseBrace,
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
        CssToken::Identifier("p".to_owned()),
        CssToken::Whitespace,
        CssToken::OpenBrace,
        CssToken::Whitespace,
        CssToken::Identifier("color".to_owned()),
        CssToken::Colon,
        CssToken::Whitespace,
        CssToken::Identifier("red".to_owned()),
        CssToken::Semicolon,
        CssToken::Whitespace,
        CssToken::Identifier("h3".to_owned()),
        CssToken::Whitespace,
        CssToken::OpenBrace,
        CssToken::Whitespace,
        CssToken::Identifier("text-decoration".to_owned()),
        CssToken::Colon,
        CssToken::Whitespace,
        CssToken::Identifier("none".to_owned()),
        CssToken::Semicolon,
        CssToken::Whitespace,
        CssToken::CloseBrace,
        CssToken::CloseBrace,
    ];
    assert_eq!(tokens.len(), expected_tokens.len());

    for (token, expected_token) in tokens.iter().zip(expected_tokens.iter()) {
        assert_eq!(&token.css_token, expected_token);
    }
}


#[test]
fn test_multiple_selectors() {
    let css_text = "h3, h4 {color:red}";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);

    let expected_tokens = vec![
        CssToken::Identifier("h3".to_owned()),
        CssToken::Comma,
        CssToken::Whitespace,
        CssToken::Identifier("h4".to_owned()),
        CssToken::Whitespace,
        CssToken::OpenBrace,
        CssToken::Identifier("color".to_owned()),
        CssToken::Colon,
        CssToken::Identifier("red".to_owned()),
        CssToken::CloseBrace,
    ];
    assert_eq!(tokens.len(), expected_tokens.len());

    for (token, expected_token) in tokens.iter().zip(expected_tokens.iter()) {
        assert_eq!(&token.css_token, expected_token);
    }
}


#[test]
fn test_basic_combinator() {
    let css_text = "h3 > p{color:red}";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);

    let expected_tokens = vec![
        CssToken::Identifier("h3".to_owned()),
        CssToken::Whitespace,
        CssToken::Greater,
        CssToken::Whitespace,
        CssToken::Identifier("p".to_owned()),
        CssToken::OpenBrace,
        CssToken::Identifier("color".to_owned()),
        CssToken::Colon,
        CssToken::Identifier("red".to_owned()),
        CssToken::CloseBrace,
    ];
    assert_eq!(tokens.len(), expected_tokens.len());

    for (token, expected_token) in tokens.iter().zip(expected_tokens.iter()) {
        assert_eq!(&token.css_token, expected_token);
    }
}
