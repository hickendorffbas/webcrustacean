use std::collections::HashMap;
use std::rc::Rc;

use crate::debug::debug_log_warn;

use super::js_console;
use super::js_interpreter::JsInterpreter;
use super::js_values::{
    JsArray,
    JsBuiltinFunction,
    JsError,
    JsHeapObject,
    JsFunction,
    JsReference,
    JsObject,
    JsValue,
};

pub type Script = Vec<JsAstStatement>;


#[derive(Debug)]
pub enum JsAstStatement {
    Expression(JsAstExpression),
    Declaration(Vec<JsAstDeclaration>),
    FunctionDeclaration(JsAstFunctionDeclaration),  //TODO: a function declaration is not a statement, technically, but we pretend it is for now
                                                    //      (it actually is a "source element", a statement is also a source element)
    Return(JsAstExpression),                        //TODO: it might make more sense to have return seperately on the function declaration ast node,
                                                    //      but of type JsAstStatement::Expression, instead of type JsAstStatement::Return
    Conditional(JsAstConditional),
    While(JsAstWhile),
    For(JsAstFor),
    ForEach(JsAstForEach),
}
impl JsAstStatement {
    pub fn execute(&self, js_interpreter: &mut JsInterpreter) -> bool {
        //returns a boolean saying whether to run the next statement

        match self {
            JsAstStatement::Expression(expression) => {
                let _ = expression.execute(js_interpreter);
            },
            JsAstStatement::Declaration(declarations) => {
                for decl in declarations {
                    decl.execute(js_interpreter);
                }
            },
            JsAstStatement::Return(return_expression) => {
                let value = return_expression.execute(js_interpreter);
                js_interpreter.register_return_value(value);
                return false;
            },
            JsAstStatement::FunctionDeclaration(function_declaration) => { function_declaration.execute(js_interpreter); },
            JsAstStatement::Conditional(condition_expression) => { condition_expression.execute(js_interpreter); },
            JsAstStatement::While(while_statement) => { while_statement.execute(js_interpreter); },
            JsAstStatement::For(for_statement) => { for_statement.execute(js_interpreter); },
            JsAstStatement::ForEach(for_statement) => { for_statement.execute(js_interpreter); },
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
        let argument_names = self.arguments.iter().map(|arg| arg.name.clone()).collect();
        let function = JsFunction { script: Some(self.script.clone()), argument_names: argument_names, builtin: None };

        let address = js_interpreter.add_new_heap_item(JsHeapObject::Object(JsObject::make_function(function)));
        js_interpreter.set_reference(JsReference::Variable(self.name.clone()), JsValue::Address(address));
    }
}


#[derive(Debug)]
pub struct JsAstFunctionExpression {
    #[allow(unused)] pub name: Option<String>, //TODO: currently unused, but function expressions with a name exist
    pub arguments: Vec<JsAstIdentifier>,
    pub script: Rc<Script>,
}
impl JsAstFunctionExpression {
    fn execute(&self, js_interpreter: &mut JsInterpreter) -> JsValue {
        let argument_names = self.arguments.iter().map(|arg| arg.name.clone()).collect();
        let function = JsFunction { script: Some(self.script.clone()), argument_names: argument_names, builtin: None };
        let address = js_interpreter.add_new_heap_item(JsHeapObject::Object(JsObject::make_function(function)));
        return JsValue::Address(address);
    }
}


#[derive(Debug)]
pub struct JsAstConditional {
    pub condition: Rc<JsAstExpression>,
    pub script: Rc<Script>,
    pub else_script: Option<Rc<Script>>,
}
impl JsAstConditional {
    fn execute(&self, js_interpreter: &mut JsInterpreter) {
        let result = self.condition.execute(js_interpreter);

        if result.is_thruty() {

            for statement in self.script.iter() {
                let keep_going = statement.execute(js_interpreter);
                if !keep_going {
                    return;
                }
            };

        } else {

            if self.else_script.is_some() {
                for statement in self.else_script.as_ref().unwrap().iter() {
                    let keep_going = statement.execute(js_interpreter);
                    if !keep_going {
                        return;
                    }
                };
            }

        }
    }
}


#[derive(Debug)]
pub struct JsAstWhile {
    pub condition: Rc<JsAstExpression>,
    pub script: Rc<Script>,
}
impl JsAstWhile {
    fn execute(&self, js_interpreter: &mut JsInterpreter) {

        loop {
            let condition_result = self.condition.execute(js_interpreter);
            if !condition_result.is_thruty() {
                break;
            }

            for statement in self.script.iter() {
                let keep_going = statement.execute(js_interpreter);
                if !keep_going {
                    return;
                }
            };
        }
    }
}


#[derive(Debug)]
pub struct JsAstFor {
    //NOTE: we expect either initial expression, or intial declarations because both are allowed in the first expression of the for()
    pub initial_expression: Option<Rc<JsAstExpression>>,
    pub initial_declarations: Option<Rc<Vec<JsAstDeclaration>>>,

