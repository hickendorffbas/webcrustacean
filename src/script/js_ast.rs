use std::collections::HashMap;
use std::rc::Rc;

use super::js_console;
use super::js_execution_context::{
    JsBuiltinFunction,
    JsError,
    JsExecutionContext,
    JsFunction,
    JsObject,
    JsValue,
};
use super::js_interpreter::JsInterpreter;


pub type Script = Vec<JsAstStatement>;


#[derive(Debug)]
pub enum JsAstStatement {
    Expression(JsAstExpression),
    Assign(JsAstAssign),
    Declaration(JsAstDeclaration),
    FunctionDeclaration(JsAstFunctionDeclaration),  //TODO: a function declaration is not a statement, technically, but we pretend it is for now
                                                    //      (it actually is a "source element", a statement is also a source element)
    Return(JsAstExpression),
}
impl JsAstStatement {

    pub fn execute(&self, js_interpreter: &mut JsInterpreter) -> bool {
        //returns a boolean saying whether to run the next statement

        match self {
            JsAstStatement::Expression(expression) => {
                let _ = expression.execute(js_interpreter);
            },
            JsAstStatement::Assign(assign) => {
                assign.execute(js_interpreter)
            },
            JsAstStatement::Declaration(declaration) => {
                declaration.execute(js_interpreter)
            },
            JsAstStatement::FunctionDeclaration(function_declaration) => {
                function_declaration.execute(js_interpreter);
            },
            JsAstStatement::Return(return_expression) => {
                let value = return_expression.execute(js_interpreter);
                js_interpreter.register_return_value(value);
                return false;
            },
        }
        return true;
    }
}


#[derive(Debug)]
pub struct JsAstFunctionDeclaration {
    pub name: String,
    pub arguments: Vec<JsAstIdentifier>,
    pub script: Rc<Script>,
}
impl JsAstFunctionDeclaration {
    fn execute(&self, js_interpreter: &mut JsInterpreter) {
        let global_context = js_interpreter.context_stack.iter_mut().next().unwrap();

        let argument_names = self.arguments.iter().map(|arg| arg.name.clone()).collect();
        let value = JsFunction { script: Some(self.script.clone()), argument_names: argument_names, builtin: None };

        let target_address = global_context.add_new_value(JsValue::Function(value));
        global_context.update_variable(self.name.clone(), target_address);
    }
}



