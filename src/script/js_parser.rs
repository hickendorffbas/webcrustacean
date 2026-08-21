use std::rc::Rc;

use crate::network::url::Url;
use crate::script::js_ast::*;
use crate::script::js_console;
use crate::script::js_lexer::{JsToken, JsTokenWithLocation};


struct ParserState {
    cursor: usize,  //cursor points at the next token to read
    number_of_tokens: usize,
    source_url: Url,
}
impl ParserState {
    fn has_ended(&self) -> bool {
        return self.cursor >= self.number_of_tokens;
    }
    fn next(&mut self) {
        self.cursor += 1;
    }
}


pub enum ParseResult<T> {
    Ok(T),
    ParsingFailed(ParseError),
}
pub struct ParseError {
    error_type: ParseErrorType,
    line: u32,
    character: u32,
    url: Url,
}
pub enum ParseErrorType {
    EOF,
    IdentierExpected,
    ExpectedEndOfArgumentList,
}
impl ParseError {
    fn error_message(&self) -> String {
        match self.error_type {
            ParseErrorType::EOF => format!("Unexpected end of script at {}:{} ({})", self.line, self.character, self.url.to_string()),
            ParseErrorType::IdentierExpected => format!("Identifier expected at {}:{} ({})", self.line, self.character, self.url.to_string()),
            ParseErrorType::ExpectedEndOfArgumentList => format!("Expected end of argument list at {}:{} ({})", self.line, self.character, self.url.to_string()),
        }
    }
    fn error_for_token(error_type: ParseErrorType, tokens: &Vec<JsTokenWithLocation>, parser_state: &ParserState) -> ParseError {
        return ParseError { error_type, line: tokens[parser_state.cursor].line, character: tokens[parser_state.cursor].character, url: parser_state.source_url.clone() }
    }
}


pub fn parse_js(tokens: &Vec<JsTokenWithLocation>, source_url: &Url) -> Script {

    let mut parser_state = ParserState { cursor: 0, number_of_tokens: tokens.len(), source_url: source_url.clone() };
    let mut statements = Vec::new();

    while !parser_state.has_ended() {
        let possible_statement = parse_statement(tokens, &mut parser_state);
        if possible_statement.is_some() {
            match possible_statement.unwrap() {
                ParseResult::Ok(node) => statements.push(node),
                ParseResult::ParsingFailed(error) => {
                    js_console::log_js_error(&error.error_message());
                    return Vec::new();
                }
            }
        }
    }

    return statements;
}


fn parse_statement(tokens: &Vec<JsTokenWithLocation>, parser_state: &mut ParserState) -> Option<ParseResult<JsAstStatement>> {

    eat_newlines(tokens, parser_state);
    if parser_state.has_ended() {
        return None;
    }

    match &tokens[parser_state.cursor].token {
        JsToken::KeyWordFunction => {
            parser_state.next();
            return match parse_function_declaration(tokens, parser_state) {
                ParseResult::Ok(ast) => Some(ParseResult::Ok(JsAstStatement::FunctionDeclaration(ast))),
                ParseResult::ParsingFailed(error) => Some(ParseResult::ParsingFailed(error)),
            }
        },
        JsToken::KeyWordReturn => {
            parser_state.next();
            return match pratt_parse_expression(tokens, parser_state, 0, false, true) {
                ParseResult::Ok(expr) => Some(ParseResult::Ok(JsAstStatement::Return(expr))),
                ParseResult::ParsingFailed(parse_error) => Some(ParseResult::ParsingFailed(parse_error)),
            }
        },
        JsToken::KeyWordIf => {
            parser_state.next();
            return match parse_conditional(tokens, parser_state, true) {
                ParseResult::Ok(ast) => Some(ParseResult::Ok(ast)),
                ParseResult::ParsingFailed(error) => Some(ParseResult::ParsingFailed(error)),
            }
        },
        JsToken::KeyWordWhile => {
            parser_state.next();
            return match parse_while_loop(tokens, parser_state, true) {
                ParseResult::Ok(ast) => Some(ParseResult::Ok(ast)),
                ParseResult::ParsingFailed(error) => Some(ParseResult::ParsingFailed(error)),
            }
        },
        JsToken::KeyWordFor => {
            parser_state.next();
            return match parse_for_loop(tokens, parser_state) {
                ParseResult::Ok(ast) => Some(ParseResult::Ok(ast)),
                ParseResult::ParsingFailed(error) => Some(ParseResult::ParsingFailed(error)),
            }
        }
        decl_keyword @ (JsToken::KeyWordVar | JsToken::KeyWordLet | JsToken::KeyWordConst) => {
            parser_state.next();

            let decl_type = match decl_keyword {
                JsToken::KeyWordVar => { DeclType::Var },
                JsToken::KeyWordLet => { DeclType::Let },
                JsToken::KeyWordConst => { DeclType::Const },
                _ => { panic!("unreachable"); }
            };

            let result = match parse_declaration(tokens, parser_state, decl_type, true) {
                ParseResult::Ok(ast) => Some(ParseResult::Ok(JsAstStatement::Declaration(ast))),
                ParseResult::ParsingFailed(error) => return Some(ParseResult::ParsingFailed(error)),
            };

            match &tokens[parser_state.cursor].token {
                JsToken::Semicolon => {
                    parser_state.next();
                },
                _ => {
                    todo!(); //TODO: This probably should be an error, but needs checking
                },
            }

            return result;
        },
        JsToken::Semicolon => {
            parser_state.next();
            return None;
        }
        _ => {},
    }

    let expression_result = pratt_parse_expression(tokens, parser_state, 0, false, true);
    match expression_result {
        ParseResult::Ok(result) => {
            match &tokens[parser_state.cursor].token {
                JsToken::Semicolon => {
                    parser_state.next();
                },
                _ => {},
            }
            return Some(ParseResult::Ok(JsAstStatement::Expression(result)));
        }
        ParseResult::ParsingFailed(error) => return Some(ParseResult::ParsingFailed(error)),
    }
}


