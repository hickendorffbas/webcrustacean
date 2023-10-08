use crate::style::{
    Selector,
    StyleRule,
    css_lexer::{CssToken, CssTokenWithLocation}
};


pub fn parse_css(css_tokens: &Vec<CssTokenWithLocation>) -> Vec<StyleRule> {
    let mut style_rules = Vec::new();
    let mut current_context = Vec::new();
    let mut last_property = "";

    for token in css_tokens {

        match &token.css_token {
            CssToken::Selector(element) => {
                current_context.push(element);
            }
            CssToken::Property(property) => {
                last_property = property;
            }
            CssToken::Value(value) => {
                style_rules.push( StyleRule { selector: build_selector_from_context(&current_context),
                                              property: last_property.to_string(), value: value.to_string() } );
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
    //TODO: eventually we need to parse other things than just nodes here...

    let mut all_selectors = Vec::new();
    for selector in context {
        all_selectors.push((*selector).clone());
    }

    return Selector { nodes: Some(all_selectors) }
}
