use std::iter::Peekable;
use std::slice::Iter;

use crate::debug::debug_log_warn;
use crate::style::css_lexer::{CssToken, CssTokenWithLocation};
use crate::style::{
    CssCombinator,
    CssProperty,
    Selector,
    StyleRule,
};


pub fn parse_css(css_tokens: &Vec<CssTokenWithLocation>) -> Vec<StyleRule> {
    let mut style_rules = Vec::new();
    let mut current_selector_context = Vec::new();
    let mut token_iterator = css_tokens.iter().peekable();

    parse_statements(&mut style_rules, &mut current_selector_context, &mut token_iterator);
    return style_rules;
}


fn parse_statements(style_rules: &mut Vec<StyleRule>, current_selector_context: &mut Vec<(CssCombinator, String)>,
                    token_iterator: &mut Peekable<Iter<CssTokenWithLocation>>) {
    while token_iterator.peek().is_some() {
        match token_iterator.peek().unwrap().css_token {
            CssToken::CloseBrace => return,
            _ => {},
        }

        parse_statement(style_rules, current_selector_context, token_iterator);
    }
}


fn parse_statement(style_rules: &mut Vec<StyleRule>, current_context: &mut Vec<(CssCombinator, String)>, token_iterator: &mut Peekable<Iter<CssTokenWithLocation>>) {
    if token_iterator.peek().is_some() {
        match &token_iterator.peek().unwrap().css_token {
            CssToken::AtKeyword(_) => {
                todo!(); //TODO: how we parse this exactly unfortunately depends on the kind of keywords, some have rulesets, others have not...
                         //      maybe don't check the keyword but just check for ; or { ... } ??
            },
            _ => {
                let selectors = parse_selectors(current_context, token_iterator);
                parse_declaration_block(selectors, style_rules, token_iterator);
            },
        }
    }
}


fn parse_selectors(current_selector_context: &mut Vec<(CssCombinator, String)>, token_iterator: &mut Peekable<Iter<CssTokenWithLocation>>) -> Vec<Selector> {
    let mut selectors = Vec::new();
    let mut selector_elements = current_selector_context.clone();
    let mut next_combinator = CssCombinator::None;
    let mut parsing_pseudoclass = false;
    let mut current_pseudoclasses = Vec::new();

    while token_iterator.peek().is_some() {
        match &token_iterator.peek().unwrap().css_token {
            CssToken::OpenBrace => {
                if !selector_elements.is_empty() {
                    selector_elements.reverse();
                    selectors.push(Selector { elements: selector_elements, pseudoclasses: Some(current_pseudoclasses) });
                }

                return selectors;
            },
            CssToken::Comma => {
                parsing_pseudoclass = false;
                token_iterator.next();

                if !selector_elements.is_empty() {
                    selector_elements.reverse();
                    selectors.push(Selector { elements: selector_elements, pseudoclasses: Some(current_pseudoclasses) });
                    selector_elements = current_selector_context.clone();
                    current_pseudoclasses = Vec::new();
                }
            },
            CssToken::Identifier(ident) => {
                token_iterator.next();

                if parsing_pseudoclass {
                    current_pseudoclasses.push(ident.clone());
                } else {
                    selector_elements.push( (next_combinator, ident.clone()) );
                }
            },
            CssToken::Whitespace => {
                parsing_pseudoclass = false;
                token_iterator.next();
                next_combinator = CssCombinator::Descendent;
            },
            CssToken::Greater => {
                token_iterator.next();
                next_combinator = CssCombinator::Child;
            },
            CssToken::Plus => {
                token_iterator.next();
                next_combinator = CssCombinator::NextSibling;
            },
            CssToken::Tilde => {
                token_iterator.next();
                next_combinator = CssCombinator::GeneralSibling;
            },
            CssToken::Colon => {
                token_iterator.next();

                if token_iterator.peek().is_some() {
                    match &token_iterator.peek().unwrap().css_token {
                        CssToken::Colon => {
                            //This is an pseudo-element
                            todo!();
                        },
                        _ => {
                            parsing_pseudoclass = true;
                        }
                    }
                }
            },
            _ => {
                todo!(); //TODO: this should be an error
            }
        }
    }

    return selectors;
}


fn parse_declaration_block(selectors: Vec<Selector>, style_rules: &mut Vec<StyleRule>, token_iterator: &mut Peekable<Iter<CssTokenWithLocation>>) {
    while token_iterator.peek().is_some() {
        match token_iterator.peek().unwrap().css_token {
            CssToken::OpenBrace => {
                token_iterator.next();
                break;
            },
            _ => {
                todo!(); //TODO: this should be an error
            }
        }
    }

    while token_iterator.peek().is_some() {
        match token_iterator.peek().unwrap().css_token {
            CssToken::CloseBrace => {
                token_iterator.next();
                return
            },
            _ => {
                let declaration = parse_declaration(token_iterator);
                if declaration.is_some() {
                    let (property, value) = declaration.unwrap();
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
    let mut parsed_value = None;

    while token_iterator.peek().is_some() {
        match &token_iterator.peek().unwrap().css_token {

            CssToken::Identifier(ident) => {
                token_iterator.next();
                if parsed_property.is_none() {
                    parsed_property = CssProperty::from_string(&ident);
                    if parsed_property.is_none() {
                        debug_log_warn(format!("Unknown css property: {}", ident));

                        //TODO: consume all until ; or {  (maybe others?)
                        todo!();
                    }
                } else if parsed_value.is_none() {
                    //TODO: can the value span more tokens?
                    parsed_value = Some(ident.clone());
                } else {
                    todo!(); //TODO: this should be an error (we should have returned already if both are filled)
                }
            },
            CssToken::Semicolon => {
                token_iterator.next();
                if parsed_property.is_some() && parsed_value.is_some() {
                    return Some( (parsed_property.unwrap(), parsed_value.unwrap()) );
                }
                return None;
            },
            CssToken::CloseBrace => {
                if parsed_property.is_some() && parsed_value.is_some() {
                    return Some( (parsed_property.unwrap(), parsed_value.unwrap()) );
                }
                return None;
            },
            CssToken::Colon => {
                token_iterator.next();
                if parsed_property.is_none() || parsed_value.is_some() {
                    todo!() //TODO: this should be an error
                }
            },
            CssToken::Whitespace => {
                token_iterator.next();
            },
            _ => {
                todo!(); //TODO: this should be an error
            }
        }
    }

    todo!(); //TODO: this should be an error
}
