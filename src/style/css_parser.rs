use std::iter::Peekable;
use std::slice::Iter;

use crate::debug::debug_log_warn;
use crate::style::css_lexer::{CssToken, CssTokenWithLocation};
use crate::style::{
    CssCombinator,
    CssFunction,
    CssProperty,
    CssValue,
    Selector,
    SelectorType,
    StyleRule,
};


pub fn parse_css(css_tokens: &Vec<CssTokenWithLocation>) -> Vec<StyleRule> {
    let mut style_rules = Vec::new();
    let mut current_selector_context = Vec::new();
    let mut token_iterator = css_tokens.iter().peekable();

    parse_statements(&mut style_rules, &mut current_selector_context, &mut token_iterator);
    return process_shorthands(style_rules);
}


fn process_shorthands(style_rules: Vec<StyleRule>) -> Vec<StyleRule> {
    let mut resolved_style_rules = Vec::new();

    for style_rule in style_rules {

        match style_rule.property {
            CssProperty::Flex => {

                //TODO: in reality, this is more complicated, since some of the 3 values can be omitted, but how that is determined
                //      depends on the values (wether they are percentages or numbers etc.)

                match style_rule.value {

                    CssValue::List(style_values) => {
                        let mut expecting_flex_grow = true;
                        let mut expecting_flex_shrink = true;
                        let mut expecting_flex_basis = true;

                        for part in style_values {
                            let property = if expecting_flex_grow {
                                expecting_flex_grow = false;
                                CssProperty::FlexGrow
                            } else if expecting_flex_shrink {
                                expecting_flex_shrink = false;
                                CssProperty::FlexShrink
                            } else if expecting_flex_basis {
                                expecting_flex_basis = false;
                                CssProperty::FlexBasis
                            } else {
                                break;
                            };

                            resolved_style_rules.push(StyleRule { selector: style_rule.selector.clone(), property, value: part });
                        }
                    },
                    CssValue::String(_) => {
                        todo!();
                    },
                    CssValue::Function(_) => {
                        todo!();
                    }
                }
            },
            _ => {
                resolved_style_rules.push(style_rule);
            }
        }
    }

    return resolved_style_rules;
}


fn parse_statements(style_rules: &mut Vec<StyleRule>, current_selector_context: &mut Vec<(CssCombinator, SelectorType, String)>,
                    token_iterator: &mut Peekable<Iter<CssTokenWithLocation>>) {
    while token_iterator.peek().is_some() {
        match token_iterator.peek().unwrap().css_token {
            CssToken::CloseBrace => return,
            _ => {},
        }

        parse_statement(style_rules, current_selector_context, token_iterator);
    }
}