    pub loop_condition: Rc<JsAstExpression>,
    pub next_step_expression: Rc<JsAstExpression>,
    pub script: Rc<Script>,
}
impl JsAstFor {
    fn execute(&self, js_interpreter: &mut JsInterpreter) {
        if self.initial_declarations.is_some() {
            for node in self.initial_declarations.as_ref().unwrap().iter() {
                node.execute(js_interpreter);
            }
        }

        if self.initial_expression.is_some() {
            self.initial_expression.as_ref().unwrap().execute(js_interpreter);
        }

        loop {
            let condition_value = self.loop_condition.execute(js_interpreter);
            if !condition_value.is_thruty() {
                break;
            }

            for statement in self.script.iter() {
                statement.execute(js_interpreter);
            }

            self.next_step_expression.execute(js_interpreter);
        }
    }
}


#[derive(Debug)]
pub struct JsAstForEach {
    pub initial_expression: Option<Rc<JsAstExpression>>,
    pub initial_declarations: Option<Rc<Vec<JsAstDeclaration>>>,

    pub iteration_target: Rc<JsAstExpression>,
    pub script: Rc<Script>,
}
impl JsAstForEach {
    fn execute(&self, js_interpreter: &mut JsInterpreter) {
        let iteration_reference = if self.initial_expression.is_some() {
            self.initial_expression.as_ref().unwrap().execute_for_reference(js_interpreter).unwrap()
        } else {
            // In a for ... in ....  we can only have a single declaration:
            let declaration = self.initial_declarations.as_ref().unwrap().first().unwrap();
            declaration.execute_for_reference(js_interpreter)
        };

        let object = self.iteration_target.execute(js_interpreter);

        match &object {
            JsValue::Address(address) => {
                match js_interpreter.get_from_heap(*address) {
                    JsHeapObject::Object(js_object) => {
                        let keys = js_object.members.keys().cloned().collect::<Vec<_>>();

                        for key in keys {
                            js_interpreter.set_reference(iteration_reference.clone(), JsValue::String(key.clone()));
                            for statement in self.script.iter() {
                                statement.execute(js_interpreter);
                            }
                        }
                    },
                    JsHeapObject::Array(_) => {
                        todo!(); //TODO: this should be an error (foreach is not supported on arrays)
                    },
                };
            },
            _ => {
                todo!(); //TODO: this should always be an error
            }
        };
    }
}


#[derive(Debug)]
pub struct JsAstTernary {
    pub condition: Rc<JsAstExpression>,
    pub if_true: Rc<JsAstExpression>,
    pub if_false: Rc<JsAstExpression>,
}
impl JsAstTernary {
    fn execute(&self, js_interpreter: &mut JsInterpreter) -> JsValue {
        let result = self.condition.execute(js_interpreter);

        if result.is_thruty() {
            return self.if_true.execute(js_interpreter);
        } else {
            return self.if_false.execute(js_interpreter);
        }
    }
}


#[derive(Debug)]
pub struct JsAstBinOp {
    pub op: JsBinOp,
    pub left: Rc<JsAstExpression>,
    pub right: Rc<JsAstExpression>,
    pub is_dot_property_access: bool, //TODO: this is only needed for propertyAccess, we might want to make a seperate AST for that
}
impl JsAstBinOp {
    fn execute(&self, js_interpreter: &mut JsInterpreter) -> JsValue {
        let left_val = self.left.execute(js_interpreter);

        match self.op {
            JsBinOp::Plus => {
                let right_val = self.right.execute(js_interpreter);

                match left_val {
                    JsValue::Number(left_number) => {
                        match right_val {
                            JsValue::Number(right_number) => {
                                return JsValue::Number(left_number + right_number);
                            },
                            _ => { todo!(); }
                        }
                    },
                    JsValue::String(left_string) => {
                        match right_val {
                            JsValue::String(right_string) => {
                                return JsValue::String(left_string + &right_string);
                            },
                            _ => { todo!(); }
                        }
                    }
                    _ => { todo!() }
                }
            },
            JsBinOp::Minus => {
                let right_val = self.right.execute(js_interpreter);

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
                let right_val = self.right.execute(js_interpreter);

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
                let right_val = self.right.execute(js_interpreter);

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
                let property = if self.is_dot_property_access {
                    match self.right.as_ref() {
                        // when the right hand side of our accessor is an identifier, we don't execute, but just take its name as a string
                        // this is because a.b is equivalent to a["b"]
                        JsAstExpression::Identifier(ident) => { JsValue::String(ident.name.clone()) }
                        _ => { self.right.execute(js_interpreter) }
                    }
                } else {
                    self.right.execute(js_interpreter)
                };

                let object = match left_val {
                    JsValue::Address(address) => {
                        js_interpreter.get_from_heap(address)
                    },
                    _ => {
                        todo!(); //TODO: this should always be an error
                    }
                };

                match object {
                    JsHeapObject::Object(object) => {
                        match &property {
                            JsValue::String(property_value) => {
                                match object.members.get(property_value) {
                                    Some(value) => { return value.clone() },
                                    None => todo!(),  //TODO: this should be an error
                                }
                            },
                            _ => {
                                //TODO: some of these are invalid, others (like number) are valid (for example for "x[3]")
                                todo!();
                            }
                        }
                    },
                    JsHeapObject::Array(array) => {
                        match property {
                            JsValue::Number(number) => {
                                match array.elements.get(number as usize) {
                                    Some(value) => { return value.clone() },
                                    None => todo!(),  //TODO: this should be an error
                                }
                            },
                            _ => {
                                //TODO: there are some standard functions and attributes you can call on an array, not sure how to handle that nicely
                                //      we should probably defer to the same code that an object has in those cases
                                return JsValue::Undefined;
                            }
                        }
                    },
                }
            },
            JsBinOp::Equals => {
                let right_val = self.right.execute(js_interpreter);

                match left_val {
                    JsValue::Number(left_number) => {
                        match right_val {
                            JsValue::Number(right_number) => {
                                return JsValue::Boolean(left_number == right_number);
                            },
                            _ => { todo!() }
                        }
                    },
                    JsValue::String(left_string) => {
                        match right_val {
                            JsValue::String(right_string) => {
                                return JsValue::Boolean(left_string == right_string);
                            },
                            _ => { todo!() }
                        }
                    },
                    _ => { todo!() }
                }
            },
            JsBinOp::EqualsStrict => {
                let right_val = self.right.execute(js_interpreter);

                match left_val {
                    JsValue::Number(left_number) => {
                        match right_val {
                            JsValue::Number(right_number) => {
                                return JsValue::Boolean(left_number == right_number);
                            },
                            _ => { return JsValue::Boolean(false); }
                        }
                    },
                    JsValue::String(left_string) => {
                        match right_val {
                            JsValue::String(right_string) => {
                                return JsValue::Boolean(left_string == right_string);
                            },
                            _ => { return JsValue::Boolean(false); }
                        }
                    },
                    _ => { todo!() }
                }
            }
            JsBinOp::LogicalAnd => {
                let right_val = self.right.execute(js_interpreter);

                match left_val {
                    JsValue::Undefined => { return left_val; },
                    JsValue::Boolean(left_bool) => {
                        match right_val {
                            JsValue::Boolean(right_bool) => { return JsValue::Boolean(left_bool && right_bool); },
                            _ => { todo!(); }
                        }
                    }
                    _ => { todo!() }
                }
            },
            JsBinOp::LogicalOr => {
                let right_val = self.right.execute(js_interpreter);

                match left_val {
                    JsValue::Undefined => { return right_val; },
                    JsValue::Boolean(left_bool) => {
                        match right_val {
                            JsValue::Boolean(right_bool) => { return JsValue::Boolean(left_bool || right_bool); },
                            _ => { todo!(); }
                        }
                    }
                    _ => { todo!() }
                }
            },
            JsBinOp::BitWiseOr => {
                let right_val = self.right.execute(js_interpreter);

                match left_val {
                    JsValue::Number(num) => {
                        match right_val {
                            JsValue::Number(other_num) => {
                                return JsValue::Number(num | other_num);
                            },
                            _ => { todo!() }
                        }
                    },
                    _ => { todo!() }
                }
            },
            JsBinOp::BitWiseXor => {
                let right_val = self.right.execute(js_interpreter);

                match left_val {
                    JsValue::Number(num) => {
                        match right_val {
                            JsValue::Number(other_num) => {
                                return JsValue::Number(num ^ other_num);
                            },
                            _ => { todo!() }
                        }
                    },
                    _ => { todo!() }
                }
            },
            JsBinOp::BitWiseAnd => {
                let right_val = self.right.execute(js_interpreter);

                match left_val {
                    JsValue::Number(num) => {
                        match right_val {
                            JsValue::Number(other_num) => {
                                return JsValue::Number(num & other_num);
                            },
                            _ => { todo!() }
                        }
                    },
                    _ => { todo!() }
                }
            }
            JsBinOp::Comma => {
                return self.right.execute(js_interpreter);
            },
            JsBinOp::LeftShift => {
                let right_val = self.right.execute(js_interpreter);

                match left_val {
                    JsValue::Number(left_number) => {
                        match right_val {
                            JsValue::Number(right_number) => {
                                return JsValue::Number(left_number << right_number);
                            }
                            _ => todo!(), //TODO: probably needs to be an error
                        }
                    },
                    _ => todo!(), //TODO: probably needs to be an error
                }
            },
            JsBinOp::RightShift => {
                let right_val = self.right.execute(js_interpreter);

                match left_val {
                    JsValue::Number(left_number) => {
                        match right_val {
                            JsValue::Number(right_number) => {
                                return JsValue::Number(left_number >> right_number);
                            }
                            _ => todo!(), //TODO: probably needs to be an error
                        }
                    },
                    _ => todo!(), //TODO: probably needs to be an error
                }
            },
            JsBinOp::Bigger => {
                let right_val = self.right.execute(js_interpreter);

                match left_val {
                    JsValue::Number(left_number) => {
                        match right_val {
                            JsValue::Number(right_number) => {
                                return JsValue::Boolean(left_number > right_number);
                            },
                            _ => { todo!() }
                        }
                    },
                    _ => { todo!() }
                }
            },
            JsBinOp::Smaller => {
                let right_val = self.right.execute(js_interpreter);

                match left_val {
                    JsValue::Number(left_number) => {
                        match right_val {
                            JsValue::Number(right_number) => {
                                return JsValue::Boolean(left_number < right_number);
                            },
                            _ => { todo!() }
                        }
                    },
                    _ => { todo!() }
                }
            },
            JsBinOp::In => {
                let right_val = self.right.execute(js_interpreter);

                match right_val {
                    JsValue::Address(address) => {
                        match js_interpreter.get_from_heap(address) {
                            JsHeapObject::Object(js_object) => {
                                match left_val {
                                    JsValue::String(member_to_check) => {
                                        return JsValue::Boolean(js_object.members.contains_key(&member_to_check));
                                    },
                                    _ => {
                                        todo!(); //TODO: number indexes like "3 in x" , seems to be coerced to a string, other types seem not allowed
                                    }
                                }
                            },
                            JsHeapObject::Array(_) => {
                                todo!(); //TODO: I think this is always an error
                            },
                        }
                    },
                    _ => {
                        todo!(); //TODO: this should be an error
                    }
                }
            },
            JsBinOp::Remainder => {
                let right_val = self.right.execute(js_interpreter);

                match left_val {
                    JsValue::Number(left_number) => {
                        match right_val {
                            JsValue::Number(right_number) => {
                                return JsValue::Number(left_number % right_number);
                            },
                            _ => { todo!() }
                        }
                    },
                    _ => { todo!() }
                }
            },
        }
    }
}


#[derive(Debug)]
pub struct JsAstAssign {
    pub left: Rc<JsAstExpression>,
    pub right: Rc<JsAstExpression>,
}
impl JsAstAssign {
    fn execute(&self, js_interpreter: &mut JsInterpreter) {
        let opt_assignment_reference = self.left.execute_for_reference(js_interpreter);
        let value = self.right.execute(js_interpreter);

        match opt_assignment_reference {
            Some(assignment_reference) => {
                js_interpreter.set_reference(assignment_reference, value);
            },
            None => {
                js_console::log_js_error("Assignment failed, no valid target"); //TODO: this should include a line number (we need to build that generically)
                //TODO: we should stop evaluating on these kind of errors, so we should probably return a result or something
            },
        }
    }
}


#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DeclType {
    Var,
    Let,
    Const,
}