fn pratt_parse_expression(tokens: &Vec<JsTokenWithLocation>, parser_state: &mut ParserState, min_binding_power: u8,
                          needs_assignment_expression: bool, in_is_allowed: bool) -> ParseResult<JsAstExpression> {

    let mut lhs = match parse_expression_prefix(tokens, parser_state, in_is_allowed) {
        ParseResult::Ok(result) => result,
        ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
    };

    loop {
        if parser_state.has_ended() {
            return ParseResult::ParsingFailed(ParseError::error_for_token(ParseErrorType::EOF, tokens, parser_state))
        };

        match (&tokens[parser_state.cursor].token, needs_assignment_expression) {
            (JsToken::Semicolon, _)    | (JsToken::CloseParenthesis, _) | (JsToken::CloseBrace, _) |
            (JsToken::CloseBracket, _) | (JsToken::Comma, true)         | (JsToken::Colon, _) => {
                //we can pop back to the previous level of parsing:
                break;
            },
            (JsToken::Newline, _) => {
                //TODO: here something might need to happen wrt to deciding if we should insert a semicolon (stop parsing the statement)
                parser_state.next();
                continue;
            },

            //postfix operator parsing:
            (JsToken::OpenBracket, _) => {
                parser_state.next();

                let index_node = match pratt_parse_expression(tokens, parser_state, min_binding_power, true, in_is_allowed) {
                    ParseResult::Ok(index_expression) => JsAstExpression::BinOp(JsAstBinOp {
                        op: JsBinOp::PropertyAccess, left: Rc::from(lhs), right: Rc::from(index_expression), is_dot_property_access: false
                    }),
                    ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
                };

                match &tokens[parser_state.cursor].token {
                    JsToken::CloseBracket => parser_state.next(),
                    _ => todo!(), //TODO: this should be an error
                }

                lhs = index_node;
                continue;
            }

            _ => {},
        }

        let (left_bp, right_bp) = infix_binding_power(&tokens[parser_state.cursor].token);

        if left_bp < min_binding_power {
            break;
        }

        match &tokens[parser_state.cursor].token {

            JsToken::Dot => {
                parser_state.next();
                match &tokens[parser_state.cursor].token {
                    JsToken::Identifier(ident) => {
                        parser_state.next();
                        lhs = JsAstExpression::BinOp(JsAstBinOp { op: JsBinOp::PropertyAccess, left: Rc::from(lhs),
                                                                  right: Rc::from(JsAstExpression::Identifier(JsAstIdentifier { name: ident.clone() })),
                                                                  is_dot_property_access: true });
                    },
                    _ => {
                        return ParseResult::ParsingFailed(ParseError::error_for_token(ParseErrorType::IdentierExpected, tokens, parser_state));
                    }
                }
            },

            JsToken::Assign => {
                parser_state.next();

                let rhs = match pratt_parse_expression(tokens, parser_state, right_bp, true, in_is_allowed) {
                    ParseResult::Ok(rhs) => rhs,
                    ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
                };
                lhs = JsAstExpression::Assignment(JsAstAssign { left: Rc::from(lhs), right: Rc::from(rhs) });
            },

            compound @ (JsToken::CompoundAssignAdd | JsToken::CompoundAssignMinus | JsToken::CompoundAssignTimes | JsToken::CompoundAssignDiv |
                        JsToken::CompoundAssignBitWiseOr | JsToken::CompoundAssignBitWiseXor | JsToken::CompoundAssignBitWiseAnd) => {
                parser_state.next();

                let rhs = match pratt_parse_expression(tokens, parser_state, right_bp, true, in_is_allowed) {
                    ParseResult::Ok(rhs) => rhs,
                    ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
                };

                let op = match compound {
                    JsToken::CompoundAssignAdd        => JsBinOp::Plus,
                    JsToken::CompoundAssignMinus      => JsBinOp::Minus,
                    JsToken::CompoundAssignTimes      => JsBinOp::Times,
                    JsToken::CompoundAssignDiv        => JsBinOp::Divide,
                    JsToken::CompoundAssignBitWiseOr  => JsBinOp::BitWiseOr,
                    JsToken::CompoundAssignBitWiseXor => JsBinOp::BitWiseXor,
                    JsToken::CompoundAssignBitWiseAnd => JsBinOp::BitWiseAnd,
                    _ => panic!("This should never happen"),
                };

                let lhs_rc = Rc::from(lhs);
                let rhs = JsAstExpression::BinOp(JsAstBinOp { op, left: lhs_rc.clone(), right: Rc::from(rhs), is_dot_property_access: false });
                lhs = JsAstExpression::Assignment(JsAstAssign { left: lhs_rc, right: Rc::from(rhs) });
            },

            binop @ (JsToken::Plus | JsToken::Minus | JsToken::Star | JsToken::ForwardSlash | JsToken::LeftShift | JsToken::RightShift | JsToken::BitWiseXor |
                     JsToken::Equals | JsToken::EqualsStrict | JsToken::LogicalAnd | JsToken::LogicalOr | JsToken::BitWiseOr | JsToken::BitWiseAnd | JsToken::Comma |
                     JsToken::Bigger | JsToken::Smaller | JsToken::KeyWordIn | JsToken::Remainder) => {

                if matches!(binop, JsToken::KeyWordIn) && !in_is_allowed {
                    //There is situations where "in" is not allowed, for example on the left size of for(.... in ....)
                    return ParseResult::Ok(lhs);
                }

                parser_state.next();

                let rhs = match pratt_parse_expression(tokens, parser_state, right_bp, true, in_is_allowed) {
                    ParseResult::Ok(rhs) => rhs,
                    ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
                };

                let js_binop = match binop {
                    JsToken::Plus           => JsBinOp::Plus,
                    JsToken::Minus          => JsBinOp::Minus,
                    JsToken::Star           => JsBinOp::Times,
                    JsToken::ForwardSlash   => JsBinOp::Divide,
                    JsToken::Equals         => JsBinOp::Equals,
                    JsToken::EqualsStrict   => JsBinOp::EqualsStrict,
                    JsToken::LogicalAnd     => JsBinOp::LogicalAnd,
                    JsToken::LogicalOr      => JsBinOp::LogicalOr,
                    JsToken::BitWiseOr      => JsBinOp::BitWiseOr,
                    JsToken::BitWiseXor     => JsBinOp::BitWiseXor,
                    JsToken::BitWiseAnd     => JsBinOp::BitWiseAnd,
                    JsToken::Comma          => JsBinOp::Comma,
                    JsToken::LeftShift      => JsBinOp::LeftShift,
                    JsToken::RightShift     => JsBinOp::RightShift,
                    JsToken::Bigger         => JsBinOp::Bigger,
                    JsToken::Smaller        => JsBinOp::Smaller,
                    JsToken::KeyWordIn      => JsBinOp::In,
                    JsToken::Remainder      => JsBinOp::Remainder,
                    _ => panic!("This should never happen"),
                };

                lhs = JsAstExpression::BinOp(JsAstBinOp { op: js_binop, left: Rc::from(lhs), right: Rc::from(rhs), is_dot_property_access: false });
            },

            JsToken::OpenParenthesis => {
                parser_state.next();

                let arguments = match parse_list_of_expressions(tokens, parser_state, in_is_allowed) {
                    ParseResult::Ok(arguments) => arguments,
                    ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
                };
                lhs = JsAstExpression::FunctionCall(JsAstFunctionCall { function_expression: Rc::from(lhs), arguments });
            },

            JsToken::QuestionMark => {
                parser_state.next();

                let if_true_node = match pratt_parse_expression(tokens, parser_state, right_bp, true, in_is_allowed) {
                    ParseResult::Ok(if_true_expression) => Rc::from(if_true_expression),
                    ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
                };

                match &tokens[parser_state.cursor].token {
                    JsToken::Colon => {
                        parser_state.next();
                    },
                    _ => {
                        todo!(); //TODO: some kind of error
                    },
                }

                let if_false_node = match pratt_parse_expression(tokens, parser_state, right_bp, true, in_is_allowed) {
                    ParseResult::Ok(if_false_expression) => Rc::from(if_false_expression),
                    ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
                };

                lhs = JsAstExpression::Ternary(JsAstTernary { condition: Rc::from(lhs), if_true: if_true_node, if_false: if_false_node });
            },

            JsToken::Increment => {
                parser_state.next();
                lhs = JsAstExpression::UnaryOp(JsAstUnOp { op: JsUnOp::PostfixIncrement, operand: Rc::from(lhs) });
            },
            JsToken::Decrement => {
                parser_state.next();
                lhs = JsAstExpression::UnaryOp(JsAstUnOp { op: JsUnOp::PostfixDecrement, operand: Rc::from(lhs) });
            },

            _ => todo!(),
        }
    }

    return ParseResult::Ok(lhs);
}


