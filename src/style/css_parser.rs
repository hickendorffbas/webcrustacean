use crate::style::{
    Selector,
    StyleRule,
    css_lexer::{CssToken, CssTokenWithLocation}
};


pub fn parse_css(css_tokens: &Vec<CssTokenWithLocation>) -> Vec<StyleRule> {
    let mut style_rules = Vec::new();
    let mut current_context = vec![ Vec::new() ];
    let mut last_property = "";

    for token in css_tokens {

        match &token.css_token {
            CssToken::Selector(element) => {
                let last_idx = current_context.len() - 1;
                current_context[last_idx].push(element);
            }
            CssToken::Property(property) => {
                last_property = property;
            }
            CssToken::Value(value) => {
                style_rules.push( StyleRule { selector: build_selector_from_context(&current_context),
                                              property: last_property.to_string(), value: value.to_string() } );
            },
            CssToken::BlockStart => {
                current_context.push(Vec::new());
            },
            CssToken::BlockEnd => {
                current_context.pop(); //clear context(s) from inside the block
                let last_idx = current_context.len() - 1;
                current_context[last_idx].clear(); //reset the current context (the one parsed before entering the block)
            },
        }
    }

    return style_rules;
}


fn build_selector_from_context(context: &Vec<Vec<&String>>) -> Selector {
    //TODO: eventually we need to parse other things than just nodes here...

    let mut all_nodes = Vec::new();
    for nodes in context {
        for node in nodes {
            all_nodes.push((*node).clone())
        }
    }

    return Selector { nodes: Some(all_nodes) }
}