#[derive(Debug)]
pub struct JsAstDeclaration {
    #[allow(dead_code)] pub decl_type: DeclType, //TODO: use, and remove dead code attribute
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
        js_interpreter.set_reference(JsReference::Variable(self.variable.name.clone()), initial_value);
    }

    fn execute_for_reference(&self, js_interpreter: &mut JsInterpreter) -> JsReference {
        self.execute(js_interpreter);
        return JsReference::Variable(self.variable.name.clone());
    }
}


#[derive(Debug)]
pub enum JsBinOp {
    Plus,
    Minus,
    Times,
    Divide,
    PropertyAccess,
    Equals,
    EqualsStrict,
    LogicalAnd,
    LogicalOr,
    BitWiseOr,
    BitWiseXor,
    BitWiseAnd,
    Comma,
    LeftShift,
    RightShift,
    Bigger,
    Smaller,
    In,
    Remainder,
}


#[derive(Debug)]
pub enum JsUnOp {
    Plus,
    Minus,
    Not,
    PostfixIncrement,
    PostfixDecrement,
    #[allow(dead_code)] PrefixIncrement, //TODO: implement
    #[allow(dead_code)] PrefixDecrement, //TODO: implement
}


#[derive(Debug)]
pub struct JsAstUnOp {
    pub op: JsUnOp,
    pub operand: Rc<JsAstExpression>,
}
impl JsAstUnOp {
    fn execute(&self, js_interpreter: &mut JsInterpreter) -> JsValue {
        match self.op {
            JsUnOp::Plus => {
                let operand = self.operand.execute(js_interpreter);
                match operand {
                    JsValue::Number(number) => return JsValue::Number(number),
                    _ => { todo!() }, //TODO: most of the others are not valid, implement errors
                }
            },
            JsUnOp::Minus => {
                let operand = self.operand.execute(js_interpreter);
                match operand {
                    JsValue::Number(number) => return JsValue::Number(-number),
                    _ => { todo!() }, //TODO: most of the others are not valid, implement errors
                }
            },
            JsUnOp::Not => {
                let operand = self.operand.execute(js_interpreter);
                match operand {
                    JsValue::Boolean(bool) => return JsValue::Boolean(!bool),
                    _ => { todo!() }, //TODO: most of the others are not valid, implement errors
                }
            }
            JsUnOp::PostfixIncrement => {
                let operand_reference = self.operand.execute_for_reference(js_interpreter);
                match operand_reference {
                    Some(reference) => {
                        let target = js_interpreter.get_by_reference(reference);
                        match target {
                            Some(value) => {
                                match value {
                                    JsValue::Number(num) => {
                                        let original = *num;
                                        *num += 1;
                                        return JsValue::Number(original);
                                    },
                                    _ => {
                                        todo!(); //TODO: some kind of error
                                    }
                                }
                            }
                            None => todo!(), //TODO: some kind of error
                        }
                    },
                    None => {
                        todo!(); //TODO: this should become an error, you are trying to increment something that does not resolve to an address
                    },
                }
            },
            JsUnOp::PostfixDecrement => todo!(), //TODO: implement
            JsUnOp::PrefixIncrement => todo!(), //TODO: implement
            JsUnOp::PrefixDecrement => todo!(), //TODO: implement
        }
    }
}


