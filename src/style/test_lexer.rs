use crate::style::css_lexer::{self, CssToken};


#[test]
fn test_parse_basic_style() {
    let css_text = "h3 { color: red; }";
    let tokens = css_lexer::lex_css(&css_text, 1, 1);
    assert_eq!(tokens.len(), 5);

    let expected_tokens = vec![
        CssToken::Selector("h3".to_owned()),
        CssToken::BlockStart,
        CssToken::Property("color".to_owned()),
        CssToken::Value("red".to_owned()),
        CssToken::BlockEnd,
    ];

    for (token, expected_token) in tokens.iter().zip(expected_tokens.iter()) {
        assert_eq!(&token.css_token, expected_token);
    }
}
