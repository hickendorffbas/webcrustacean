use crate::style::CssProperty;

use super::css_lexer::{CssToken, CssTokenWithLocation};
use super::css_parser;



#[test]
fn test_parse_basic_style() {
    let tokens = vec![
        CssTokenWithLocation { css_token: CssToken::Selector("h3".to_owned()), line: 1, character: 1 },
        CssTokenWithLocation { css_token: CssToken::BlockStart, line: 1, character: 3 },
        CssTokenWithLocation { css_token: CssToken::Property("color".to_owned()), line: 1, character: 4 },
        CssTokenWithLocation { css_token: CssToken::Value("red".to_owned()), line: 1, character: 8 },
        CssTokenWithLocation { css_token: CssToken::BlockEnd, line: 1, character: 13 },
    ];

    let result = css_parser::parse_css(&tokens);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].property, CssProperty::Color);
    assert_eq!(result[0].value, "red");
    assert_eq!(result[0].selector.nodes.as_ref().unwrap()[0], "h3");
}