#[derive(Debug, Clone)]
pub struct JsAstIdentifier {
    pub name: String,
}
impl JsAstIdentifier {
    fn execute(&self, js_interpreter: &mut JsInterpreter) -> JsValue {

        match js_interpreter.get_by_reference(JsReference::Variable(self.name.clone())) {
            Some(value) => return value.clone(),
            None => {
                js_interpreter.set_error(JsError::ReferenceError);
                js_console::log_js_error(format!("variable not found: {}", self.name).as_str()); //TODO: eventually we want to trigger the logging of the error
                                                                                                 //      from setting it (so we can also show stack etc.)
                return JsValue::Undefined;
            },
        }
    }
}


#[derive(Debug)]
pub enum JsAstExpression {
    BinOp(JsAstBinOp),
    UnaryOp(JsAstUnOp),
    Ternary(JsAstTernary),
    NumericLiteral(String),
    StringLiteral(String),
    BooleanLiteral(bool),
    UndefinedLiteral(),
    FunctionCall(JsAstFunctionCall),
    Identifier(JsAstIdentifier),
    ObjectLiteral(JsAstObjectLiteral),
    ArrayLiteral(JsAstArrayLiteral),
    Assignment(JsAstAssign),
    FunctionExpression(JsAstFunctionExpression),
    RegexLiteral(JsAstRegexLiteral),
    ObjectCreation(JsAstObjectCreation),
    TypeOf(JsAstTypeOf),
}
impl JsAstExpression {
    fn execute(&self, js_interpreter: &mut JsInterpreter) -> JsValue {
        match self {
            JsAstExpression::BinOp(binop) => { return binop.execute(js_interpreter); },
            JsAstExpression::UnaryOp(unop) => { return unop.execute(js_interpreter); },
            JsAstExpression::Ternary(ternary) => { return ternary.execute(js_interpreter); }
            JsAstExpression::Identifier(variable) => { return variable.execute(js_interpreter); },
            JsAstExpression::ObjectLiteral(obj) => { return obj.execute(js_interpreter); },
            JsAstExpression::ArrayLiteral(array) => { return array.execute(js_interpreter); },
            JsAstExpression::FunctionExpression(js_ast_function_expression) => { return js_ast_function_expression.execute(js_interpreter); },
            JsAstExpression::RegexLiteral(regex_literal) => { return regex_literal.execute(); },
            JsAstExpression::ObjectCreation(object_construction) => { return object_construction.execute(); },
            JsAstExpression::BooleanLiteral(boolean_value) => { return JsValue::Boolean(*boolean_value); },
            JsAstExpression::StringLiteral(string_literal) => { return JsValue::String(string_literal.clone()); },
            JsAstExpression::UndefinedLiteral() => { return JsValue::Undefined; },
            JsAstExpression::TypeOf(type_of) => { return type_of.execute(js_interpreter); },
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
            JsAstExpression::FunctionCall(function_call) => {
                //TODO: all this code should be moved to the JsAstFunctionCall object

                let function = function_call.function_expression.execute(js_interpreter);

                match function {
                    JsValue::Address(address) => {
                        match js_interpreter.get_from_heap(address) {
                            JsHeapObject::Object(js_object) => {
                                if js_object.callable.is_none() {
                                    //TODO: report an error (non-callable object)
                                    return JsValue::Undefined;
                                }
                            },
                            JsHeapObject::Array(_) => {
                                //TODO: report an error (non-callable object)
                                return JsValue::Undefined;
                            },
                        };

                        let is_builtin = js_interpreter.get_callable_from_heap(address).builtin.is_some();
                        if is_builtin {
                            match js_interpreter.get_callable_from_heap(address).builtin.as_ref().unwrap() {
                                JsBuiltinFunction::ConsoleLog => {
                                    let to_log = function_call.arguments.get(0); //TODO: handle there being to little or to many arguments

                                    let to_log = to_log.unwrap().execute(js_interpreter);

                                    let to_log = match to_log {
                                        JsValue::String(string) =>  { string }
                                        JsValue::Number(number) => { number.to_string() },
                                        JsValue::Boolean(bool) => { if bool { "true".to_owned() } else { "false".to_owned() } },
                                        JsValue::Undefined => { "undefined".to_owned() },
                                        JsValue::Address(_) => todo!(), //TODO: implement
                                    };

                                    js_console::print(to_log.as_str());
                                    return JsValue::Undefined;
                                },
                                #[cfg(test)] JsBuiltinFunction::TesterExport => {
                                    let data_ast = function_call.arguments.get(0);
                                    let data = data_ast.unwrap().execute(js_interpreter); //TODO: even for tests, we probably want to handle the unwrap here
                                    js_interpreter.export_test_data(data);
                                    return JsValue::Undefined;
                                }
                            }

                        } else {
                            let mut args = Vec::new();
                            let argument_names = js_interpreter.get_callable_from_heap(address).argument_names.clone();
                            for (idx, argument_name) in argument_names.into_iter().enumerate() {
                                let arg_ast = function_call.arguments.get(idx);
                                let arg_value = arg_ast.unwrap().execute(js_interpreter); //TODO: we need to properly handle the unwrap here
                                args.push( (argument_name, arg_value));
                            }

                            let script = &js_interpreter.get_callable_from_heap(address).script;
                            js_interpreter.run_script(&script.as_ref().unwrap().clone(), args);

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
            JsAstExpression::Assignment(js_ast_assign) => {
                js_ast_assign.execute(js_interpreter);
                return JsValue::Undefined; //TODO: I think an assignment expression should return its value, we need to fix that if so
            },
        }
    }

    fn execute_for_reference(&self, js_interpreter: &mut JsInterpreter) -> Option<JsReference> {
        match self {
            JsAstExpression::Identifier(ident) => {
                return Some(JsReference::Variable(ident.name.clone()));
            },
            JsAstExpression::BinOp(binop) => {
                match binop.op {
                    JsBinOp::PropertyAccess => {
                        let left_value = binop.left.execute(js_interpreter);

                        let property = if binop.is_dot_property_access {
                            match binop.right.as_ref() {
                                // when the right hand side of our accessor is an identifier, we don't execute, but just take its name as a string
                                // this is because a.b is equivalent to a["b"]
                                JsAstExpression::Identifier(ident) => { JsValue::String(ident.name.clone()) }
                                _ => { binop.right.execute(js_interpreter) }
                            }
                        } else {
                            binop.right.execute(js_interpreter)
                        };

                        match left_value {
                            JsValue::Address(left_address) => {
                                match property {
                                    JsValue::Number(index) => {
                                        return Some(JsReference::Index { object_address: left_address, index: index as usize })
                                    },
                                    JsValue::String(member) => {
                                        return Some(JsReference::Property { object_address: left_address, member });
                                    },
                                    _ => {
                                        todo!(); //TODO: most of these should be an error or None, but maybe some are valid?
                                    }
                                }
                            },
                            JsValue::Undefined => {
                                js_console::log_js_error("Can't access property of undefined"); //TODO: this should include a line number (we need to build that generically)
                                //TODO: we should stop evaluating on these kind of errors, so we should probably return a result or something
                                return None;
                            },
                            _ => {
                                todo!();
                            }
                        }
                    },
                    _ => { return None; }
                }
            },
            _ => { return None; }
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
                    members.insert(property_name, value_ast.execute(js_interpreter));
                },
                _ => {
                    todo!(); //TODO: this should be an error
                }
            }
        }

        let address = js_interpreter.add_new_heap_item(JsHeapObject::Object(JsObject { members, callable: None }));
        return JsValue::Address(address);
    }
}


#[derive(Debug)]
pub struct JsAstArrayLiteral {
    pub elements: Vec<JsAstExpression>,
}
impl JsAstArrayLiteral {
    fn execute(&self, js_interpreter: &mut JsInterpreter) -> JsValue {
        let mut elements = Vec::new();

        for value_ast in self.elements.iter() {
            let value = value_ast.execute(js_interpreter);
            elements.push(value);
        }

        let address = js_interpreter.add_new_heap_item(JsHeapObject::Array(JsArray { elements: elements }));
        return JsValue::Address(address);
    }
}


#[derive(Debug)]
pub struct JsAstRegexLiteral {
    #[allow(unused)] pub regex: String, //TODO: remove unused when implemented
}
impl JsAstRegexLiteral {
    fn execute(&self) -> JsValue {
        debug_log_warn("Literal Regexes are not yet supported in javascript");
        return JsValue::Undefined;
    }
}


#[derive(Debug)]
pub struct JsAstObjectCreation {
    #[allow(unused)] pub constructor: JsAstFunctionCall, //TODO: remove unused when implemented
}
impl JsAstObjectCreation {
    fn execute(&self) -> JsValue {

        //TODO: we need to call the constructor function, but also do some extra work like instantiating an object etc.

        debug_log_warn("Object creation is not yet supported in javascript");
        return JsValue::Undefined;
    }
}


#[derive(Debug)]
pub struct JsAstTypeOf {
    pub expression: Rc<JsAstExpression>,
}
impl JsAstTypeOf {
    fn execute(&self, js_interpreter: &mut JsInterpreter) -> JsValue {
        let value = self.expression.execute(js_interpreter);

        return match &value {
            JsValue::Number(_) => JsValue::String(String::from("number")),
            JsValue::String(_) => JsValue::String(String::from("string")),
            JsValue::Boolean(_) => JsValue::String(String::from("boolean")),
            JsValue::Address(address) => {
                match js_interpreter.get_from_heap(*address) {
                    JsHeapObject::Object(js_object) => {
                        if js_object.callable.is_some() {
                            JsValue::String(String::from("function"))
                        } else {
                            JsValue::String(String::from("object"))
                        }
                    },
                    JsHeapObject::Array(_) => JsValue::String(String::from("object")),
                }

            }
            JsValue::Undefined => JsValue::String(String::from("undefined")),
        }
    }
}