fn parse_expression_prefix(tokens: &Vec<JsTokenWithLocation>, parser_state: &mut ParserState, in_is_allowed: bool) -> ParseResult<JsAstExpression> {

    eat_newlines(tokens, parser_state); //TODO: not sure if this is always correct, given semicolon insertion
    if parser_state.has_ended() {
        return ParseResult::ParsingFailed(ParseError::error_for_token(ParseErrorType::EOF, tokens, parser_state));
    }

    match &tokens[parser_state.cursor].token {

        operator @ (JsToken::Minus | JsToken::Plus | JsToken::ExclamationMark) => {
            //These are the unary operators
            parser_state.next();

            let right_bp = prefix_binding_power(operator);

            match pratt_parse_expression(tokens, parser_state, right_bp, true, in_is_allowed) {
                ParseResult::Ok(rhs) => {
                    let un_op = match operator {
                        JsToken::Minus => JsUnOp::Minus,
                        JsToken::Plus => JsUnOp::Plus,
                        JsToken::ExclamationMark => JsUnOp::Not,
                        _ => panic!("unreachable"),
                    };
                    return ParseResult::Ok(JsAstExpression::UnaryOp(JsAstUnOp { op: un_op, operand: Rc::from(rhs) }))
                }
                ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
            };
        },

        JsToken::Number(literal_number) => {
            parser_state.next();
            return ParseResult::Ok(JsAstExpression::NumericLiteral(literal_number.clone()));
        },
        JsToken::LiteralString(literal_string) => {
            parser_state.next();
            return ParseResult::Ok(JsAstExpression::StringLiteral(literal_string.clone()));
        },
        JsToken::LiteralBoolean(boolean_value) => {
            parser_state.next();
            return ParseResult::Ok(JsAstExpression::BooleanLiteral(*boolean_value));
        },
        JsToken::Identifier(ident) => {
            parser_state.next();
            return ParseResult::Ok(JsAstExpression::Identifier(JsAstIdentifier { name: ident.clone() }));
        },
        JsToken::OpenBrace => { //This is an object literal
            parser_state.next();

            let mut members = Vec::new();
            let mut first = true;
            let mut current_property_name;
            loop {

                match &tokens[parser_state.cursor].token {
                    JsToken::CloseBrace => {
                        parser_state.next();
                        break;
                    },
                    JsToken::Comma => {
                        if first {
                            todo!(); //TODO: this should be an error
                        }
                        parser_state.next();
                    },
                    _ => {
                        if !first {
                            todo!(); //TODO: this should be an error, because we expect a comma
                        }
                        //the first time we don't expect a comma, so we just don't do anything here
                    },
                }

                eat_newlines(tokens, parser_state);

                match &tokens[parser_state.cursor].token {
                    JsToken::Identifier(property_name) => {
                        parser_state.next();
                        current_property_name = property_name;
                    },
                    JsToken::LiteralString(property_name) => {
                        parser_state.next();
                        current_property_name = property_name;
                    },
                    JsToken::CloseBrace => { //This is possible due to allowing trailing comma's
                        parser_state.next();
                        break;
                    },
                    _ => {
                        todo!(); //TODO: are there any valid cases for this?
                    },
                }

                match &tokens[parser_state.cursor].token {
                    JsToken::Colon => {
                        parser_state.next();
                    },
                    _ => {
                        todo!(); //TODO: handle the case where a shorthand is used (i.e. {a} to mean { a : a })
                    },
                }

                match pratt_parse_expression(tokens, parser_state, 0, true, in_is_allowed) {
                    ParseResult::Ok(expression) => members.push((JsAstExpression::StringLiteral(current_property_name.clone()), expression)),
                    ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
                }

                first = false;
            }
            return ParseResult::Ok(JsAstExpression::ObjectLiteral(JsAstObjectLiteral { members: members }));
        },
        JsToken::OpenBracket => { // This is an array Literal
            parser_state.next();

            let mut elements = Vec::new();
            let mut first = true;
            loop {

                match &tokens[parser_state.cursor].token {
                    JsToken::CloseBracket => {
                        parser_state.next();
                        break;
                    },
                    JsToken::Comma => {
                        if first {
                            todo!(); //TODO: this should be an error
                        }
                        parser_state.next();
                    },
                    _ => {
                        if !first {
                            todo!(); //TODO: this should be an error, because we expect a comma
                        }
                        //the first time we don't expect a comma, so we just don't do anything here
                    }
                }

                match pratt_parse_expression(tokens, parser_state, 0, true, in_is_allowed) {
                    ParseResult::Ok(expression) => elements.push(expression),
                    ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
                }

                first = false;
            }
            return ParseResult::Ok(JsAstExpression::ArrayLiteral(JsAstArrayLiteral { elements: elements }));
        },
        JsToken::KeyWordFunction => {  //(anonymous) functions can also be an expression in JS
            parser_state.next();

            //TODO: epression functions are also allowed to not be anonymous....

            match &tokens[parser_state.cursor].token {
                JsToken::OpenParenthesis => {
                    parser_state.next();
                },
                _ => {
                    todo!(); //TODO: this should be an error (function arguments expected)
                }
            }

            let mut arguments = Vec::new();
            let mut first = true;
            loop {

                match &tokens[parser_state.cursor].token {
                    JsToken::Identifier(ident) => {
                        parser_state.next();
                        arguments.push(JsAstIdentifier { name: ident.clone() });
                    },
                    JsToken::CloseParenthesis => {
                        if first {
                            parser_state.next();
                            break;
                        } else {
                            todo!(); //TODO: some kind of error
                        }
                    },
                    _ => {
                        todo!(); //TODO: some kind of error
                    }
                }

                match &tokens[parser_state.cursor].token {
                    JsToken::Comma => {
                        parser_state.next();
                    },
                    JsToken::CloseParenthesis => {
                        parser_state.next();
                        break;
                    }
                    _ => {
                        todo!(); //TODO: this should be an error (function arguments expected)
                    },
                }

                first = false;
            }

            match &tokens[parser_state.cursor].token {
                JsToken::OpenBrace => {
                    parser_state.next();
                },
                _ => {
                    todo!(); //TODO: some kind of error
                },
            }

            let script = match parse_script(tokens, parser_state) {
                ParseResult::Ok(script) => script,
                ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
            };

            return ParseResult::Ok(JsAstExpression::FunctionExpression(JsAstFunctionExpression { name: None, arguments, script: Rc::from(script) }));
        },
        JsToken::KeyWordNew => {
            parser_state.next();
            eat_newlines(tokens, parser_state);

            //To use something with "new" it needs to be "constructable". For now we are happy with any function. However, we do already
            //take the parsing rules into account. i.e. as opposed to function parsing, "new" has higher precedence than "." , so "new a.b()"
            //will make a new a, and call b on it.

            let function_expression = Rc::from(match &tokens[parser_state.cursor].token {
                JsToken::OpenParenthesis => {
                    parser_state.next();
                    match pratt_parse_expression(tokens, parser_state, 0, true, in_is_allowed) {
                        ParseResult::Ok(expression) => expression,
                        ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
                    }
                },
                JsToken::Identifier(ident) => {
                    parser_state.next();
                    JsAstExpression::Identifier(JsAstIdentifier { name: ident.clone() } )
                },
                _ => {
                    todo!(); //TODO: this should be an error
                }
            });

            let arguments = match &tokens[parser_state.cursor].token {
                JsToken::OpenParenthesis => {
                    parser_state.next();
                    match parse_list_of_expressions(tokens, parser_state, in_is_allowed) {
                        ParseResult::Ok(arguments) => arguments,
                        ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
                    }
                },
                _ => {
                    //Arguments in object construction are optional (if the constructor has no arguments)
                    Vec::new()
                }
            };

            return ParseResult::Ok(JsAstExpression::ObjectCreation(JsAstObjectCreation { constructor: JsAstFunctionCall { function_expression, arguments } }));
        }
        JsToken::OpenParenthesis => {
            parser_state.next();

            match pratt_parse_expression(tokens, parser_state, 0, false, in_is_allowed) {
                ParseResult::Ok(expression) => {

                    match &tokens[parser_state.cursor].token {
                        JsToken::CloseParenthesis => {
                            parser_state.next();
                            return ParseResult::Ok(expression);
                        },
                        _ => todo!()
                    }
                },
                ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
            }
        },
        JsToken::RegexLiteral(regex_literal) => {
            parser_state.next();
            return ParseResult::Ok(JsAstExpression::RegexLiteral(JsAstRegexLiteral { regex: regex_literal.clone() }));
        },
        JsToken::KeyWordTypeOf => {
            parser_state.next();

            match pratt_parse_expression(tokens, parser_state, 0, true, in_is_allowed) {
                ParseResult::Ok(expression) => return ParseResult::Ok(JsAstExpression::TypeOf(JsAstTypeOf { expression: Rc::from(expression) })),
                ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
            }
        }
        _ => todo!(),
    }
}


