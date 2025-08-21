use std::rc::Rc;

use crate::script::js_ast::{
    JsAstAssign,
    JsAstBinOp,
    JsAstExpression,
    JsAstFunctionCall,
    JsAstFunctionDeclaration,
    JsAstIdentifier,
    JsAstObjectLiteral,
    JsAstStatement,
    JsBinOp,
    Script,
};
use crate::script::js_lexer::{JsToken, JsTokenWithLocation};


struct ParserState {
    cursor: usize,  //cursor points at the next token to read
    number_of_tokens: usize,
    previous_positions: Vec<usize>,
}
impl ParserState {
    fn has_next(&self) -> bool {
        return self.cursor < self.number_of_tokens;
    }
    fn next(&mut self) {
        self.cursor += 1;
    }
    fn push_state(&mut self) {
        self.previous_positions.push(self.cursor);
    }
    fn pop_state(&mut self) {
        self.cursor = self.previous_positions.pop().unwrap();
    }
}


pub enum ParseResult<T> {
    Ok(T),
    NoMatch,
    ParsingFailed(ParseError),
}
pub struct ParseError {
    error_type: ParseErrorType,
    line: u32,
    character: u32
}
pub enum ParseErrorType {
    IdentierExpected,
    ExpectedEndOfArgumentList,
    Unknown, //ideally we replace every instance of this one with a specific error
}
impl ParseError {
    pub fn error_message(&self) -> String {
        match self.error_type {
            ParseErrorType::Unknown => format!("Unknown error while parsing script at {}:{}", self.line, self.character),
            ParseErrorType::IdentierExpected => format!("Identifier expected at {}:{}", self.line, self.character),
            ParseErrorType::ExpectedEndOfArgumentList => format!("Expected end of argument list at {}:{}", self.line, self.character),
        }
    }
    pub fn error_for_token(error_type: ParseErrorType, token: &JsTokenWithLocation) -> ParseError {
        return ParseError { error_type, line: token.line, character: token.character }
    }
}


pub fn parse_js(tokens: &Vec<JsTokenWithLocation>) -> ParseResult<Script> {
    let mut parser_state = ParserState { cursor: 0, number_of_tokens: tokens.len(), previous_positions: Vec::new() };
    let mut statements = Vec::new();

    while parser_state.has_next() {
        let possible_statement = parse_statement(tokens, &mut parser_state);
        match possible_statement {
            ParseResult::Ok(node) => statements.push(node),
            ParseResult::NoMatch => {
                //this happens for the empty statement, so we just continue
            },
            ParseResult::ParsingFailed(error) => return ParseResult::ParsingFailed(error),
        }
    }

    return ParseResult::Ok(statements);
}


fn parse_statement(tokens: &Vec<JsTokenWithLocation>, parser_state: &mut ParserState) -> ParseResult<JsAstStatement> {
    match tokens[parser_state.cursor].token {
        JsToken::KeyWordFunction => {
            let possible_function = parse_function_declaration(tokens, parser_state);
            return match possible_function {
                ParseResult::Ok(ast) => ParseResult::Ok(JsAstStatement::FunctionDeclaration(ast)),
                ParseResult::NoMatch => ParseResult::ParsingFailed(ParseError::error_for_token(ParseErrorType::Unknown, &tokens[0])),
                ParseResult::ParsingFailed(error) => ParseResult::ParsingFailed(error),
            }
        },
        _ => {},
    }

    parser_state.push_state();
    let expression_result = pratt_parse_expression(tokens, parser_state, 0);
    match expression_result {
        ParseResult::Ok(result) => {
            match &tokens[parser_state.cursor].token {
                JsToken::Semicolon => {
                    parser_state.next();
                },
                _ => {},
            }
            return ParseResult::Ok(JsAstStatement::Expression(result));
        }
        ParseResult::NoMatch => return ParseResult::NoMatch, //This happens for the empty statement
        ParseResult::ParsingFailed(error) => return ParseResult::ParsingFailed(error),
    }
}


