use std::collections::HashMap;
use std::rc::Rc;

use super::js_console;
use super::js_execution_context::{
    JsBuiltinFunction,
    JsExecutionContext,
    JsObject,
    JsValue
};
use super::js_lexer::{JsToken, JsTokenWithLocation};


#[derive(Debug)]
pub struct Script {
    statements: Vec<JsAstStatement>,
}
impl Script {
    pub fn execute(&self, js_execution_context: &mut JsExecutionContext) {
        //I might not want these methods and structs in the parser, maybe move them to the general mod.rs file? or better, in a ast.rs
        for statement in &self.statements {
            statement.execute(js_execution_context);
        }

    }
}


#[derive(Debug)]
enum JsAstStatement {
    Expression(JsAstExpression),
    Assign(JsAstAssign),
    Declaration(JsAstDeclaration),
}
impl JsAstStatement {
    fn execute(&self, js_execution_context: &mut JsExecutionContext) {
        match self {
            JsAstStatement::Expression(expression) => {
                let _ = expression.execute(js_execution_context);
            },
            JsAstStatement::Assign(assign) => {
                assign.execute(js_execution_context)
            },
            JsAstStatement::Declaration(declaration) => {
                declaration.execute(js_execution_context)
            },
        }
    }
}


#[derive(Debug)]
struct JsAstBinOp {
    op: JsBinOp,
    left: Rc<JsAstExpression>,
    right: Rc<JsAstExpression>,
}
impl JsAstBinOp {
    fn execute(&self, js_execution_context: &mut JsExecutionContext) -> JsValue {
        let mut left_val = self.left.execute(js_execution_context);

        match self.op {
            JsBinOp::Plus => {
                let mut right_val = self.right.execute(js_execution_context);

                left_val = left_val.deref(&js_execution_context);
                right_val = right_val.deref(&js_execution_context);

                match left_val {
                    JsValue::Number(left_number) => {
                        match right_val {
                            JsValue::Number(right_number) => {
                                return JsValue::Number(left_number + right_number);
                            },
                            _ => { todo!() }
                        }
                    },
                    _ => { todo!() }
                }
            },
            JsBinOp::Minus => {
                let mut right_val = self.right.execute(js_execution_context);

                left_val = left_val.deref(&js_execution_context);
                right_val = right_val.deref(&js_execution_context);

                match left_val {
                    JsValue::Number(left_number) => {
                        match right_val {
                            JsValue::Number(right_number) => {
                                return JsValue::Number(left_number - right_number);
                            },
                            _ => { todo!() }
                        }
                    },
                    _ => { todo!() }
                }
            },
            JsBinOp::Times => {
                let mut right_val = self.right.execute(js_execution_context);

                left_val = left_val.deref(&js_execution_context);
                right_val = right_val.deref(&js_execution_context);

                match left_val {
                    JsValue::Number(left_number) => {
                        match right_val {
                            JsValue::Number(right_number) => {
                                return JsValue::Number(left_number * right_number);
                            },
                            _ => { todo!() }
                        }
                    },
                    _ => { todo!() }
                }
            },
            JsBinOp::Divide => {
                let mut right_val = self.right.execute(js_execution_context);

                left_val = left_val.deref(&js_execution_context);
                right_val = right_val.deref(&js_execution_context);

                match left_val {
                    JsValue::Number(left_number) => {
                        match right_val {
                            JsValue::Number(right_number) => {
                                return JsValue::Number(left_number / right_number);
                            },
                            _ => { todo!() }
                        }
                    },
                    _ => { todo!() }
                }
            },
            JsBinOp::PropertyAccess => {
                let property = match self.right.as_ref() {
                    // when the right hand side of our accessor is an identifier, we don't execute, but just take its name as a string
                    // this is because a.b is equivalent to a["b"]
                    JsAstExpression::Identifier(ident) => { JsValue::String(ident.name.clone()) }
                    _ => { self.right.execute(js_execution_context) }
                };

                let object = JsValue::deref(left_val, js_execution_context);

                match object {
                    JsValue::Object(object) => {
                        match property {
                            JsValue::String(property_value) => {
                                match object.members.get(&property_value) {
                                    Some(address) => { JsValue::Address(*address) },
                                    None => {
                                        //TODO: handle error
                                        todo!()
                                    }
                                }
                            },
                            _ => {
                                //TODO: some of these are invalid, others (like number) are valid (for example for "x[3]")
                                todo!();
                            }
                        }

                    },
                    _ => {
                        todo!();
                    }
                }
            },
        }
    }