fn parse_statement(style_rules: &mut Vec<StyleRule>, current_context: &mut Vec<(CssCombinator, SelectorType, String)>,
                   token_iterator: &mut Peekable<Iter<CssTokenWithLocation>>) {
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


fn parse_selectors(current_selector_context: &mut Vec<(CssCombinator, SelectorType, String)>,
                   token_iterator: &mut Peekable<Iter<CssTokenWithLocation>>)-> Vec<Selector> {
    let mut selectors = Vec::new();
    let mut selector_elements = current_selector_context.clone();
    let mut next_combinator = CssCombinator::None;
    let mut next_selector_type = SelectorType::Name;
    let mut parsing_pseudoclass = false;
    let mut current_pseudoclasses = None;

    while token_iterator.peek().is_some() {
        match &token_iterator.peek().unwrap().css_token {
            CssToken::OpenBrace => {
                if !selector_elements.is_empty() {
                    selector_elements.reverse();
                    selectors.push(Selector { elements: selector_elements, pseudoclasses: current_pseudoclasses });
                }

                return selectors;
            },
            CssToken::Comma => {
                parsing_pseudoclass = false;
                token_iterator.next();

                if !selector_elements.is_empty() {
                    selector_elements.reverse();
                    selectors.push(Selector { elements: selector_elements, pseudoclasses: current_pseudoclasses });
                    selector_elements = current_selector_context.clone();
                    current_pseudoclasses = None;
                }
            },
            CssToken::Identifier(ident) => {
                token_iterator.next();

                if parsing_pseudoclass {
                    if current_pseudoclasses.is_none() {
                        current_pseudoclasses = Some(Vec::new());
                    }

                    current_pseudoclasses.as_mut().unwrap().push(ident.clone());
                } else {
                    selector_elements.push( (next_combinator, next_selector_type, ident.clone()) );
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
            CssToken::Dot => {
                token_iterator.next();
                next_selector_type = SelectorType::Class;
            },
            CssToken::Hash => {
                token_iterator.next();
                next_selector_type = SelectorType::Id;
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


fn parse_declaration(token_iterator: &mut Peekable<Iter<CssTokenWithLocation>>) -> Option<(CssProperty, CssValue)> {
    let mut parsed_property = None;
    let mut parsed_value = None;

    while token_iterator.peek().is_some() {
        match &token_iterator.peek().unwrap().css_token {

            CssToken::Identifier(ident) => {
                if parsed_property.is_none() {
                    token_iterator.next();

                    parsed_property = CssProperty::from_string(&ident);
                    if parsed_property.is_none() {
                        debug_log_warn(format!("Unknown css property: {}", ident));

                        //Skip the value of this property
                        while token_iterator.peek().is_some() {
                            match &token_iterator.peek().unwrap().css_token {
                                CssToken::Semicolon => {
                                    token_iterator.next();
                                    break;
                                },
                                CssToken::CloseBrace => {
                                    break;
                                }
                                _ => {
                                    token_iterator.next();
                                },
                            }
                        }
                    }
                } else if parsed_value.is_none() {
                    parsed_value = Some(parse_value(token_iterator));
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


fn parse_value(token_iterator: &mut Peekable<Iter<CssTokenWithLocation>>) -> CssValue {
    let mut elements = Vec::new();

    while token_iterator.peek().is_some() {
        let first = token_iterator.next().unwrap();

        match &first.css_token {
            CssToken::Whitespace => {
                continue;
            },
            CssToken::Semicolon => {
                break;
            },
            _ => {}
        }

        let first_ident = match &first.css_token {
            CssToken::Identifier(ident) => ident,
            _ => todo!(), //TODO: this should be an error
        };

        let mut second = token_iterator.peek();
        if second.is_none() {
            return CssValue::String(first_ident.clone())
        }

        while second.is_some() {
            match &second.unwrap().css_token {
                CssToken::Whitespace | CssToken::Comma => {
                    token_iterator.next();
                    second = token_iterator.peek();
                },
                _ => {
                    break;
                }
            }
        }

        match &second.unwrap().css_token {
            CssToken::Identifier(_) => {
                elements.push(CssValue::String(first_ident.clone()));
            },
            CssToken::OpenParenthesis => {
                elements.push(parse_css_function(token_iterator, first_ident));
            },
            CssToken::CloseBrace | CssToken::Semicolon => {
                elements.push(CssValue::String(first_ident.clone()));
                break;
            }
            _ => {
                todo!(); //TODO: this needs to become an error
            }
        }
    }

    if elements.len() == 1 {
        return elements[0].clone()
    } else {
        return CssValue::List(elements);
    }
}

fn parse_css_function(token_iterator: &mut Peekable<Iter<CssTokenWithLocation>>, function_name: &String) -> CssValue {

    token_iterator.next(); //consume the (

    let mut arguments = Vec::new();


    while token_iterator.peek().is_some() {
        let first = token_iterator.next().unwrap();

        match &first.css_token {
            CssToken::Whitespace => {
                continue;
            },
            CssToken::CloseParenthesis => {
                break;
            },
            _ => {}
        }

        let first_ident = match &first.css_token {
            CssToken::Identifier(ident) => ident,
            _ => todo!(), //TODO: this should be an error
        };

        let mut second = token_iterator.peek();
        if second.is_none() {
            return CssValue::String(first_ident.clone())
        }

        while second.is_some() {
            match &second.unwrap().css_token {
                CssToken::Whitespace | CssToken::Comma => {
                    token_iterator.next();
                    second = token_iterator.peek();
                },
                _ => {
                    break;
                }
            }
        }

        match &second.unwrap().css_token {
            CssToken::Identifier(_) => {
                arguments.push(CssValue::String(first_ident.clone()));
            },
            CssToken::OpenParenthesis => {
                arguments.push(parse_css_function(token_iterator, first_ident));
            },
            CssToken::CloseParenthesis => {
                arguments.push(CssValue::String(first_ident.clone()));
                token_iterator.next(); //eat the ")"
                break;
            }
            CssToken::CloseBrace | CssToken::Semicolon => {
                arguments.push(CssValue::String(first_ident.clone()));
                if arguments.len() == 1 {
                    return arguments[0].clone()
                } else {
                    return CssValue::List(arguments);
                }
            }
            _ => {
                todo!(); //TODO: this needs to become an error
            }
        }
    }

    return CssValue::Function(CssFunction { name: function_name.clone(), arguments })
}
