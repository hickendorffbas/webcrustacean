use crate::css_lexer::{CssTokenWithLocation, CssToken};
use crate::style::{StyleRule, Selector};


pub fn parse_css(css_tokens: &Vec<CssTokenWithLocation>) -> Vec<StyleRule> {
    let mut style_rules = Vec::new();
    let mut current_context = Selector { nodes: None };
    let mut last_property = "";

    for token in css_tokens {

        match &token.css_token {
            CssToken::Selector(element) => {
                if current_context.nodes.is_none() {
                    current_context.nodes = Some(Vec::new());
                }

                let node_vec = current_context.nodes.as_mut().unwrap();
                node_vec.push(element.clone());

            }
            CssToken::Property(property) => {
                last_property = property;
            }
            CssToken::Value(value) => {
                style_rules.push( StyleRule { selector: current_context.clone(), property: last_property.to_string(), value: value.to_string() } );
            },
            CssToken::BlockStart => {
                //TODO: is there even something we need to do here? Maybe not. Should we then even emit the token?
            },
            CssToken::BlockEnd => {
                //TODO: somehow pop away current selector context (but not the context of higher levels)
            },
        }
    }

    return style_rules;
}