fn parse_list_of_expressions(tokens: &Vec<JsTokenWithLocation>, parser_state: &mut ParserState, in_is_allowed: bool) -> ParseResult<Vec<JsAstExpression>> {
    let mut arguments = Vec::new();
    let mut first = true;

    loop {
        if parser_state.has_ended() {
            return ParseResult::ParsingFailed(ParseError::error_for_token(ParseErrorType::EOF, tokens, parser_state))
        };

        match &tokens[parser_state.cursor].token {
            JsToken::CloseParenthesis => {
                parser_state.next();
                break;
            },
            JsToken::Comma => {
                if first {
                    todo!(); //TODO: raise a parsing failed error
                }
                parser_state.next();
            },
            _ => {
                if !first {
                    return ParseResult::ParsingFailed(ParseError::error_for_token(ParseErrorType::ExpectedEndOfArgumentList, tokens, parser_state))
                }
            },
        }
        match pratt_parse_expression(tokens, parser_state, 0, true, in_is_allowed) {
            ParseResult::Ok(expression) => {
                arguments.push(expression);
            },
            ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
        }
        first = false;
    }

    return ParseResult::Ok(arguments);
}


fn prefix_binding_power(token: &JsToken) -> u8 {
    match token {
        JsToken::Plus => 99,
        JsToken::Minus => 99,
        JsToken::ExclamationMark => 99,
        _ => todo!(),
    }
}


