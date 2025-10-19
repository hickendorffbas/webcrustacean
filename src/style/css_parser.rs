use crate::style::css_lexer::{CssToken, CssTokenWithLocation};
use crate::style::{
    CssCombinator,
    CssProperty,
    Selector,
    StyleRule
};


pub fn parse_css(css_tokens: &Vec<CssTokenWithLocation>) -> Vec<StyleRule> {
    let mut style_rules = Vec::new();
    let mut current_context = Vec::new();
    let mut last_property = None;

    for token in css_tokens {

        match &token.css_token {
            CssToken::Selector(element) => {
                current_context.push(element);
            }
            CssToken::Property(property) => {
                last_property = CssProperty::from_string(&property.to_string());
            }
            CssToken::Value(value) => {
                if last_property.is_none() {
                    //TODO: do we need to log this error somewhere?
                } else {
                    for selector in build_selectors_from_context(&current_context) {
                        style_rules.push( StyleRule { selector, property: last_property.unwrap(), value: value.to_string() } );
                    }
                }
            },
            CssToken::BlockStart => {
                // currently we have no logic for a block start, since we push the context for each selector, assuming we start a block after...
            },
            CssToken::BlockEnd => {
                current_context.pop();
            },
            CssToken::DescendentCombinator => {
                todo!(); //TODO: implement
            },
            CssToken::ChildCombinator => {
                todo!(); //TODO: implement
            },
            CssToken::SubsequentSiblingCombinator =>  {
                todo!(); //TODO: implement
            },
            CssToken::NextSiblingCombinator =>  {
                todo!(); //TODO: implement
            },
            CssToken::Comma => {
                todo!(); //TODO: implement
            },
        }
    }

    return style_rules;
}


fn build_selectors_from_context(context: &Vec<&String>) -> Vec<Selector> {

    //TODO: we need to also parse combinators here
    //TODO: also, I want to split on comma's return selectors for each of those (comma is now a token)

    let mut all_selectors = Vec::new();
    for selector in context {
        all_selectors.push(((*selector).clone(), CssCombinator::None));
    }

    return vec![Selector { elements: Some(all_selectors) }];
}
