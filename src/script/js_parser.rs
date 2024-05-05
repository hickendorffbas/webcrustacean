use std::rc::Rc;

use super::js_lexer::{JsToken, JsTokenWithLocation};


pub struct Script {
    #[allow(dead_code)] statements: Vec<JsAstStatement>, //TODO: use
}

enum JsAstStatement {
    Expression(JsAstExpression),
    Assign(JsAstAssign)
}

struct JsAstBinOp {
    #[allow(dead_code)] op: JsBinOp, //TODO: use
    #[allow(dead_code)] left: Rc<JsAstExpression>, //TODO: use
    #[allow(dead_code)] right: Rc<JsAstExpression>, //TODO: use
}

struct JsAstAssign {
    #[allow(dead_code)] left: JsAstExpression, //TODO: use
    #[allow(dead_code)] right: JsAstExpression, //TODO: use
}

enum JsBinOp {
    Plus,
    Minus,
    Times,
    Divide,
    MemberLookup, //this is the "." in object.member
}

struct JsAstVariable {
    #[allow(dead_code)] name: String, //TODO: use
}

enum JsAstExpression {
    BinOp(JsAstBinOp),
    NumericLiteral(String),
    #[allow(dead_code)] StringLiteral(String), //TODO: use
    FunctionCall(JsAstFunctionCall),
    Variable(JsAstVariable)
}

struct JsAstFunctionCall {
    #[allow(dead_code)] name: String, //TODO: use
    #[allow(dead_code)] arguments: Vec<JsAstExpression>, //TODO: use
}