fn infix_binding_power(token: &JsToken) -> (u8, u8) {
    match token {
        JsToken::Comma => (0, 1),
        JsToken::Assign => (2, 1),
        JsToken::CompoundAssignAdd => (2, 1),
        JsToken::CompoundAssignMinus => (2, 1),
        JsToken::CompoundAssignTimes => (2, 1),
        JsToken::CompoundAssignDiv => (2, 1),
        JsToken::CompoundAssignBitWiseOr => (2, 1),
        JsToken::CompoundAssignBitWiseXor => (2, 1),
        JsToken::CompoundAssignBitWiseAnd => (2, 1),
        JsToken::QuestionMark => (3, 2),
        JsToken::LogicalOr => (4, 5),
        JsToken::LogicalAnd => (6, 7),
        JsToken::BitWiseOr => (8, 9),
        JsToken::BitWiseXor => (10, 11),
        JsToken::BitWiseAnd => (12, 13),
        JsToken::Equals => (14, 15),
        JsToken::EqualsStrict => (14, 15),
        JsToken::Bigger => (16, 17),
        JsToken::Smaller => (16, 17),
        JsToken::KeyWordIn => (16, 17),
        JsToken::LeftShift => (18, 19),
        JsToken::RightShift => (18, 19),
        JsToken::Plus => (20, 21),
        JsToken::Minus => (20, 21),
        JsToken::Star => (22, 23),
        JsToken::ForwardSlash => (22, 23),
        JsToken::Remainder => (22, 23),
        JsToken::Dot => (100, 101),
        JsToken::OpenParenthesis => (100, 101),
        JsToken::Increment => (110, 111),
        JsToken::Decrement => (110, 111),
        _ => todo!(),
    }
}