    fn build_var_path(&self, path: &mut Vec<String>) {
        match self.op {
            JsBinOp::Plus => todo!(),  //TODO: not sure yet if there is a valid case for this (there might be and we then need to execute())
            JsBinOp::Minus => todo!(),  //TODO: not sure yet if there is a valid case for this (there might be and we then need to execute())
            JsBinOp::Times => todo!(),  //TODO: not sure yet if there is a valid case for this (there might be and we then need to execute())
            JsBinOp::Divide => todo!(),  //TODO: not sure yet if there is a valid case for this (there might be and we then need to execute())
            JsBinOp::PropertyAccess => {
                self.left.build_var_path(path);
                self.right.build_var_path(path);
            },
        }
    }
}


#[derive(Debug)]
struct JsAstAssign {
    left: JsAstExpression,
    right: JsAstExpression,
}
impl JsAstAssign {
    fn execute(&self, js_execution_context: &mut JsExecutionContext) {
        let value = self.right.execute(js_execution_context);
        let target_address = js_execution_context.add_new_value(value);

        let mut variable_path = Vec::new();
        self.left.build_var_path(&mut variable_path);

        let mut first = true;
        let mut current_object_address = None;

        for idx in 0..variable_path.len() {
            let last = idx == variable_path.len() - 1;

            if first {
                if last {
                    js_execution_context.update_variable(variable_path[idx].clone(), target_address);
                } else {
                    match js_execution_context.get_var_address(&variable_path[idx]) {
                        Some(address) => {
                            current_object_address = Some(*address);
                        },
                        None => {
                            todo!();  //TODO: this is an error, var not found
                        }
                    }
                }

                first = false;

            } else {  //not the first element in the path, so we need to keep looking up members in objects

                let object = js_execution_context.get_value(&current_object_address.unwrap());

                if last {
                    match object.unwrap() {
                        JsValue::Object(ref mut obj) => {
                            obj.members.insert(variable_path[idx].clone(), target_address);
                        },
                        _ => {
                            todo!();  //TODO: are there valid cases here? Don't think so....
                        }
                    }
                } else {

                    match object.unwrap() {
                        JsValue::Object(obj) => {
                            let next_address = obj.members.get(&variable_path[idx]);

                            match next_address {
                                Some(address) => {
                                    current_object_address = Some(*address);
                                },
                                None => {
                                    todo!(); //TODO: report error that the member is not found
                                }
                            }

                        },
                        _ => {
                            todo!();  //TODO: are there valid cases here? Don't think so....
                        }
                    }
                }
            }
        }
    }
}


#[derive(Debug)]
struct JsAstDeclaration {
    variable: JsAstIdentifier,
    initial_value: Option<JsAstExpression>,
}
impl JsAstDeclaration {
    fn execute(&self, js_execution_context: &mut JsExecutionContext) {
        let initial_value = if self.initial_value.is_some() {
            self.initial_value.as_ref().unwrap().execute(js_execution_context)
        } else {
            JsValue::Undefined
        };
        let new_address = js_execution_context.add_new_value(initial_value);

        js_execution_context.update_variable(self.variable.name.clone(), new_address);
    }
}


#[derive(Debug)]
enum JsBinOp {
    Plus,
    Minus,
    Times,
    Divide,
    PropertyAccess,
}


#[derive(Debug)]
struct JsAstIdentifier {
    name: String,
}
impl JsAstIdentifier {
    fn execute(&self, js_execution_context: &mut JsExecutionContext) -> JsValue {
        let opt_address = js_execution_context.get_var_address(&self.name);
        if opt_address.is_some() {
            return JsValue::Address(*opt_address.unwrap());
        }
        //TODO: report a proper error (variable not found)
        //      this one might be a bit more complicated if we are assigning the variable for the first time, we might need to pass a parameter
        //      whether or not to create the varibale if it does not exist
        todo!();
    }
}


