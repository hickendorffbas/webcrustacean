use std::rc::Rc;

use super::js_ast::*;
use super::js_console;
use super::js_lexer::{JsToken, JsTokenWithLocation};


struct ParserState {
    cursor: usize,  //cursor points at the next token to read
    number_of_tokens: usize,
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
    character: u32
}
pub enum ParseErrorType {
    EOF,
    IdentierExpected,
    ExpectedEndOfArgumentList,
}
impl ParseError {
    pub fn error_message(&self) -> String {
        match self.error_type {
            ParseErrorType::EOF => format!("Unexpected end of script at {}:{}", self.line, self.character),
            ParseErrorType::IdentierExpected => format!("Identifier expected at {}:{}", self.line, self.character),
            ParseErrorType::ExpectedEndOfArgumentList => format!("Expected end of argument list at {}:{}", self.line, self.character),
        }
    }
    pub fn error_for_token(error_type: ParseErrorType, token: &JsTokenWithLocation) -> ParseError {
        return ParseError { error_type, line: token.line, character: token.character }
    }
}


pub fn parse_js(tokens: &Vec<JsTokenWithLocation>) -> Script {

    let mut parser_state = ParserState { cursor: 0, number_of_tokens: tokens.len() };
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
            return match pratt_parse_expression(tokens, parser_state, 0) {
                ParseResult::Ok(expr) => Some(ParseResult::Ok(JsAstStatement::Return(expr))),
                ParseResult::ParsingFailed(parse_error) => Some(ParseResult::ParsingFailed(parse_error)),
            }
        },
        JsToken::KeyWordIf => {
            parser_state.next();
            return match parse_conditional(tokens, parser_state) {
                ParseResult::Ok(ast) => Some(ParseResult::Ok(ast)),
                ParseResult::ParsingFailed(error) => Some(ParseResult::ParsingFailed(error)),
            }
        },
        decl_keyword @ (JsToken::KeyWordVar | JsToken::KeyWordLet | JsToken::KeyWordConst) => {
            parser_state.next();

            let decl_type = match decl_keyword {
                JsToken::KeyWordVar => { DeclType::Var },
                JsToken::KeyWordLet => { DeclType::Let },
                JsToken::KeyWordConst => { DeclType::Const },
                _ => { panic!("unreachable"); }
            };

            return match parse_declaration(tokens, parser_state, decl_type) {
                ParseResult::Ok(ast) => Some(ParseResult::Ok(JsAstStatement::Declaration(ast))),
                ParseResult::ParsingFailed(error) => Some(ParseResult::ParsingFailed(error)),
            }
        },
        JsToken::Semicolon => {
            parser_state.next();
            return None;
        }
        _ => {},
    }

    let expression_result = pratt_parse_expression(tokens, parser_state, 0);
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


fn pratt_parse_expression(tokens: &Vec<JsTokenWithLocation>, parser_state: &mut ParserState, min_binding_power: u8) -> ParseResult<JsAstExpression> {

    let mut lhs = match parse_expression_prefix(tokens, parser_state) {
        ParseResult::Ok(result) => result,
        ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
    };

    loop {

        if parser_state.has_ended() { return ParseResult::ParsingFailed(ParseError::error_for_token(ParseErrorType::EOF, &tokens[parser_state.cursor - 1])) };

        match &tokens[parser_state.cursor].token {
            JsToken::Semicolon | JsToken::CloseParenthesis | JsToken::CloseBrace | JsToken::CloseBracket | JsToken::Comma => {
                //we can pop back to the previous level of parsing:
                break;
            },
            JsToken::Newline => {
                //TODO: here something might need to happen wrt to deciding if we should insert a semicolon (stop parsing the statement)
                parser_state.next();
                continue;
            },

            //postfix operator parsing:
            JsToken::OpenBracket => {
                parser_state.next();

                let index_node = match pratt_parse_expression(tokens, parser_state, min_binding_power) {
                    ParseResult::Ok(index_expression) => JsAstExpression::BinOp(JsAstBinOp {
                        op: JsBinOp::PropertyAccess, left: Rc::from(lhs), right: Rc::from(index_expression)
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
                                                                  right: Rc::from(JsAstExpression::Identifier(JsAstIdentifier { name: ident.clone() })) });
                    },
                    _ => {
                        return ParseResult::ParsingFailed(ParseError::error_for_token(ParseErrorType::IdentierExpected, &tokens[parser_state.cursor]));
                    }
                }
            },

            JsToken::Assign => {
                parser_state.next();

                let rhs = match pratt_parse_expression(tokens, parser_state, right_bp) {
                    ParseResult::Ok(rhs) => rhs,
                    ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
                };
                lhs = JsAstExpression::Assignment(JsAstAssign { left: Rc::from(lhs), right: Rc::from(rhs) });
            },

            binop @ (JsToken::Plus | JsToken::Minus | JsToken::Star | JsToken::ForwardSlash | JsToken::EqualsEquals) => {
                parser_state.next();
                let rhs = match pratt_parse_expression(tokens, parser_state, right_bp) {
                    ParseResult::Ok(rhs) => rhs,
                    ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
                };

                match binop {
                    JsToken::Plus => { lhs = JsAstExpression::BinOp(JsAstBinOp { op: JsBinOp::Plus, left: Rc::from(lhs), right: Rc::from(rhs) }); },
                    JsToken::Minus => { lhs = JsAstExpression::BinOp(JsAstBinOp { op: JsBinOp::Minus, left: Rc::from(lhs), right: Rc::from(rhs) }); },
                    JsToken::Star => { lhs = JsAstExpression::BinOp(JsAstBinOp { op: JsBinOp::Times, left: Rc::from(lhs), right: Rc::from(rhs) }); }
                    JsToken::ForwardSlash => { lhs = JsAstExpression::BinOp(JsAstBinOp { op: JsBinOp::Divide, left: Rc::from(lhs), right: Rc::from(rhs) }); }
                    JsToken::EqualsEquals => { lhs = JsAstExpression::BinOp(JsAstBinOp { op: JsBinOp::EqualsEquals, left: Rc::from(lhs), right: Rc::from(rhs) }); }
                    _ => panic!("This should never happen"),
                }
            },

            JsToken::OpenParenthesis => {
                parser_state.next();

                let arguments = match parse_list_of_expressions(tokens, parser_state) {
                    ParseResult::Ok(arguments) => arguments,
                    ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
                };
                lhs = JsAstExpression::FunctionCall(JsAstFunctionCall { function_expression: Rc::from(lhs), arguments });
            },
            _ => todo!(),
        }

    }

    return ParseResult::Ok(lhs);
}