fn parse_function_declaration(tokens: &Vec<JsTokenWithLocation>, parser_state: &mut ParserState) -> ParseResult<JsAstFunctionDeclaration> {
    let function_name = match &tokens[parser_state.cursor].token {
        JsToken::Identifier(ident) => {
            parser_state.next();
            ident
        },
        _ => {
            todo!(); //TODO: this should probably always be a "function name expected" error
        }
    };

    match &tokens[parser_state.cursor].token {
        JsToken::OpenParenthesis => {
            parser_state.next();
        },
        _ => {
            todo!(); //TODO: this should be an error (function arguments expected)
        }
    }

    let mut arguments = Vec::new();
    let mut first = true;
    loop {

        match &tokens[parser_state.cursor].token {
            JsToken::Identifier(ident) => {
                parser_state.next();
                arguments.push(JsAstIdentifier { name: ident.clone() });
            },
            JsToken::CloseParenthesis => {
                if first {
                    parser_state.next();
                    break;
                } else {
                    todo!(); //TODO: some kind of error
                }
            },
            _ => {
                todo!(); //TODO: some kind of error
            }
        }

        match &tokens[parser_state.cursor].token {
            JsToken::Comma => {
                parser_state.next();
            },
            JsToken::CloseParenthesis => {
                parser_state.next();
                break;
            }
            _ => {
                todo!(); //TODO: this should be an error (function arguments expected)
            },
        }

        first = false;
    }

    match &tokens[parser_state.cursor].token {
        JsToken::OpenBrace => {
            parser_state.next();
        },
        _ => {
            todo!(); //TODO: some kind of error
        },
    }

    let script = match parse_script(tokens, parser_state) {
        ParseResult::Ok(script) => script,
        ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
    };

    return ParseResult::Ok(JsAstFunctionDeclaration { name: function_name.clone(), arguments, script: Rc::from(script) })
}