#[derive(Debug)]
struct JsParserSliceIterator {
    start_idx: usize,
    end_idx: usize,  //start and end are inclusive
    next_idx: usize,
}
impl JsParserSliceIterator {
    fn has_next(&self) -> bool {
        return self.next_idx <= self.end_idx;
    }
    fn next_non_whitespace(&self, tokens: &Vec<JsTokenWithLocation>) -> bool {
        let mut temp_next = self.next_idx;

        loop {
            if temp_next > self.end_idx {
                return false;
            }

            match &tokens[temp_next].token {
                JsToken::Whitespace | JsToken::Newline => { },
                _ => { return true; }
            }
            temp_next += 1;
        }
    }
    fn read_only_identifier(&mut self, tokens: &Vec<JsTokenWithLocation>) -> Option<String> {
        //check if there is only an identifier left, and if so, return it, and consume the iterator

        let mut temp_next = self.next_idx;
        let mut name_to_return = None;
        let mut identifier_seen = false;
        loop {
            if temp_next > self.end_idx {
                return name_to_return;
            }

            match &tokens[temp_next].token {
                JsToken::Whitespace | JsToken::Newline => { },
                JsToken::Identifier(name) => {
                    if identifier_seen {
                        return None;
                    }
                    name_to_return = Some(name.clone());
                    identifier_seen = true;
                    self.next_idx = temp_next + 1;
                }
                _ => { return None }
            }
            temp_next += 1;
        }
    }
    fn read_only_literal_number(&mut self, tokens: &Vec<JsTokenWithLocation>) -> Option<String> {
        //check if there is only an number left, and if so, return it, and consume the iterator

        //TODO: parse fractionals here well, by consuming the dot, and return one string containing the full fractional number

        let mut temp_next = self.next_idx;
        let mut name_to_return = None;
        let mut number_seen = false;
        loop {
            if temp_next > self.end_idx {
                return name_to_return;
            }

            match &tokens[temp_next].token {
                JsToken::Whitespace | JsToken::Newline => { },
                JsToken::Number(number) => {
                    if number_seen {
                        return None;
                    }
                    name_to_return = Some(number.clone());
                    number_seen = true;
                    self.next_idx = temp_next + 1;
                }
                _ => { return None }
            }
            temp_next += 1;
        }
    }
    fn read_possible_function_call(&mut self, tokens: &Vec<JsTokenWithLocation>) -> Option<JsAstFunctionCall> {
        let mut temp_next = self.next_idx;
        let mut function_name = None;
        let mut in_arguments = false;

        loop {
            if temp_next > self.end_idx {
                return None;
            }

            if function_name.is_none() {
                match &tokens[temp_next].token {
                    JsToken::Whitespace | JsToken::Newline => { },
                    JsToken::Identifier(name) => { function_name = Some(name.clone()); }
                    _ => { return None }
                }
            } else if !in_arguments {
                match &tokens[temp_next].token {
                    JsToken::Whitespace | JsToken::Newline => { },
                    JsToken::OpenParenthesis => { in_arguments = true; }
                    _ => { return None }
                }
            } else {
                //TODO: we should handle nested parenthesis

                //at this point we are sure we are parsing a function call, so we start advancing our next pointer
                self.next_idx = temp_next;

                let mut arguments = Vec::new();

                while self.has_next() {
                    let argument_iterator = self.split_and_advance_until_next_token(tokens, JsToken::Comma);

                    if argument_iterator.is_some() {
                        arguments.push(parse_expression(&mut argument_iterator.unwrap(), tokens));

                    } else {
                        let final_argument_iterator = self.split_and_advance_until_next_token(tokens, JsToken::CloseParenthesis);
                        arguments.push(parse_expression(&mut final_argument_iterator.unwrap(), tokens));
                        break;
                    }
                }

                if self.next_non_whitespace(&tokens) {
                    panic!("unexpected tokens after function call");
                }

                return Some(JsAstFunctionCall {
                    name: function_name.unwrap(),
                    arguments: arguments,
                });
            }

            temp_next += 1;
        }

    }
    fn split_and_advance_until_next_token(&mut self, tokens: &Vec<JsTokenWithLocation>, token_to_find: JsToken) -> Option<JsParserSliceIterator> {
        let mut size = 1;
        loop {
            let potential_end_idx = self.next_idx + (size - 1);

            if potential_end_idx > self.end_idx {
                return None;
            }

            let ending_token = &tokens[potential_end_idx];
            if ending_token.token == token_to_find {
                let new_start_idx = self.next_idx;
                self.next_idx += size;

                return Some(JsParserSliceIterator {
                    start_idx: new_start_idx,
                    end_idx: potential_end_idx - 1, //we remove the token_to_find
                    next_idx: new_start_idx,
                });
            }

            size += 1;
        }
    }
    fn check_for_and_split_on(&mut self, tokens: &Vec<JsTokenWithLocation>, token: JsToken) -> Option<(JsParserSliceIterator, JsParserSliceIterator)> {
        let mut split_idx = self.next_idx;
        loop {
            if split_idx > self.end_idx {
                return None;
            }

            //TODO: re-use split_at() here (and possibly in other places)

            if tokens[split_idx].token == token {
                self.next_idx = self.end_idx;  // we consume this iterator fully

                return Some((
                    JsParserSliceIterator { start_idx: self.start_idx, end_idx: split_idx - 1, next_idx: self.start_idx },
                    JsParserSliceIterator { start_idx: split_idx + 1,  end_idx: self.end_idx,  next_idx: split_idx + 1 }
                ));
            }

            split_idx += 1;
        }
    }
    fn find_first_token_idx(&self, tokens: &Vec<JsTokenWithLocation>, token: JsToken) -> Option<usize> {
        let mut possible_idx = self.next_idx;
        loop {
            if possible_idx > self.end_idx {
                return None;
            }

            if tokens[possible_idx].token == token {
                return Some(possible_idx);
            }
            possible_idx += 1;
        }
    }
    fn split_at(&mut self, split_idx: usize) -> Option<(JsParserSliceIterator, JsParserSliceIterator)> {
        if split_idx > self.end_idx {
            return None;
        }

        self.next_idx = self.end_idx;  // we consume this iterator fully
        return Some((
            JsParserSliceIterator { start_idx: self.start_idx, end_idx: split_idx - 1, next_idx: self.start_idx },
            JsParserSliceIterator { start_idx: split_idx + 1,  end_idx: self.end_idx,  next_idx: split_idx + 1 }
        ));
    }
}


pub fn parse_js(tokens: &Vec<JsTokenWithLocation>) -> Script {
    //TODO: we need to do semicolon insertion (see rules on https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Lexical_grammar#automatic_semicolon_insertion)

    let mut token_iterator = JsParserSliceIterator {
        start_idx: 0,
        end_idx: tokens.len() - 1,
        next_idx: 0,
    };

    let mut statements = Vec::new();

    while token_iterator.has_next() {
        let statement_iterator = token_iterator.split_and_advance_until_next_token(tokens, JsToken::Semicolon);
        if statement_iterator.is_some() {
            statements.push(parse_statement(&mut statement_iterator.unwrap(), tokens));
        } else {
            break;
        }
    }

    return Script { statements };
}


