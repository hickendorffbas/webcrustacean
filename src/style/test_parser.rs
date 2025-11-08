use crate::style::{CssCombinator, CssProperty, SelectorType};

use super::css_lexer::{CssToken, CssTokenWithLocation};
use super::css_parser;



#[test]
fn test_parse_basic_style() {
    let tokens = vec![
        CssTokenWithLocation { css_token: CssToken::Identifier("h3".to_owned()), line: 1, character: 1 },
        CssTokenWithLocation { css_token: CssToken::OpenBrace, line: 1, character: 3 },
        CssTokenWithLocation { css_token: CssToken::Identifier("color".to_owned()), line: 1, character: 4 },
        CssTokenWithLocation { css_token: CssToken::Colon, line: 1, character: 6 },
        CssTokenWithLocation { css_token: CssToken::Identifier("red".to_owned()), line: 1, character: 8 },
        CssTokenWithLocation { css_token: CssToken::CloseBrace, line: 1, character: 13 },
    ];

    let result = css_parser::parse_css(&tokens);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].property, CssProperty::Color);
    assert_eq!(result[0].value, "red");
    assert_eq!(result[0].selector.elements[0], (CssCombinator::None, SelectorType::Name, "h3".to_owned()));
}


#[test]
fn test_parse_id_style() {
    let tokens = vec![
        CssTokenWithLocation { css_token: CssToken::Hash, line: 1, character: 1 },
        CssTokenWithLocation { css_token: CssToken::Identifier("ident".to_owned()), line: 1, character: 2 },
        CssTokenWithLocation { css_token: CssToken::OpenBrace, line: 1, character: 4 },
        CssTokenWithLocation { css_token: CssToken::Identifier("color".to_owned()), line: 1, character: 5 },
        CssTokenWithLocation { css_token: CssToken::Colon, line: 1, character: 7 },
        CssTokenWithLocation { css_token: CssToken::Identifier("red".to_owned()), line: 1, character: 9 },
        CssTokenWithLocation { css_token: CssToken::CloseBrace, line: 1, character: 14 },
    ];

    let result = css_parser::parse_css(&tokens);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].property, CssProperty::Color);
    assert_eq!(result[0].value, "red");
    assert_eq!(result[0].selector.elements[0], (CssCombinator::None, SelectorType::Id, "ident".to_owned()));
}