fn parse_conditional(tokens: &Vec<JsTokenWithLocation>, parser_state: &mut ParserState, in_is_allowed: bool) -> ParseResult<JsAstStatement> {
    //TODO: javascript supports having a single statement without { } after if, we still need to add that

    match tokens[parser_state.cursor].token {
        JsToken::OpenParenthesis => {
            parser_state.next();
        },
        _ => {
            todo!(); //TODO: this should be an error
        }
    }

    let condition = match pratt_parse_expression(tokens, parser_state, 0, false, in_is_allowed) {
        ParseResult::Ok(expression) => expression,
        ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
    };

    match tokens[parser_state.cursor].token {
        JsToken::CloseParenthesis => {
            parser_state.next();
        },
        _ => {
            todo!(); //TODO: this should be an error
        }
    }
    match tokens[parser_state.cursor].token {
        JsToken::OpenBrace => {
            parser_state.next();
        },
        _ => {
            todo!(); //TODO: this should be an error (not in all cases, if we have a single statement after the if....)
        }
    }

    let script = match parse_script(tokens, parser_state) {
        ParseResult::Ok(script) => script,
        ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
    };

    eat_newlines(tokens, parser_state);

    let else_present = !parser_state.has_ended() &&
        match tokens[parser_state.cursor].token {
            JsToken::KeyWordElse => {
                parser_state.next();
                true
            }
            _ => { false }
    };

    let else_script;
    if else_present {

        match tokens[parser_state.cursor].token {
            JsToken::OpenBrace => {
                parser_state.next();
            },
            _ => {
                todo!(); //TODO: this should be an error (not in all cases, if we have a single statement after the if....)
            }
        }

        else_script = match parse_script(tokens, parser_state) {
            ParseResult::Ok(script) => Some(Rc::from(script)),
            ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
        };

    } else {
        else_script = None;
    }

    return ParseResult::Ok(JsAstStatement::Conditional(JsAstConditional { condition: Rc::from(condition), script: Rc::from(script), else_script }));
}


fn parse_while_loop(tokens: &Vec<JsTokenWithLocation>, parser_state: &mut ParserState, in_is_allowed: bool) -> ParseResult<JsAstStatement> {
    match tokens[parser_state.cursor].token {
        JsToken::OpenParenthesis => {
            parser_state.next();
        },
        _ => {
            todo!(); //TODO: this should be an error
        }
    }

    let condition = match pratt_parse_expression(tokens, parser_state, 0, false, in_is_allowed) {
        ParseResult::Ok(expression) => expression,
        ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
    };

    match tokens[parser_state.cursor].token {
        JsToken::CloseParenthesis => {
            parser_state.next();
        },
        _ => {
            todo!(); //TODO: this should be an error
        }
    }
    match tokens[parser_state.cursor].token {
        JsToken::OpenBrace => {
            parser_state.next();
        },
        _ => {
            todo!(); //TODO: this should be an error
        }
    }

    let script = match parse_script(tokens, parser_state) {
        ParseResult::Ok(script) => script,
        ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
    };

    eat_newlines(tokens, parser_state);

    return ParseResult::Ok(JsAstStatement::While(JsAstWhile { condition: Rc::from(condition), script: Rc::from(script) }));
}