fn parse_statement(statement_iterator: &mut JsParserSliceIterator, tokens: &Vec<JsTokenWithLocation>) -> JsAstStatement {

    let optional_equals_split = statement_iterator.check_for_and_split_on(tokens, JsToken::Equals);

    if optional_equals_split.is_some() {
        let (mut left, mut right) = optional_equals_split.unwrap();
        let parsed_left = parse_expression(&mut left, tokens);
        let parsed_right = parse_expression(&mut right, tokens);
        return JsAstStatement::Assign(JsAstAssign { left: parsed_left, right: parsed_right });
    }

    let expression = parse_expression(statement_iterator, tokens);

    return JsAstStatement::Expression(expression);
}


fn parse_expression(iterator: &mut JsParserSliceIterator, tokens: &Vec<JsTokenWithLocation>) -> JsAstExpression {
    //TODO: keep counters for open braces, parentesis and brackets


    /*    + and -    */
    {
        let optional_plus_idx = iterator.find_first_token_idx(tokens, JsToken::Plus);
        let optional_minus_idx = iterator.find_first_token_idx(tokens, JsToken::Minus);

        let (operator, split_idx) = if optional_plus_idx.is_some() && optional_minus_idx.is_some() {
            if optional_plus_idx.unwrap() < optional_minus_idx.unwrap() {
                (Some(JsBinOp::Plus), Some(optional_plus_idx.unwrap()))
            } else {
                (Some(JsBinOp::Minus), Some(optional_minus_idx.unwrap()))
            }
        } else if optional_plus_idx.is_some() {
            (Some(JsBinOp::Plus), Some(optional_plus_idx.unwrap()))
        } else if optional_minus_idx.is_some() {
            (Some(JsBinOp::Minus), Some(optional_minus_idx.unwrap()))
        } else {
            (None, None)
        };

        if operator.is_some() {
            let (mut left_iter, mut right_iter) = iterator.split_at(split_idx.unwrap()).unwrap();

            return JsAstExpression::BinOp(JsAstBinOp {
                op: operator.unwrap(),
                left: Rc::from(parse_expression(&mut left_iter, &tokens)),
                right: Rc::from(parse_expression(&mut right_iter, &tokens)),
            });
        }
    }


    /*    * and /    */
    {
        let optional_times_idx = iterator.find_first_token_idx(tokens, JsToken::Star);
        let optional_divide_idx = iterator.find_first_token_idx(tokens, JsToken::ForwardSlash);

        let (operator, split_idx) = if optional_times_idx.is_some() && optional_divide_idx.is_some() {
            if optional_times_idx.unwrap() < optional_divide_idx.unwrap() {
                (Some(JsBinOp::Times), Some(optional_times_idx.unwrap()))
            } else {
                (Some(JsBinOp::Divide), Some(optional_divide_idx.unwrap()))
            }
        } else if optional_times_idx.is_some() {
            (Some(JsBinOp::Times), Some(optional_times_idx.unwrap()))
        } else if optional_divide_idx.is_some() {
            (Some(JsBinOp::Divide), Some(optional_divide_idx.unwrap()))
        } else {
            (None, None)
        };

        if operator.is_some() {
            let (mut left_iter, mut right_iter) = iterator.split_at(split_idx.unwrap()).unwrap();

            return JsAstExpression::BinOp(JsAstBinOp {
                op: operator.unwrap(),
                left: Rc::from(parse_expression(&mut left_iter, &tokens)),
                right: Rc::from(parse_expression(&mut right_iter, &tokens)),
            });
        }
    }


    /*   the dot operator (member lookup)   */
    {
        let optional_dot_idx = iterator.find_first_token_idx(tokens, JsToken::Dot);
        if optional_dot_idx.is_some() {
            let (mut left_iter, mut right_iter) = iterator.split_at(optional_dot_idx.unwrap()).unwrap();
            return JsAstExpression::BinOp(JsAstBinOp{
                op: JsBinOp::MemberLookup,
                left: Rc::from(parse_expression(&mut left_iter, &tokens)),
                right: Rc::from(parse_expression(&mut right_iter, &tokens)),
            });
        }
    }


    let possible_function_call = iterator.read_possible_function_call(tokens);
    if possible_function_call.is_some() {
        return JsAstExpression::FunctionCall(possible_function_call.unwrap());
    }

    let possible_literal_number = iterator.read_only_literal_number(tokens);
    if possible_literal_number.is_some() {
        return JsAstExpression::NumericLiteral(possible_literal_number.unwrap());
    }

    let possible_ident = iterator.read_only_identifier(tokens);
    if possible_ident.is_some() {
        return JsAstExpression::Variable(JsAstVariable { name: possible_ident.unwrap() });  //TODO: is an identifier always a variable in this case??
    }

    panic!("unparsable token stream found!")
}
