use std::iter::Peekable;
use std::slice::Iter;

use crate::debug::debug_log_warn;
use crate::style::css_lexer::{CssToken, CssTokenWithLocation};
use crate::style::{
    CssCombinator,
    CssProperty,
    Selector,
    StyleRule
};


pub fn parse_css(css_tokens: &Vec<CssTokenWithLocation>) -> Vec<StyleRule> {
    let mut style_rules = Vec::new();
    let mut current_context = Vec::new();  //TODO: current context should be used to parse nested blocks
    let mut token_iterator = css_tokens.iter().peekable();

    parse_block_of_rules(&mut style_rules, &mut current_context, &mut token_iterator);
    return style_rules;
}


fn parse_block_of_rules(style_rules: &mut Vec<StyleRule>, current_context: &mut Vec<(CssCombinator, String)>, token_iterator: &mut Peekable<Iter<CssTokenWithLocation>>) {
    while token_iterator.peek().is_some() {
        match token_iterator.peek().unwrap().css_token {
            CssToken::BlockEnd => return,
            _ => {},
        }

        parse_rule(style_rules, current_context, token_iterator);
    }
}


fn parse_rule(style_rules: &mut Vec<StyleRule>, current_context: &mut Vec<(CssCombinator, String)>, token_iterator: &mut Peekable<Iter<CssTokenWithLocation>>) {
    if token_iterator.peek().is_some() {
        match token_iterator.peek().unwrap().css_token {
            CssToken::AtRule(_) => {
                parse_at_rule(token_iterator);
                return;
            }
            _ => {},
        }
    }

    let selectors = parse_selectors(current_context, token_iterator);
    parse_declaration_block(selectors, style_rules, token_iterator);
}


fn parse_selectors(current_context: &mut Vec<(CssCombinator, String)>, token_iterator: &mut Peekable<Iter<CssTokenWithLocation>>) -> Vec<Selector> {
    let mut selectors = Vec::new();
    let mut selector_elements = current_context.clone();
    let mut next_combinator = CssCombinator::None;

    while token_iterator.peek().is_some() {
        match &token_iterator.peek().unwrap().css_token {
            CssToken::BlockStart => {
                token_iterator.next();

                if !selector_elements.is_empty() {
                    selector_elements.reverse();
                    selectors.push(Selector { elements: selector_elements });
                }

                return selectors;
            },
            CssToken::Selector(selector_value) => {
                token_iterator.next();

                selector_elements.push( (next_combinator, selector_value.clone()) );

                match &token_iterator.peek().unwrap().css_token {

                    CssToken::DescendentCombinator => {
                        token_iterator.next();
                        next_combinator = CssCombinator::Descendent;
                    },
                    CssToken::ChildCombinator => {
                        token_iterator.next();
                        next_combinator = CssCombinator::Child;
                    },
                    CssToken::GeneralSiblingCombinator => {
                        token_iterator.next();
                        next_combinator = CssCombinator::GeneralSibling;
                    },
                    CssToken::NextSiblingCombinator => {
                        token_iterator.next();
                        next_combinator = CssCombinator::NextSibling;
                    },
                    _ => {
                        next_combinator = CssCombinator::None;
                    }
                }
            },
            CssToken::Comma => {
                token_iterator.next();

                if !selector_elements.is_empty() {
                    selector_elements.reverse();
                    selectors.push(Selector { elements: selector_elements });
                    selector_elements = current_context.clone();
                }
            },
            _ => {
                todo!(); //TODO: this should be an error
            }
        }
    }

    return selectors;
}


fn parse_at_rule(token_iterator: &mut Peekable<Iter<CssTokenWithLocation>>) {

    #[allow(unused)] //TODO: we don't do anything with the ruledata yet....
    let rule_data = match &token_iterator.next().unwrap().css_token {
        CssToken::AtRule(rule_data) => rule_data,
        _ => panic!("invalid state"),
    };

    match token_iterator.peek().unwrap().css_token {

        CssToken::BlockStart => {
            token_iterator.next();

            match token_iterator.peek().unwrap().css_token {
                CssToken::Selector(_) | CssToken::AtRule(_) => {
                    //TODO: for now we parse so we advance the iterator, but we don't use the parsed data yet
                    parse_block_of_rules(&mut Vec::new(), &mut Vec::new(), token_iterator);
                },
                CssToken::Property(_) => {
                    //TODO: for now we parse so we advance the iterator, but we don't use the parsed data yet
                    parse_declaration_block(Vec::new(), &mut Vec::new(), token_iterator);
                },
                CssToken::BlockEnd => {
                    //It was an empty block
                    token_iterator.next();
                    return;
                },
                _ => todo!(), //TODO: this should be an error
            }
        },
        CssToken::Selector(_) => {
            //This means the rule data was all there was, go back to normal parsing
            return;
        },
        _ => todo!() //TODO: this should be an error
    }
}


fn parse_declaration_block(selectors: Vec<Selector>, style_rules: &mut Vec<StyleRule>, token_iterator: &mut Peekable<Iter<CssTokenWithLocation>>) {
    while token_iterator.peek().is_some() {
        match token_iterator.peek().unwrap().css_token {
            CssToken::BlockEnd => {
                token_iterator.next();
                return
            },
            _ => {
                let delcaration = parse_declaration(token_iterator);
                if delcaration.is_some() {
                    let (property, value) = delcaration.unwrap();
                    for selector in &selectors {
                        style_rules.push(StyleRule { selector: selector.clone(), property, value: value.clone() });
                    }
                }
            }
        }
    }
}


fn parse_declaration(token_iterator: &mut Peekable<Iter<CssTokenWithLocation>>) -> Option<(CssProperty, String)> {
    let mut parsed_property = None;

    while token_iterator.peek().is_some() {
        match &token_iterator.next().unwrap().css_token {
            CssToken::Property(property) => {
                if parsed_property.is_none() {
                    parsed_property = CssProperty::from_string(&property);
                    if parsed_property.is_none() {
                        debug_log_warn(format!("Unknown css property: {}", property));

                        match token_iterator.peek().unwrap().css_token {
                            CssToken::Value(_) => {
                                //Eat the possible value after the unknown property
                                token_iterator.next();
                            },
                            _ => {}
                        }
                        return None;
                    }
                } else {
                    todo!(); //TODO: this should be an error
                }
            },
            CssToken::Value(value) => {
                if parsed_property.is_none() {
                    todo!(); //TODO: this should be an error
                } else {
                    return Some((parsed_property.unwrap(), value.clone()));
                }
            },
            _ => {
                todo!(); //TODO: this should be an error
            }
        }
    }

    todo!(); //TODO: this should be an error
}