#[derive(Debug)]
enum JsAstExpression {
    BinOp(JsAstBinOp),
    NumericLiteral(String),
    StringLiteral(String),
    FunctionCall(JsAstFunctionCall),
    Identifier(JsAstIdentifier),
    ObjectLiteral(JsAstObjectLiteral),
}
impl JsAstExpression {
    fn execute(&self, js_execution_context: &mut JsExecutionContext) -> JsValue {
        match self {
            JsAstExpression::BinOp(binop) => { return binop.execute(js_execution_context) },
            JsAstExpression::Identifier(variable) => { return JsValue::deref(variable.execute(js_execution_context), js_execution_context) },
            JsAstExpression::ObjectLiteral(obj) => { return obj.execute(js_execution_context) },

            JsAstExpression::NumericLiteral(numeric_literal) => {
                //TODO: we might want to cache the JsValue somehow, and we need to support more numeric types...

                let parsed_value = numeric_literal.parse();
                match parsed_value {
                    Ok(value) => {
                        return JsValue::Number(value);
                    },
                    Err(_e) => {
                        panic!("could not convert number in string to JsValue::Number");
                    }
                }
            },
            JsAstExpression::StringLiteral(string_literal) => {
                return JsValue::String(string_literal.clone()); //TODO: do we want to make a new string ever time this expression is run?
            },
            JsAstExpression::FunctionCall(function_call) => {
                //TODO: all this code should be moved to the JsAstFunctionCall object

                let mut function = function_call.function_expression.execute(js_execution_context);
                function = function.deref(js_execution_context);

                match function {
                    JsValue::Function(function) => {
                        if function.builtin.is_some() {
                            match function.builtin.as_ref().unwrap() {
                                JsBuiltinFunction::ConsoleLog => {
                                    let to_log = function_call.arguments.get(0); //TODO: handle there being to little or to many arguments

                                    let to_log = to_log.unwrap().execute(js_execution_context);
                                    let to_log = to_log.deref(js_execution_context);

                                    let to_log = match to_log {
                                        JsValue::String(string) =>  { string }
                                        JsValue::Number(number) => { number.to_string() },
                                        JsValue::Boolean(_) => todo!(), //TODO: implement
                                        JsValue::Object(_) => todo!(), //TODO: implement
                                        JsValue::Function(_) => todo!(), //TODO: implement
                                        JsValue::Undefined => { "undefined".to_owned() },
                                        JsValue::Address(_) => todo!(), //TODO: implement
                                    };

                                    js_console::print(to_log.as_str());
                                    return JsValue::Undefined;
                                },
                                #[cfg(test)] JsBuiltinFunction::TesterExport => {
                                    let data_ast = function_call.arguments.get(0);
                                    let data = data_ast.unwrap().execute(js_execution_context); //TODO: even for tests, we probably want to handle the unwrap here
                                    let data = data.deref(js_execution_context);
                                    js_execution_context.export_test_data(data);
                                    return JsValue::Undefined;
                                }
                            }
                        } else {
                            //TODO: implement non-builtin functions
                            todo!();
                        }
                    },
                    _ => {
                        //TODO: report an error (variable is not a function)
                        return JsValue::Undefined;
                    },
                }
            },
        }
    }

    fn build_var_path(&self, path: &mut Vec<String>) {
        match self {
            JsAstExpression::BinOp(binop) => { binop.build_var_path(path) },
            JsAstExpression::Identifier(ident) => { path.push(ident.name.clone()) },
            _ => {
                //TODO: I think this should always be an error
                todo!();
            }
        }
    }
}


#[derive(Debug)]
struct JsAstFunctionCall {
    function_expression: Rc<JsAstExpression>,
    arguments: Vec<JsAstExpression>,
}