fn parse_for_loop(tokens: &Vec<JsTokenWithLocation>, parser_state: &mut ParserState) -> ParseResult<JsAstStatement> {
    match tokens[parser_state.cursor].token {
        JsToken::OpenParenthesis => {
            parser_state.next();
        },
        _ => {
            todo!(); //TODO: this should be an error
        }
    }

    let (initial_declarations, initial_expression) = match &tokens[parser_state.cursor].token {
        decl_keyword @ (JsToken::KeyWordVar | JsToken::KeyWordLet | JsToken::KeyWordConst) => {
            parser_state.next();

            let decl_type = match decl_keyword {
                JsToken::KeyWordVar => { DeclType::Var },
                JsToken::KeyWordLet => { DeclType::Let },
                JsToken::KeyWordConst => { DeclType::Const },
                _ => { panic!("unreachable"); }
            };

            let initial_declarations = match parse_declaration(tokens, parser_state, decl_type, false) {
                ParseResult::Ok(ast) => Rc::from(ast),
                ParseResult::ParsingFailed(error) => return ParseResult::ParsingFailed(error),
            };

            (Some(initial_declarations), None)
        },

        _ => {
            let initial_expression = Rc::from(match pratt_parse_expression(tokens, parser_state, 0, false, false) {
                ParseResult::Ok(expression) => expression,
                ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
            });

            (None, Some(initial_expression))
        }
    };

    match &tokens[parser_state.cursor].token {
        JsToken::Semicolon => {
            parser_state.next();
        },
        JsToken::KeyWordIn => {
            parser_state.next();

            let iteration_target = Rc::from(match pratt_parse_expression(tokens, parser_state, 0, false, true) {
                ParseResult::Ok(expression) => expression,
                ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
            });

            match tokens[parser_state.cursor].token {
                JsToken::CloseParenthesis => {
                    parser_state.next();
                },
                _ => {
                    todo!(); //TODO: this should be an error
                }
            }

            match tokens[parser_state.cursor].token {
                JsToken::OpenBrace => {
                    parser_state.next();
                },
                _ => {
                    todo!(); //TODO: this should be an error
                }
            }

            let script = Rc::from(match parse_script(tokens, parser_state) {
                ParseResult::Ok(script) => script,
                ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
            });

            return ParseResult::Ok(JsAstStatement::ForEach(JsAstForEach { initial_expression, initial_declarations, iteration_target, script }));
        },
        _ => {
            todo!(); //TODO: this should be an error
        }
    }

    let loop_condition = Rc::from(match pratt_parse_expression(tokens, parser_state, 0, false, true) {
        ParseResult::Ok(expression) => expression,
        ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
    });

    match tokens[parser_state.cursor].token {
        JsToken::Semicolon => {
            parser_state.next();
        },
        _ => {
            todo!(); //TODO: this should be an error
        }
    }

    let next_step_expression = Rc::from(match pratt_parse_expression(tokens, parser_state, 0, false, true) {
        ParseResult::Ok(expression) => expression,
        ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
    });

    match tokens[parser_state.cursor].token {
        JsToken::CloseParenthesis => {
            parser_state.next();
        },
        _ => {
            todo!(); //TODO: this should be an error
        }
    }

    match tokens[parser_state.cursor].token {
        JsToken::OpenBrace => {
            parser_state.next();
        },
        _ => {
            todo!(); //TODO: this should be an error
        }
    }

    let script = Rc::from(match parse_script(tokens, parser_state) {
        ParseResult::Ok(script) => script,
        ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
    });

    eat_newlines(tokens, parser_state);

    return ParseResult::Ok(JsAstStatement::For(JsAstFor { initial_declarations, initial_expression, loop_condition, next_step_expression, script }));
}


fn parse_script(tokens: &Vec<JsTokenWithLocation>, parser_state: &mut ParserState) -> ParseResult<Script> {
    let mut script = Vec::new();
    loop {
        eat_newlines(tokens, parser_state);

        match tokens[parser_state.cursor].token {
            JsToken::CloseBrace => {
                parser_state.next();
                return ParseResult::Ok(script);
            },
            _ => {},
        }

        let statement = parse_statement(tokens, parser_state);
        if statement.is_some() {
            match statement.unwrap() {
                ParseResult::Ok(statement) => script.push(statement),
                ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
            }
        }

        eat_newlines(tokens, parser_state);
    }
}

fn eat_newlines(tokens: &Vec<JsTokenWithLocation>, parser_state: &mut ParserState) {
    while !parser_state.has_ended() {
        match tokens[parser_state.cursor].token {
            JsToken::Newline => {
                parser_state.next();
            }
            _ => { break; }
        }
    }
}


fn parse_declaration(tokens: &Vec<JsTokenWithLocation>, parser_state: &mut ParserState, decl_type: DeclType, in_is_allowed: bool) -> ParseResult<Vec<JsAstDeclaration>> {
    let mut declarations = Vec::new();

    loop {
        if parser_state.has_ended() { return ParseResult::ParsingFailed(ParseError::error_for_token(ParseErrorType::EOF, tokens, parser_state)) };

        let ident = match &tokens[parser_state.cursor].token {
            JsToken::Newline => {
                parser_state.next();
                continue;
            }
            JsToken::Identifier(ident) => {
                parser_state.next();
                JsAstIdentifier { name: ident.clone() }
            },
            _ => todo!(), //TODO: this should be an error
        };

        match tokens[parser_state.cursor].token {
            JsToken::Assign => {
                parser_state.next();
                match pratt_parse_expression(tokens, parser_state, 0, true, in_is_allowed) {
                    ParseResult::Ok(expression) => {
                        declarations.push(JsAstDeclaration { variable: ident, initial_value: Some(expression), decl_type });
                    },
                    ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
                };
                match tokens[parser_state.cursor].token {
                    JsToken::Semicolon => {
                        break;
                    }
                    JsToken::Comma => {
                        parser_state.next();
                        continue;
                    },
                    JsToken::Newline => {
                        parser_state.next();
                        continue;
                    }
                    _ => {
                        todo!(); //TODO: this should be an error
                    }
                }
            },
            JsToken::Newline => {
                parser_state.next();
                continue;
            },
            _ => {
                if decl_type == DeclType::Const {
                    todo!(); //TODO: its an error to not assign a const a value
                }
                declarations.push(JsAstDeclaration { variable: ident, initial_value: None, decl_type });
                break;
            }
        };
    }

    return ParseResult::Ok(declarations);
}
