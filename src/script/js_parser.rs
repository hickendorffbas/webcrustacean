use std::rc::Rc;

use super::js_ast::*;
use super::js_console;
use super::js_lexer::{JsToken, JsTokenWithLocation};


#[derive(Debug)]
struct JsParserSliceIterator {
    next_idx: usize,
    end_idx: usize,  //end is inclusive
}
impl JsParserSliceIterator {
    fn has_next(&self) -> bool {
        return self.next_idx <= self.end_idx;
    }
    fn move_after_next_non_whitespace(&mut self, tokens: &Vec<JsTokenWithLocation>) -> bool {
        let mut temp_next = self.next_idx;

        loop {
            if temp_next > self.end_idx { return false; }

            match &tokens[temp_next].token {
                JsToken::Whitespace | JsToken::Newline => { },
                _ => {
                    if temp_next == self.end_idx {
                        self.next_idx = self.end_idx;
                    } else {
                        self.next_idx = temp_next + 1; //we move one after the non-whitespace char (if we can)
                    }
                    return true;
                }
            }
            temp_next += 1;
        }
    }
    fn has_next_non_whitespace(&self, tokens: &Vec<JsTokenWithLocation>) -> bool {
        let mut temp_next = self.next_idx;

        loop {
            if temp_next > self.end_idx { return false; }

            match &tokens[temp_next].token {
                JsToken::Whitespace | JsToken::Newline => { },
                _ => { return true; }
            }
            temp_next += 1;
        }
    }
    fn next_non_whitespace_token_is(&mut self, tokens: &Vec<JsTokenWithLocation>, token: JsToken) -> bool {
        let mut temp_next = self.next_idx;

        loop {
            if temp_next > self.end_idx { return false; }

            match &tokens[temp_next].token {
                JsToken::Whitespace | JsToken::Newline => { },
                matching_token @ _ => {
                    return *matching_token == token;
                }
            }
            temp_next += 1;
        }
    }
    fn read_only_identifier(&mut self, tokens: &Vec<JsTokenWithLocation>) -> Option<String> {
        //check if there is only an identifier left, and if so, return it, and consume the iterator

        let mut temp_next = self.next_idx;
        let mut name_to_return = None;
        loop {
            if temp_next > self.end_idx {
                if name_to_return.is_some() {
                    self.next_idx = self.end_idx;
                }
                return name_to_return;
            }

            match &tokens[temp_next].token {
                JsToken::Whitespace | JsToken::Newline => { },
                JsToken::Identifier(name) => {
                    if name_to_return.is_some() {
                        return None;  //we saw more than 1 identifier
                    }
                    name_to_return = Some(name.clone());
                }
                _ => { return None }
            }
            temp_next += 1;
        }
    }
    fn read_only_literal_regex(&mut self, tokens: &Vec<JsTokenWithLocation>) -> Option<String> {
        //check if there is only a literal regex left, and if so, return it, and consume the iterator

        let mut temp_next = self.next_idx;
        let mut content_to_return = None;
        loop {
            if temp_next > self.end_idx {
                if content_to_return.is_some() {
                    self.next_idx = self.end_idx;
                }
                return content_to_return;
            }

            match &tokens[temp_next].token {
                JsToken::Whitespace | JsToken::Newline => { },
                JsToken::RegexLiteral(content) => {
                    if content_to_return.is_some() {
                        return None;  //we saw more than 1 regex
                    }
                    content_to_return = Some(content.clone());
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
        let mut number_to_return = None;
        loop {
            if temp_next > self.end_idx {
                if number_to_return.is_some() {
                    self.next_idx = self.end_idx;
                }
                return number_to_return;
            }

            match &tokens[temp_next].token {
                JsToken::Whitespace | JsToken::Newline => { },
                JsToken::Number(number) => {
                    if number_to_return.is_some() {
                        return None; // we saw more than 1 number
                    }
                    number_to_return = Some(number.clone());

                }
                _ => { return None }
            }
            temp_next += 1;
        }
    }
    fn read_only_literal_string(&mut self, tokens: &Vec<JsTokenWithLocation>) -> Option<String> {
        //check if there is only an literal string left, and if so, return it, and consume the iterator

        let mut temp_next = self.next_idx;
        let mut string_to_return = None;
        loop {
            if temp_next > self.end_idx {
                if string_to_return.is_some() {
                    self.next_idx = self.end_idx;
                }
                return string_to_return;
            }

            match &tokens[temp_next].token {
                JsToken::Whitespace | JsToken::Newline => { },
                JsToken::LiteralString(number) => {
                    if string_to_return.is_some() {
                        return None; // we saw more than 1 literal string
                    }
                    string_to_return = Some(number.clone());
                }
                _ => { return None }
            }
            temp_next += 1;
        }

    }
    fn is_only_object_literal(&mut self, masked_tokens: &Vec<JsToken>) -> bool {
        let mut temp_next = self.next_idx;
        let mut in_object = false;
        let mut seen_end_of_object = false;

        loop {
            if temp_next > self.end_idx {
                if seen_end_of_object {
                    return true;
                }
                return false;
            }

            match &masked_tokens[temp_next] {
                JsToken::Whitespace | JsToken::Newline => { },
                JsToken::OpenBrace => {
                    in_object = true;
                }
                JsToken::CloseBrace => {
                    in_object = false;
                    seen_end_of_object = true;
                }
                _ => {
                    if !in_object || seen_end_of_object {
                        return false;
                    }
                }
            }
            temp_next += 1;
        }
    }
    fn is_only_function_call(&self, masked_tokens: &Vec<JsToken>) -> bool {
        let mut temp_next = self.next_idx;

        let mut in_function_expression = true;
        let mut in_arguments = false;
        let mut seen_close_parentesis = false;

        loop {
            if temp_next > self.end_idx { return seen_close_parentesis; }

            if in_function_expression {
                match &masked_tokens[temp_next] {
                    JsToken::OpenParenthesis => {
                        in_arguments = true;
                        in_function_expression = false;
                    },
                    _ => { },
                }
            } else if in_arguments {
                match &masked_tokens[temp_next] {
                    JsToken::CloseParenthesis => {
                        seen_close_parentesis = true;
                        in_arguments = false;
                    },
                    _ => { },
                }
            } else if seen_close_parentesis {
                return false;
            }

            temp_next += 1;
        }
    }
    fn split_and_advance_until_next_token(&mut self, tokens: &Vec<JsToken>, token_to_find: JsToken) -> Option<JsParserSliceIterator> {
        let mut size = 1;
        loop {
            let potential_end_idx = self.next_idx + (size - 1);
            if potential_end_idx > self.end_idx { return None; }

            let ending_token = &tokens[potential_end_idx];
            if *ending_token == token_to_find {
                let new_start_idx = self.next_idx;
                self.next_idx += size;

                if potential_end_idx == 0 {
                    // if the ending token is at the first position, we need to have an iterator up to that first position. We can't do that since
                    // the end index is inclusive, and unsigned (it would need to be -1 to represent not including the token at position 0)
                    // so instead we make the iterator empty by returning a bigger next_idx.
                    return Some(JsParserSliceIterator { end_idx: 0, next_idx: 1 });
                }

                return Some(JsParserSliceIterator {
                    end_idx: potential_end_idx - 1, //we remove the token_to_find
                    next_idx: new_start_idx,
                });
            }

            size += 1;
        }
    }
    fn check_for_and_split_on(&mut self, tokens: &Vec<JsTokenWithLocation>, token: JsToken) -> Option<(JsParserSliceIterator, JsParserSliceIterator)> {
        // split this iterator in 2 new ones, starting from the current position of this iterator

        let mut split_idx = self.next_idx;
        loop {
            if split_idx > self.end_idx { return None; }

            if tokens[split_idx].token == token {
                return self.split_at(split_idx);
            }

            split_idx += 1;
        }
    }
    fn build_iterator_between_tokens(&self, token_types: &Vec<JsToken>, open_token: JsToken, close_token: JsToken) -> Option<JsParserSliceIterator> {
        let mut temp_idx = self.next_idx;
        let mut first_idx = 0;
        let mut first_seen = false;

        loop {
            if temp_idx > self.end_idx { return None; }

            if token_types[temp_idx] == open_token {
                first_idx = temp_idx + 1;
                first_seen = true;
            }
            if token_types[temp_idx] == close_token {
                if !first_seen { return None }
                return Some(JsParserSliceIterator { next_idx: first_idx, end_idx: temp_idx - 1} );
            }

            temp_idx += 1;
        }
    }
    fn find_last_token_idx(&self, tokens: &Vec<JsToken>, token_to_find: JsToken) -> Option<usize> {
        for idx in (self.next_idx..(self.end_idx+1)).rev() {
            if tokens[idx] == token_to_find {
                return Some(idx);
            }
        }
        return None;
    }
    fn split_at(&mut self, split_idx: usize) -> Option<(JsParserSliceIterator, JsParserSliceIterator)> {
        //make 2 iterators from this iterator, starting from the current position of this iterator

        if split_idx > self.end_idx || split_idx < self.next_idx { return None; }

        return Some((
            JsParserSliceIterator { end_idx: split_idx - 1, next_idx: self.next_idx },
            JsParserSliceIterator { end_idx: self.end_idx,  next_idx: split_idx + 1 }
        ));
    }
}


pub fn parse_js(tokens: &Vec<JsTokenWithLocation>) -> Script {
    //TODO: we need to do semicolon insertion (see rules on https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Lexical_grammar#automatic_semicolon_insertion)

    if tokens.len() == 0 {
        return Vec::new();
    }

    let mut token_iterator = JsParserSliceIterator {
        end_idx: tokens.len() - 1,
        next_idx: 0,
    };

    let token_types = tokens.iter().map(|token| token.token.clone()).collect::<Vec<_>>();
    let masked_token_types = mask_token_types(&mut token_iterator, &token_types);


    let mut statements = Vec::new();

    while token_iterator.has_next() {
        //TODO: if the last statement doesn't end with a semicolon we ignore it, we should fix that via semicolon insertion (also insert one at the end)
        let statement_iterator = token_iterator.split_and_advance_until_next_token(&masked_token_types, JsToken::Semicolon);
        if statement_iterator.is_some() {
            if statement_iterator.as_ref().unwrap().has_next_non_whitespace(&tokens) {
                let stat = parse_statement(&mut statement_iterator.unwrap(), tokens);
                if stat.is_some() {
                    statements.push(stat.unwrap());
                }
            }
        } else {
            break;
        }
    }

    return statements;
}


fn parse_function_call(function_iterator: &mut JsParserSliceIterator, tokens: &Vec<JsTokenWithLocation>,
                       masked_token_types: &Vec<JsToken>) -> Option<JsAstFunctionCall> {
    let token_types = tokens.iter().map(|token| token.token.clone()).collect::<Vec<_>>();

    let function_expression_iterator = function_iterator.split_and_advance_until_next_token(&masked_token_types, JsToken::OpenParenthesis);
    let function_expression = parse_expression(&mut function_expression_iterator.unwrap(), tokens);
    if function_expression.is_none() {
        return None;
    }

    let mut arguments = Vec::new();

    //The below basically just removes the close CloseParenthesis
    let function_iterator = function_iterator.check_for_and_split_on(tokens, JsToken::CloseParenthesis);

    if function_iterator.is_some() {
        let (mut function_iterator, _) = function_iterator.unwrap();

        let masked_token_types_for_args = mask_token_types(&mut function_iterator, &token_types);

        loop {
            let argument_iterator = function_iterator.split_and_advance_until_next_token(&masked_token_types_for_args, JsToken::Comma);
            if argument_iterator.is_some() {
                let expression = parse_expression(&mut argument_iterator.unwrap(), tokens);
                if expression.is_none() {
                    return None;
                }
                arguments.push(expression.unwrap());

            } else {
                if !function_iterator.has_next_non_whitespace(&tokens) {
                    //This function does not have arguments
                    //TODO: this is only a valid case in the first iteration of the loop, fix that (do this check somewhere else maybe?)
                    break;
                }

                let expression = parse_expression(&mut function_iterator, tokens);
                if expression.is_none() {
                    return None;
                }
                arguments.push(expression.unwrap());
                break;
            }
        }
    }

    return Some(JsAstFunctionCall { function_expression: Rc::from(function_expression.unwrap()), arguments });
}


fn parse_function_declaration(iterator: &mut JsParserSliceIterator, tokens: &Vec<JsTokenWithLocation>) -> Option<JsAstFunctionDeclaration> {
    let token_types = tokens.iter().map(|token| token.token.clone()).collect::<Vec<_>>();

    iterator.move_after_next_non_whitespace(tokens); //consume the "function" keyword

    let function_name_split = iterator.check_for_and_split_on(tokens, JsToken::OpenParenthesis);

    if function_name_split.is_some() {
        let (mut function_name_iterator, mut other_iterator) = function_name_split.unwrap();

        let function_name = function_name_iterator.read_only_identifier(tokens).unwrap();

        let function_body_split = other_iterator.check_for_and_split_on(tokens, JsToken::CloseParenthesis);
        if function_body_split.is_some() {
            let (mut argument_iterator, mut function_body_iterator) = function_body_split.unwrap();

            let masked_token_types_for_args = mask_token_types(&mut argument_iterator, &token_types);

            let mut arguments = Vec::new();

            while argument_iterator.has_next() {
                let possible_argument_iterator = argument_iterator.split_and_advance_until_next_token(&masked_token_types_for_args, JsToken::Comma);

                if possible_argument_iterator.is_none() {
                    let arg_name = argument_iterator.read_only_identifier(tokens).unwrap();
                    arguments.push(JsAstIdentifier { name: arg_name });
                    break;
                } else {
                    let arg_name = possible_argument_iterator.unwrap().read_only_identifier(tokens).unwrap();
                    arguments.push(JsAstIdentifier { name: arg_name });
                }
            }

            function_body_iterator.move_after_next_non_whitespace(tokens); //consume the opening brace

            let mut statements = Vec::new();

            while function_body_iterator.has_next() {

                //TODO: if the last statement doesn't end with a semicolon we ignore it, we should fix that via semicolon insertion (also insert one at the end)
                let statement_iterator = function_body_iterator.split_and_advance_until_next_token(&token_types, JsToken::Semicolon);
                if statement_iterator.is_some() {
                    if statement_iterator.as_ref().unwrap().has_next_non_whitespace(&tokens) {
                        let stat = parse_statement(&mut statement_iterator.unwrap(), tokens);
                        if stat.is_some() {
                            statements.push(stat.unwrap());
                        }
                    }
                } else {
                    break;
                }
            }

            return Some(JsAstFunctionDeclaration { name: function_name, arguments: arguments, script: Rc::from(statements) });
        }

    }

    return None;
}


fn parse_declaration(statement_iterator: &mut JsParserSliceIterator, tokens: &Vec<JsTokenWithLocation>) -> Option<JsAstDeclaration> {
    statement_iterator.move_after_next_non_whitespace(tokens); //consume the "var" keyword

    let optional_equals_split = statement_iterator.check_for_and_split_on(tokens, JsToken::Equals);

    if optional_equals_split.is_some() {
        let (mut left, mut right) = optional_equals_split.unwrap();

        let possible_ident = left.read_only_identifier(tokens);
        let variable = if possible_ident.is_some() {
            JsAstIdentifier { name: possible_ident.unwrap() }
        } else {
            panic!("Expected only an identifier after var decl");
        };

        let expression = parse_expression(&mut right, tokens);
        if expression.is_none() {
            return None;
        }

        return Some(JsAstDeclaration {
            variable,
            initial_value: expression,
        });
    }

    let possible_ident = statement_iterator.read_only_identifier(tokens);
    let variable = if possible_ident.is_some() {
        JsAstIdentifier { name: possible_ident.unwrap() }
    } else {
        panic!("Expected only an identifier after var decl");
    };

    return Some(JsAstDeclaration {
        variable,
        initial_value: None,
    });
}


fn parse_statement(statement_iterator: &mut JsParserSliceIterator, tokens: &Vec<JsTokenWithLocation>) -> Option<JsAstStatement> {

    if statement_iterator.next_non_whitespace_token_is(&tokens, JsToken::KeyWordVar) {
        let decl = parse_declaration(statement_iterator, tokens);
        if decl.is_none() {
            return None;
        }
        return Some(JsAstStatement::Declaration(decl.unwrap()));
    }

    if statement_iterator.next_non_whitespace_token_is(&tokens, JsToken::KeyWordFunction) {
        let function_declaration = parse_function_declaration(statement_iterator, tokens);
        if function_declaration.is_none() {
            return None;
        }
        return Some(JsAstStatement::FunctionDeclaration(function_declaration.unwrap()));
    }

    if statement_iterator.next_non_whitespace_token_is(&tokens, JsToken::KeyWordReturn) {
        statement_iterator.move_after_next_non_whitespace(tokens); //consume the "return" keyword

        let expression = parse_expression(statement_iterator, tokens);
        if expression.is_none() {
            return None;
        }
        return Some(JsAstStatement::Return(expression.unwrap()));

    }

    let optional_equals_split = statement_iterator.check_for_and_split_on(tokens, JsToken::Equals);

    if optional_equals_split.is_some() {
        let (mut left, mut right) = optional_equals_split.unwrap();
        let parsed_left = parse_expression(&mut left, tokens);
        let parsed_right = parse_expression(&mut right, tokens);
        if parsed_left.is_none() || parsed_right.is_none() {
            return None;
        }
        return Some(JsAstStatement::Assign(JsAstAssign { left: parsed_left.unwrap(), right: parsed_right.unwrap() }));
    }

    let expression = parse_expression(statement_iterator, tokens);
    if expression.is_none() {
        return None;
    }

    return Some(JsAstStatement::Expression(expression.unwrap()));
}


fn mask_token_types(iterator: &mut JsParserSliceIterator, token_types: &Vec<JsToken>) -> Vec<JsToken> {
    //mask token types (set those in braces/brackets/parenthesis to None), but only when in scope of the iterator

    let mut masked = Vec::new();

    let mut open_brace = 0;
    let mut open_brack = 0;
    let mut open_paren = 0;

    for (idx, token) in token_types.iter().enumerate() {
        if idx < iterator.next_idx || idx > iterator.end_idx {
            masked.push(token.clone());
            continue;
        }

        match token {
            JsToken::CloseBrace => { open_brace -= 1 },
            JsToken::CloseBracket => { open_brack -= 1 },
            JsToken::CloseParenthesis => { open_paren -= 1 },
            _ => {},
        }

        if open_brace == 0 && open_brack == 0 && open_paren == 0 {
            masked.push(token.clone());
        } else {
            masked.push(JsToken::None);
        }

        match token {
            JsToken::OpenBrace => { open_brace += 1 },
            JsToken::OpenBracket => { open_brack += 1 },
            JsToken::OpenParenthesis => { open_paren += 1 },
            _ => {},
        }

    }

    return masked;
}


fn parse_expression(iterator: &mut JsParserSliceIterator, tokens: &Vec<JsTokenWithLocation>) -> Option<JsAstExpression> {
    let token_types = tokens.iter().map(|token| token.token.clone()).collect::<Vec<_>>();
    let masked_token_types = mask_token_types(iterator, &token_types);


    /*  (precendece group 11)   + and -    */
    {
        let optional_plus_idx = iterator.find_last_token_idx(&masked_token_types, JsToken::Plus);
        let optional_minus_idx = iterator.find_last_token_idx(&masked_token_types, JsToken::Minus);

        let (operator, split_idx) = if optional_plus_idx.is_some() && optional_minus_idx.is_some() {
            if optional_plus_idx.unwrap() > optional_minus_idx.unwrap() {
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

            let left_ast = parse_expression(&mut left_iter, &tokens);
            let right_ast = parse_expression(&mut right_iter, &tokens);
            if left_ast.is_none() || right_ast.is_none() {
                return None;
            }

            return Some(JsAstExpression::BinOp(JsAstBinOp {
                op: operator.unwrap(),
                left: Rc::from(left_ast.unwrap()),
                right: Rc::from(right_ast.unwrap()),
            }));
        }
    }


    /*  (precendece group 12)    * and /    */
    {
        let optional_times_idx = iterator.find_last_token_idx(&masked_token_types, JsToken::Star);
        let optional_divide_idx = iterator.find_last_token_idx(&masked_token_types, JsToken::ForwardSlash);

        let (operator, split_idx) = if optional_times_idx.is_some() && optional_divide_idx.is_some() {
            if optional_times_idx.unwrap() > optional_divide_idx.unwrap() {
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

            let left_ast = parse_expression(&mut left_iter, &tokens);
            let right_ast = parse_expression(&mut right_iter, &tokens);
            if left_ast.is_none() || right_ast.is_none() {
                return None;
            }

            return Some(JsAstExpression::BinOp(JsAstBinOp {
                op: operator.unwrap(),
                left: Rc::from(left_ast.unwrap()),
                right: Rc::from(right_ast.unwrap()),
            }));
        }
    }


    /* (precendece group 17): function call and PropertyAccess (dot operator and [])  */
    {
        if iterator.is_only_function_call(&masked_token_types) {
            let call = parse_function_call(iterator, tokens, &masked_token_types);
            if call.is_none() {
                return None;
            }
            return Some(JsAstExpression::FunctionCall(call.unwrap()));
        }

        //TODO: implement the [] case

        let optional_dot_idx = iterator.find_last_token_idx(&masked_token_types, JsToken::Dot);
        if optional_dot_idx.is_some() {
            let (mut left_iter, mut right_iter) = iterator.split_at(optional_dot_idx.unwrap()).unwrap();

            let left_ast = parse_expression(&mut left_iter, &tokens);
            let right_ast = parse_expression(&mut right_iter, &tokens);
            if left_ast.is_none() || right_ast.is_none() {
                return None;
            }

            return Some(JsAstExpression::BinOp(JsAstBinOp{
                op: JsBinOp::PropertyAccess,
                left: Rc::from(left_ast.unwrap()),
                right: Rc::from(right_ast.unwrap()),
            }));
        }
    }

    let possible_literal_number = iterator.read_only_literal_number(tokens);
    if possible_literal_number.is_some() {
        return Some(JsAstExpression::NumericLiteral(possible_literal_number.unwrap()));
    }

    let possible_literal_string = iterator.read_only_literal_string(tokens);
    if possible_literal_string.is_some() {
        return Some(JsAstExpression::StringLiteral(possible_literal_string.unwrap()));
    }

    if iterator.is_only_object_literal(&masked_token_types) {
        let parsed_object = parse_object_literal(iterator, tokens, &masked_token_types);
        if parsed_object.is_none() {
            return None;
        }
        return Some(JsAstExpression::ObjectLiteral(parsed_object.unwrap()));
    }

    let possible_ident = iterator.read_only_identifier(tokens);
    if possible_ident.is_some() {
        return Some(JsAstExpression::Identifier(JsAstIdentifier{ name: possible_ident.unwrap() }));
    }

    let possible_literal_regex = iterator.read_only_literal_regex(tokens);
    if possible_literal_regex.is_some() {
        //TODO: regexes are not implemented yet, so for now we just return the regex itself as an empty string
        return Some(JsAstExpression::StringLiteral(String::new()));
    }

    let line = tokens[iterator.next_idx].line;
    let char = tokens[iterator.next_idx].character;
    js_console::log_js_error(format!("unparsable token stream found starting at {line}::{char}").as_str());

    return None;
}


fn parse_object_literal(iterator: &mut JsParserSliceIterator, tokens: &Vec<JsTokenWithLocation>,
    masked_token_types: &Vec<JsToken>) -> Option<JsAstObjectLiteral> {
    let mut object_properties = Vec::new();

    let mut iterator = iterator.build_iterator_between_tokens(masked_token_types, JsToken::OpenBrace, JsToken::CloseBrace).unwrap();
    let token_types = tokens.iter().map(|token| token.token.clone()).collect::<Vec<_>>();
    let masked_token_types = mask_token_types(&mut iterator, &token_types);

    let mut last_element_seen = false;
    while !last_element_seen {

        let possible_property_iterator = iterator.split_and_advance_until_next_token(&masked_token_types, JsToken::Comma);

        let mut property_iterator = if possible_property_iterator.is_some() {
            possible_property_iterator.unwrap()
        } else {
            last_element_seen = true;
            JsParserSliceIterator { next_idx: iterator.next_idx, end_idx: iterator.end_idx }
        };

        let possible_property_key_iterator = property_iterator.split_and_advance_until_next_token(&masked_token_types, JsToken::Colon);
        if last_element_seen && possible_property_key_iterator.is_none() {
            //This happens with the empty object, we are with the "last" element because no comma is present, but also no colon is present
            break;
        }
        let mut property_key_iterator = possible_property_key_iterator.unwrap();

        let key_expression = {
            let possible_literal_key = property_key_iterator.read_only_literal_string(tokens);
            if possible_literal_key.is_some() {
                JsAstExpression::StringLiteral(possible_literal_key.unwrap())
            } else {
                // An identifier seen in an object literal is not an identifier, but a literal string without quotes
                let possible_ident = property_key_iterator.read_only_identifier(tokens);
                if possible_ident.is_some() {
                    JsAstExpression::StringLiteral(possible_ident.unwrap())
                } else {
                    todo!();  //TODO: give an error
                }
            }
        };

        let value_expression = parse_expression(&mut property_iterator, tokens);

        let value_expression = match value_expression {
            Some(ast) => {
                ast
            },
            _ => { return None },
        };

        object_properties.push( (key_expression, value_expression) );
    }

    return Some(JsAstObjectLiteral { members: object_properties });
}