#[derive(Debug)]
pub struct JsAstBinOp {
    pub op: JsBinOp,
    pub left: Rc<JsAstExpression>,
    pub right: Rc<JsAstExpression>,
}
impl JsAstBinOp {
    fn execute(&self, js_interpreter: &mut JsInterpreter) -> JsValue {
        let mut left_val = self.left.execute(js_interpreter);

        match self.op {
            JsBinOp::Plus => {
                let mut right_val = self.right.execute(js_interpreter);

                left_val = left_val.deref(js_interpreter);
                right_val = right_val.deref(js_interpreter);

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
                let mut right_val = self.right.execute(js_interpreter);

                left_val = left_val.deref(js_interpreter);
                right_val = right_val.deref(js_interpreter);

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
                let mut right_val = self.right.execute(js_interpreter);

                left_val = left_val.deref(js_interpreter);
                right_val = right_val.deref(js_interpreter);

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
                let mut right_val = self.right.execute(js_interpreter);

                left_val = left_val.deref(js_interpreter);
                right_val = right_val.deref(js_interpreter);

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
                    _ => { self.right.execute(js_interpreter) }
                };

                let object = JsValue::deref(left_val, js_interpreter);

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
pub struct JsAstAssign {
    pub left: JsAstExpression,
    pub right: JsAstExpression,
}
impl JsAstAssign {
    fn execute(&self, js_interpreter: &mut JsInterpreter) {
        let value = self.right.execute(js_interpreter);

        //TODO: not all actions might need to be in the current stack frame. Some might be globals, or from outer scopes
        let current_context = js_interpreter.context_stack.iter_mut().last().unwrap();


        let target_address = current_context.add_new_value(value);

        let mut variable_path = Vec::new();
        self.left.build_var_path(&mut variable_path);

        let mut first = true;
        let mut current_object_address = None;

        for idx in 0..variable_path.len() {
            let last = idx == variable_path.len() - 1;

            if first {
                if last {
                    current_context.update_variable(variable_path[idx].clone(), target_address);
                } else {
                    match current_context.get_var_address(&variable_path[idx]) {
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

                let object = current_context.get_value(&current_object_address.unwrap());

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
pub struct JsAstDeclaration {
    pub variable: JsAstIdentifier,
    pub initial_value: Option<JsAstExpression>,
}
impl JsAstDeclaration {
    fn execute(&self, js_interpreter: &mut JsInterpreter) {
        let initial_value = if self.initial_value.is_some() {
            self.initial_value.as_ref().unwrap().execute(js_interpreter)
        } else {
            JsValue::Undefined
        };
        let current_context = js_interpreter.context_stack.iter_mut().last().unwrap();
        let new_address = current_context.add_new_value(initial_value);

        current_context.update_variable(self.variable.name.clone(), new_address);
    }
}


#[derive(Debug)]
pub enum JsBinOp {
    Plus,
    Minus,
    Times,
    Divide,
    PropertyAccess,
}


#[derive(Debug, Clone)]
pub struct JsAstIdentifier {
    pub name: String,
}
impl JsAstIdentifier {
    fn execute(&self, js_interpreter: &mut JsInterpreter) -> JsValue {
        let opt_address = js_interpreter.get_var_address(&self.name);
        if opt_address.is_some() {
            return JsValue::Address(*opt_address.unwrap());
        }
        js_interpreter.set_error(JsError::ReferenceError);
        js_console::log_js_error(format!("variable not found: {}", self.name).as_str()); //TODO: eventually we want to trigger the logging of the error
                                                                                         //      from setting it (so we can also show stack etc.)
        return JsValue::Undefined;
    }
}


#[derive(Debug)]
pub enum JsAstExpression {
    BinOp(JsAstBinOp),
    NumericLiteral(String),
    StringLiteral(String),
    FunctionCall(JsAstFunctionCall),
    Identifier(JsAstIdentifier),
    ObjectLiteral(JsAstObjectLiteral),
}
impl JsAstExpression {
    fn execute(&self, js_interpreter: &mut JsInterpreter) -> JsValue {
        match self {
            JsAstExpression::BinOp(binop) => { return binop.execute(js_interpreter) },
            JsAstExpression::Identifier(variable) => { return JsValue::deref(variable.execute(js_interpreter), js_interpreter) },
            JsAstExpression::ObjectLiteral(obj) => { return obj.execute(js_interpreter) },

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

                let mut function = function_call.function_expression.execute(js_interpreter);
                function = function.deref(js_interpreter);

                match function {
                    JsValue::Function(function) => {
                        if function.builtin.is_some() {
                            match function.builtin.as_ref().unwrap() {
                                JsBuiltinFunction::ConsoleLog => {
                                    let to_log = function_call.arguments.get(0); //TODO: handle there being to little or to many arguments

                                    let to_log = to_log.unwrap().execute(js_interpreter);
                                    let to_log = to_log.deref(js_interpreter);

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
                                    let data = data_ast.unwrap().execute(js_interpreter); //TODO: even for tests, we probably want to handle the unwrap here
                                    let data = data.deref(js_interpreter);
                                    js_interpreter.export_test_data(data);
                                    return JsValue::Undefined;
                                }
                            }
                        } else {

                            let mut args = Vec::new();
                            for (idx, argument_name) in function.argument_names.into_iter().enumerate() {
                                let arg_ast = function_call.arguments.get(idx);
                                let arg_value = arg_ast.unwrap().execute(js_interpreter); //TODO: we need to properly handle the unwrap here
                                args.push( (argument_name, arg_value));
                            }

                            let mut new_context = JsExecutionContext::new();
                            for (arg_name, arg_value) in args {
                                let address = new_context.add_new_value(arg_value);
                                new_context.update_variable(arg_name, address);
                            }
                            js_interpreter.context_stack.push(new_context);

                            js_interpreter.run_script_with_context_stack(&function.script.unwrap());

                            js_interpreter.context_stack.pop();
                            let return_value = js_interpreter.return_value.clone();
                            js_interpreter.return_value = None;

                            if return_value.is_some() {
                                return return_value.unwrap();
                            }
                            return JsValue::Undefined;
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
pub struct JsAstFunctionCall {
    pub function_expression: Rc<JsAstExpression>,
    pub arguments: Vec<JsAstExpression>,
}


#[derive(Debug)]
pub struct JsAstObjectLiteral {
    //NOTE: for now, we only support strings as member names, but we keep expressions here as key, because eventually we need to support
    //      computed property names (using square brackets)
    pub members: Vec<(JsAstExpression, JsAstExpression)>,
}
impl JsAstObjectLiteral {
    fn execute(&self, js_interpreter: &mut JsInterpreter) -> JsValue {
        let mut members = HashMap::new();

        for (key_ast, value_ast) in self.members.iter() {

            match key_ast.execute(js_interpreter) {
                JsValue::String(property_name) => {

                    let value = value_ast.execute(js_interpreter);
                    let current_context = js_interpreter.context_stack.iter_mut().last().unwrap();
                    let address = current_context.add_new_value(value);


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