#[derive(Debug)]
struct JsAstObjectLiteral {
    //NOTE: for now, we only support strings as member names, but we keep expressions here as key, because eventually we need to support
    //      computed property names (using square brackets)
    members: Vec<(JsAstExpression, JsAstExpression)>,
}
impl JsAstObjectLiteral {
    fn execute(&self, js_execution_context: &mut JsExecutionContext) -> JsValue {
        let mut members = HashMap::new();

        for (key_ast, value_ast) in self.members.iter() {

            match key_ast.execute(js_execution_context) {
                JsValue::String(property_name) => {

                    let value = value_ast.execute(js_execution_context);
                    let address = js_execution_context.add_new_value(value);


                    members.insert(property_name, address);
                },
                _ => {
                    todo!(); //TODO: this should be an error
                }
            }

        }
        return JsValue::Object(JsObject { members });
    }
}


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
    fn is_only_object_literal(&mut self, blocked_tokens: &Vec<JsToken>) -> bool {
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

            match &blocked_tokens[temp_next] {
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
    fn is_only_function_call(&self, blocked_tokens: &Vec<JsToken>) -> bool {
        let mut temp_next = self.next_idx;

        let mut in_function_expression = true;
        let mut in_arguments = false;
        let mut seen_close_parentesis = false;

        loop {
            if temp_next > self.end_idx { return seen_close_parentesis; }

            if in_function_expression {
                match &blocked_tokens[temp_next] {
                    JsToken::OpenParenthesis => {
                        in_arguments = true;
                        in_function_expression = false;
                    },
                    _ => { },
                }
            } else if in_arguments {
                match &blocked_tokens[temp_next] {
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
    fn build_iterator_for_blocked_out_tokens(&self, blocked_tokens: &Vec<JsToken>, open_token: JsToken, close_token: JsToken) -> Option<JsParserSliceIterator> {
        let mut temp_idx = self.next_idx;
        let mut first_idx = 0;
        let mut first_seen = false;

        loop {
            if temp_idx > self.end_idx { return None; }

            if blocked_tokens[temp_idx] == open_token {
                first_idx = temp_idx + 1;
                first_seen = true;
            }
            if blocked_tokens[temp_idx] == close_token {
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

        if split_idx > self.end_idx || split_idx <= self.next_idx { return None; }

        return Some((
            JsParserSliceIterator { end_idx: split_idx - 1, next_idx: self.next_idx },
            JsParserSliceIterator { end_idx: self.end_idx,  next_idx: split_idx + 1 }
        ));
    }
}


pub fn parse_js(tokens: &Vec<JsTokenWithLocation>) -> Script {
    //TODO: we need to do semicolon insertion (see rules on https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Lexical_grammar#automatic_semicolon_insertion)

    if tokens.len() == 0 {
        return Script { statements: Vec::new() };
    }

    let mut token_iterator = JsParserSliceIterator {
        end_idx: tokens.len() - 1,
        next_idx: 0,
    };

    let token_types = tokens.iter().map(|token| token.token.clone()).collect::<Vec<_>>();
    let blocked_out_token_types = block_out_token_types(&mut token_iterator, &token_types);


    let mut statements = Vec::new();

    while token_iterator.has_next() {
        //TODO: if the last statement doesn't end with a semicolon we ignore it, we should fix that via semicolon insertion (also insert one at the end)
        let statement_iterator = token_iterator.split_and_advance_until_next_token(&blocked_out_token_types, JsToken::Semicolon);
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

    return Script { statements };
}


fn parse_function_call(function_iterator: &mut JsParserSliceIterator, tokens: &Vec<JsTokenWithLocation>,
                       blocked_out_token_types: &Vec<JsToken>) -> Option<JsAstFunctionCall> {

    let function_expression_iterator = function_iterator.split_and_advance_until_next_token(&blocked_out_token_types, JsToken::OpenParenthesis);
    let function_expression = parse_expression(&mut function_expression_iterator.unwrap(), tokens);
    if function_expression.is_none() {
        return None;
    }

    let mut arguments = Vec::new();

    while function_iterator.has_next() {
        let argument_iterator = function_iterator.split_and_advance_until_next_token(&blocked_out_token_types, JsToken::Comma);
        if argument_iterator.is_some() {
            let expression = parse_expression(&mut argument_iterator.unwrap(), tokens);
            if expression.is_none() {
                return None;
            }
            arguments.push(expression.unwrap());

        } else {
            let final_argument_iterator = function_iterator.split_and_advance_until_next_token(&blocked_out_token_types, JsToken::CloseParenthesis);
            let expression = parse_expression(&mut final_argument_iterator.unwrap(), tokens);
            if expression.is_none() {
                return None;
            }
            arguments.push(expression.unwrap());
        }
    }

    return Some(JsAstFunctionCall { function_expression: Rc::from(function_expression.unwrap()), arguments });
}


fn parse_declaration(statement_iterator: &mut JsParserSliceIterator, tokens: &Vec<JsTokenWithLocation>) -> Option<JsAstDeclaration> {
    statement_iterator.move_after_next_non_whitespace(tokens); //consume the "var"

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


fn block_out_token_types(iterator: &mut JsParserSliceIterator, token_types: &Vec<JsToken>) -> Vec<JsToken> {
    //block out token types, but only when in scope of the iterator

    let mut blocked_out = Vec::new();

    let mut open_brace = 0;
    let mut open_brack = 0;
    let mut open_paren = 0;

    for (idx, token) in token_types.iter().enumerate() {
        if idx < iterator.next_idx || idx > iterator.end_idx {
            blocked_out.push(token.clone());
            continue;
        }

        match token {
            JsToken::CloseBrace => { open_brace -= 1 },
            JsToken::CloseBracket => { open_brack -= 1 },
            JsToken::CloseParenthesis => { open_paren -= 1 },
            _ => {},
        }

        if open_brace == 0 && open_brack == 0 && open_paren == 0 {
            blocked_out.push(token.clone());
        } else {
            blocked_out.push(JsToken::None);
        }

        match token {
            JsToken::OpenBrace => { open_brace += 1 },
            JsToken::OpenBracket => { open_brack += 1 },
            JsToken::OpenParenthesis => { open_paren += 1 },
            _ => {},
        }

    }

    return blocked_out;
}


fn parse_expression(iterator: &mut JsParserSliceIterator, tokens: &Vec<JsTokenWithLocation>) -> Option<JsAstExpression> {
    let token_types = tokens.iter().map(|token| token.token.clone()).collect::<Vec<_>>();
    let blocked_out_token_types = block_out_token_types(iterator, &token_types);


    /*  (precendece group 11)   + and -    */
    {
        let optional_plus_idx = iterator.find_last_token_idx(&blocked_out_token_types, JsToken::Plus);
        let optional_minus_idx = iterator.find_last_token_idx(&blocked_out_token_types, JsToken::Minus);

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
        let optional_times_idx = iterator.find_last_token_idx(&blocked_out_token_types, JsToken::Star);
        let optional_divide_idx = iterator.find_last_token_idx(&blocked_out_token_types, JsToken::ForwardSlash);

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
        if iterator.is_only_function_call(&blocked_out_token_types) {
            let call = parse_function_call(iterator, tokens, &blocked_out_token_types);
            if call.is_none() {
                return None;
            }
            return Some(JsAstExpression::FunctionCall(call.unwrap()));
        }

        //TODO: implement the [] case

        let optional_dot_idx = iterator.find_last_token_idx(&blocked_out_token_types, JsToken::Dot);
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

    if iterator.is_only_object_literal(&blocked_out_token_types) {
        let parsed_object = parse_object_literal(iterator, tokens, &blocked_out_token_types);
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

    js_console::log_js_error("unparsable token stream found!"); //TODO: add information about line number, char index and maybe part of the text?
    return None;
}


fn parse_object_literal(iterator: &mut JsParserSliceIterator, tokens: &Vec<JsTokenWithLocation>,
                        blocked_out_token_types: &Vec<JsToken>) -> Option<JsAstObjectLiteral> {
    let mut object_properties = Vec::new();

    let mut iterator = iterator.build_iterator_for_blocked_out_tokens(blocked_out_token_types, JsToken::OpenBrace, JsToken::CloseBrace).unwrap();
    let token_types = tokens.iter().map(|token| token.token.clone()).collect::<Vec<_>>();
    let blocked_out_token_types = block_out_token_types(&mut iterator, &token_types);

    let mut last_element_seen = false;
    while !last_element_seen {

        let possible_property_iterator = iterator.split_and_advance_until_next_token(&blocked_out_token_types, JsToken::Comma);

        let mut property_iterator = if possible_property_iterator.is_some() {
            possible_property_iterator.unwrap()
        } else {
            last_element_seen = true;
            JsParserSliceIterator { next_idx: iterator.next_idx, end_idx: iterator.end_idx }
        };

        let mut property_key_iterator = property_iterator.split_and_advance_until_next_token(&blocked_out_token_types, JsToken::Colon).unwrap();

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