fn parse_expression_prefix(tokens: &Vec<JsTokenWithLocation>, parser_state: &mut ParserState) -> ParseResult<JsAstExpression> {

    eat_newlines(tokens, parser_state); //TODO: not sure if this is always correct, given semicolon insertion
    if parser_state.has_ended() {
        return ParseResult::ParsingFailed(ParseError::error_for_token(ParseErrorType::EOF, &tokens[parser_state.cursor - 1]));
    }

    match &tokens[parser_state.cursor].token {

        operator @ (JsToken::Minus | JsToken::Plus) => {
            //These are the unary operators
            parser_state.next();

            let right_bp = prefix_binding_power(operator);

            match pratt_parse_expression(tokens, parser_state, right_bp) {
                ParseResult::Ok(rhs) => {
                    let un_op = match operator {
                        JsToken::Minus => JsUnOp::Minus,
                        JsToken::Plus => JsUnOp::Plus,
                        _ => panic!("unreachable"),
                    };
                    return ParseResult::Ok(JsAstExpression::UnaryOp(JsAstUnOp { op: un_op, right: Rc::from(rhs) }))
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
                    }
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

                match pratt_parse_expression(tokens, parser_state, 0) {
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

            let mut script = Vec::new();
            loop {
                eat_newlines(tokens, parser_state);

                match &tokens[parser_state.cursor].token {
                    JsToken::CloseBrace => {
                        parser_state.next();
                        break;
                    },
                    _ => {}
                }

                let statement = parse_statement(tokens, parser_state);
                if statement.is_some() {
                    match statement.unwrap() {
                        ParseResult::Ok(stat) => script.push(stat),
                        ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
                    }
                }
            }

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
                    match pratt_parse_expression(tokens, parser_state, 0) {
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
                    match parse_list_of_expressions(tokens, parser_state) {
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

            match pratt_parse_expression(tokens, parser_state, 0) {
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
        _ => todo!(),
    }
}


fn parse_list_of_expressions(tokens: &Vec<JsTokenWithLocation>, parser_state: &mut ParserState) -> ParseResult<Vec<JsAstExpression>> {
    let mut arguments = Vec::new();
    let mut first = true;

    loop {
        if parser_state.has_ended() { return ParseResult::ParsingFailed(ParseError::error_for_token(ParseErrorType::EOF, &tokens[parser_state.cursor - 1])) };

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
                if !first { return ParseResult::ParsingFailed(ParseError::error_for_token(ParseErrorType::ExpectedEndOfArgumentList, &tokens[parser_state.cursor])) }
            },
        }
        match pratt_parse_expression(tokens, parser_state, 0) {
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
        JsToken::Plus => 20,
        JsToken::Minus => 20,
        _ => todo!(),
    }
}


fn infix_binding_power(token: &JsToken) -> (u8, u8) {
    match token {
        JsToken::Assign => (2, 1),
        JsToken::EqualsEquals => (7, 8),
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

    let mut script = Vec::new();
    loop {
        eat_newlines(tokens, parser_state);

        match &tokens[parser_state.cursor].token {
            JsToken::CloseBrace => {
                parser_state.next();
                break;
            },
            _ => {}
        }

        let statement = parse_statement(tokens, parser_state);
        if statement.is_some() {
            match statement.unwrap() {
                ParseResult::Ok(stat) => script.push(stat),
                ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
            }
        }
    }

    return ParseResult::Ok(JsAstFunctionDeclaration { name: function_name.clone(), arguments, script: Rc::from(script) })
}


fn parse_conditional(tokens: &Vec<JsTokenWithLocation>, parser_state: &mut ParserState) -> ParseResult<JsAstStatement> {
    //TODO: javascript supports having a single statement without { } after if, we still need to add that

    match tokens[parser_state.cursor].token {
        JsToken::OpenParenthesis => {
            parser_state.next();
        },
        _ => {
            todo!(); //TODO: this should be an error
        }
    }

    let condition = match pratt_parse_expression(tokens, parser_state, 0) {
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

    let mut script = Vec::new();
    loop {

        let statement = parse_statement(tokens, parser_state);
        if statement.is_some() {
            match statement.unwrap() {
                ParseResult::Ok(statement) => script.push(statement),
                ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
            }
        }

        eat_newlines(tokens, parser_state);

        match tokens[parser_state.cursor].token {
            JsToken::CloseBrace => {
                parser_state.next();
                break;
            },
            _ => {},
        }
    }

    eat_newlines(tokens, parser_state);

    let else_present = match tokens[parser_state.cursor].token {
        JsToken::KeyWordElse => {
            parser_state.next();
            true
        }
        _ => { false }
    };

    let else_script;
    if else_present {
        let mut else_script_buffer = Vec::new();

        match tokens[parser_state.cursor].token {
            JsToken::OpenBrace => {
                parser_state.next();
            },
            _ => {
                todo!(); //TODO: this should be an error (not in all cases, if we have a single statement after the if....)
            }
        }

        loop {
            let statement = parse_statement(tokens, parser_state);
            if statement.is_some() {
                match statement.unwrap() {
                    ParseResult::Ok(statement) => else_script_buffer.push(statement),
                    ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
                }
            }

            eat_newlines(tokens, parser_state);

            match tokens[parser_state.cursor].token {
                JsToken::CloseBrace => {
                    parser_state.next();
                    break;
                },
                _ => {},
            }
        }

        else_script = Some(Rc::from(else_script_buffer));

    } else {
        else_script = None;
    }

    return ParseResult::Ok(JsAstStatement::Conditional(JsAstConditional { condition: Rc::from(condition), script: Rc::from(script), else_script }));
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


fn parse_declaration(tokens: &Vec<JsTokenWithLocation>, parser_state: &mut ParserState, decl_type: DeclType) -> ParseResult<Vec<JsAstDeclaration>> {
    let mut declarations = Vec::new();

    loop {
        if parser_state.has_ended() { return ParseResult::ParsingFailed(ParseError::error_for_token(ParseErrorType::EOF, &tokens[parser_state.cursor - 1])) };

        let ident = match &tokens[parser_state.cursor].token {
            JsToken::Identifier(ident) => {
                parser_state.next();
                JsAstIdentifier { name: ident.clone() }
            },
            _ => todo!(), //TODO: this should be an error
        };

        match tokens[parser_state.cursor].token {
            JsToken::Semicolon => {
                if decl_type == DeclType::Const {
                    todo!(); //TODO: its an error to not assign a const a value
                }
                parser_state.next();
                declarations.push(JsAstDeclaration { variable: ident, initial_value: None, decl_type });
                break;
            }
            JsToken::Comma => {
                if decl_type == DeclType::Const {
                    todo!(); //TODO: its an error to not assign a const a value
                }
                parser_state.next();
                declarations.push(JsAstDeclaration { variable: ident, initial_value: None, decl_type });
                continue;
            },
            JsToken::Assign => {
                parser_state.next();
                match pratt_parse_expression(tokens, parser_state, 0) {
                    ParseResult::Ok(expression) => {
                        declarations.push(JsAstDeclaration { variable: ident, initial_value: Some(expression), decl_type });
                    },
                    ParseResult::ParsingFailed(parse_error) => return ParseResult::ParsingFailed(parse_error),
                };
                match tokens[parser_state.cursor].token {
                    JsToken::Semicolon => {
                        parser_state.next();
                        break;
                    }
                    JsToken::Comma => {
                        parser_state.next();
                        continue;
                    },
                    _ => {
                        todo!(); //TODO: this should be an error
                    }
                }
            },
            _ => {
                todo!(); //TODO: this should be an error
            }
        };
    }

    return ParseResult::Ok(declarations);
}
