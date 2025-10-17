use crate::style::css_lexer::{CssToken, CssTokenWithLocation};
use crate::style::CssProperty;
use crate::style::Selector;
use crate::style::StyleRule;


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
                    style_rules.push( StyleRule { selector: build_selector_from_context(&current_context),
                                                  property: last_property.unwrap(), value: value.to_string() } );
                }
            },
            CssToken::BlockStart => {
                // currently we have no logic for a block start, since we push the context for each selector, assuming we start a block after...
            },
            CssToken::BlockEnd => {
                current_context.pop();
            },
        }
    }

    return style_rules;
}


fn build_selector_from_context(context: &Vec<&String>) -> Selector {

    //TODO: we need to also parse combinators here

    let mut all_selectors = Vec::new();
    for selector in context {
        all_selectors.push((*selector).clone());
    }

    return Selector { elements: Some(all_selectors) }
}