fn pratt_parse_expression(tokens: &Vec<JsTokenWithLocation>, parser_state: &mut ParserState, min_binding_power: u8) -> ParseResult<JsAstExpression> {

    let mut lhs = match parse_expression_prefix(tokens, parser_state) {
        ParseResult::Ok(result) => result,
        ParseResult::NoMatch => return ParseResult::NoMatch,
        ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
    };

    loop {

        match &tokens[parser_state.cursor].token {
            JsToken::Semicolon | JsToken::CloseParenthesis | JsToken::CloseBrace | JsToken::CloseBracket | JsToken::Comma => {
                //we can pop back to the previous level of parsing:
                break;
            },
            JsToken::Newline => todo!(), //TODO: here something might need to happen wrt to deciding if we should insert a semicolon (stop parsing the statement)
            _ => {}
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
                                                                  right: Rc::from(JsAstExpression::Identifier(JsAstIdentifier { name: ident.clone() })) });
                    },
                    _ => {
                        return ParseResult::ParsingFailed(ParseError::error_for_token(ParseErrorType::IdentierExpected, &tokens[parser_state.cursor]));
                    }
                }
            },

            JsToken::Equals => {
                parser_state.next();

                let rhs = match pratt_parse_expression(tokens, parser_state, right_bp) {
                    ParseResult::Ok(rhs) => rhs,
                    ParseResult::NoMatch => todo!(), //TODO: implement
                    ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
                };
                lhs = JsAstExpression::Assignment(JsAstAssign { left: Rc::from(lhs), right: Rc::from(rhs) });
            },

            binop @ (JsToken::Plus | JsToken::Minus | JsToken::Star | JsToken::ForwardSlash) => {
                parser_state.next();
                let rhs = match pratt_parse_expression(tokens, parser_state, right_bp) {
                    ParseResult::Ok(rhs) => rhs,
                    ParseResult::NoMatch => todo!(), //TODO: implement
                    ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
                };

                match binop {
                    JsToken::Plus => { lhs = JsAstExpression::BinOp(JsAstBinOp { op: JsBinOp::Plus, left: Rc::from(lhs), right: Rc::from(rhs) }); },
                    JsToken::Minus => { lhs = JsAstExpression::BinOp(JsAstBinOp { op: JsBinOp::Minus, left: Rc::from(lhs), right: Rc::from(rhs) }); },
                    JsToken::Star => { lhs = JsAstExpression::BinOp(JsAstBinOp { op: JsBinOp::Times, left: Rc::from(lhs), right: Rc::from(rhs) }); }
                    JsToken::ForwardSlash => { lhs = JsAstExpression::BinOp(JsAstBinOp { op: JsBinOp::Divide, left: Rc::from(lhs), right: Rc::from(rhs) }); }
                    _ => panic!("This should never happen"),
                }
            },

            JsToken::OpenParenthesis => {
                //TODO: it is not clear to me how this is distinguished from parenthesis that are used for grouping

                parser_state.next();

                let mut arguments = Vec::new();
                let mut first = true;
                loop {
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
                                return ParseResult::ParsingFailed(ParseError::error_for_token(ParseErrorType::ExpectedEndOfArgumentList, &tokens[parser_state.cursor]))
                            }
                        },
                    }
                    match pratt_parse_expression(tokens, parser_state, 0) {
                        ParseResult::Ok(expression) => {
                            arguments.push(expression);
                        },
                        ParseResult::NoMatch => todo!(), //TODO: implement
                        ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
                    }
                    first = false;
                }

                lhs = JsAstExpression::FunctionCall(JsAstFunctionCall { function_expression: Rc::from(lhs), arguments });
            }

            _ => todo!(),
        }

    }

    return ParseResult::Ok(lhs);
}


fn parse_expression_prefix(tokens: &Vec<JsTokenWithLocation>, parser_state: &mut ParserState) -> ParseResult<JsAstExpression> {

    loop {

        if !parser_state.has_next() {
            return ParseResult::NoMatch;
        }

        match &tokens[parser_state.cursor].token {
            JsToken::Newline => {
                //TODO: not sure if this is always correct, given semicolon insertion
                parser_state.next();
            },
            _ => {
                break;
            }
        }
    }

    match &tokens[parser_state.cursor].token {
        JsToken::Number(literal_number) => {
            parser_state.next();
            return ParseResult::Ok(JsAstExpression::NumericLiteral(literal_number.clone()));
        },
        JsToken::LiteralString(literal_string) => {
            parser_state.next();
            return ParseResult::Ok(JsAstExpression::StringLiteral(literal_string.clone()));
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
                            todo!(); //TODO: this should be an error
                        }
                        //the first time we don't expect a comma, so we just don't do anything here
                    }
                }

                match &tokens[parser_state.cursor].token {
                    JsToken::Identifier(property_name) => {
                        parser_state.next();
                        current_property_name = property_name;
                    },
                    JsToken::LiteralString(property_name) => {
                        parser_state.next();
                        current_property_name = property_name;
                    }
                    _ => {
                        todo!(); //TODO: are there any valid cases for this?
                    }
                }

                match &tokens[parser_state.cursor].token {
                    JsToken::Colon => {
                        parser_state.next();
                    },
                    _ => {
                        todo!(); //TODO: handle the case where a shorthand is used (i.e. {a} to mean { a : a })
                    }
                }

                match pratt_parse_expression(tokens, parser_state, 0) {
                    ParseResult::Ok(expression) => members.push((JsAstExpression::StringLiteral(current_property_name.clone()), expression)),
                    ParseResult::NoMatch => todo!(), //TODO: implement
                    ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
                }

                first = false;
            }
            return ParseResult::Ok(JsAstExpression::ObjectLiteral(JsAstObjectLiteral { members: members }));
        },
        _ => todo!(),
    }
}


//TODO: this is unused because, although we parse expression prefixes, we don't process prefix operators yet. we should (like the "-" for negative numbers)
fn prefix_binding_power(token: &JsToken) -> u8 {
    match token {
        JsToken::Identifier(_) => 10,
        _ => todo!(),
    }
}


fn infix_binding_power(token: &JsToken) -> (u8, u8) {
    match token {
        JsToken::Equals => (2, 1),
        JsToken::Plus => (10, 11),
        JsToken::Minus => (10, 11),
        JsToken::Star => (14, 15),
        JsToken::ForwardSlash => (14, 15),
        JsToken::Dot => (100, 101),
        JsToken::OpenParenthesis => (110, 111),
        _ => todo!(),
    }
}


fn parse_function_declaration(tokens: &Vec<JsTokenWithLocation>, parser_state: &mut ParserState) -> ParseResult<JsAstFunctionDeclaration> {
    todo!(); //TODO: implement
}
