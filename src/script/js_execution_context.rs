use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::script::js_interpreter::JsInterpreter;

use super::js_ast::Script;


pub type JsAddress = usize;


static NEXT_JS_VALUE_ADDRESS: AtomicUsize = AtomicUsize::new(1);
pub fn get_next_js_value_address() -> JsAddress { NEXT_JS_VALUE_ADDRESS.fetch_add(1, Ordering::Relaxed) }


pub struct JsExecutionContext {
    variables: HashMap<String, JsAddress>,
    values: HashMap<JsAddress, JsValue>,  //TODO: I think this should be global on the interpreter, not per context, then we need context in less places
}
impl JsExecutionContext {
    pub fn new() -> JsExecutionContext {
        //TODO: I don't think we need to create the objects on every new context, we should just set references to objects
        //      we create in the interpreter (assuming we need to have the names available at all, scoping rules would probably
        //      require us to look into higher stack frames when a var is not found anyway...)

        let mut variables = HashMap::new();
        let mut values = HashMap::new();

        let console_log_function = JsValue::Object(JsObject::make_function(JsFunction {
            argument_names: Vec::new(), //Note that this function _does_ take an argument, but it does not have a name
            script: None,
            builtin: Some(JsBuiltinFunction::ConsoleLog),
        }));

        let console_log_address = get_next_js_value_address();
        values.insert(console_log_address, console_log_function);

        let console_builtin = JsValue::Object(JsObject {
            members: HashMap::from([(String::from("log"), console_log_address)]), callable: None,
        });
        let console_object_address = get_next_js_value_address();
        values.insert(console_object_address, console_builtin);

        variables.insert(String::from("console"), console_object_address);


        #[cfg(test)] {
            let tester_export_function = JsValue::Object(JsObject::make_function(JsFunction {
                argument_names: Vec::new(), //Note that this function _does_ take an argument, but it does not have a name
                script: None,
                builtin: Some(JsBuiltinFunction::TesterExport),
            }));

            let tester_export_address = get_next_js_value_address();
            values.insert(tester_export_address, tester_export_function);

            let tester_builtin = JsValue::Object(JsObject {
                members: HashMap::from([(String::from("export"), tester_export_address)]), callable: None,
            });
            let tester_object_address = get_next_js_value_address();
            values.insert(tester_object_address, tester_builtin);

            variables.insert(String::from("tester"), tester_object_address);
        }

        return JsExecutionContext {
            variables,
            values,
        };
    }

    pub fn get_var_address(&self, name: &String) -> Option<&JsAddress> {
        return self.variables.get(name);
    }

    pub fn get_value(&mut self, address: &JsAddress) -> Option<&mut JsValue> {
        return self.values.get_mut(address);
    }

    pub fn set_reference(&mut self, reference: JsReference, address: JsAddress) {
        match reference {
            JsReference::Variable(variable_name) => {
                self.variables.insert(variable_name, address);
            },
            JsReference::Property { object_address, member } => {
                let object = self.values.get_mut(&object_address).unwrap();
                match object {
                    JsValue::Object(ref mut js_object) => {
                        js_object.members.insert(member, address);
                    },
                    _ => {
                        todo!(); //TODO: some kind of error?
                    }
                }
            },
            JsReference::Index { object_address, index } => {
                let object = self.values.get_mut(&object_address).unwrap();
                match object {
                    JsValue::Array(ref mut js_object) => {
                        js_object.elements[index] = address;
                    },
                    _ => {
                        todo!(); //TODO: some kind of error?
                    }
                }
            },
        }
    }

    pub fn add_new_value(&mut self, value: JsValue) -> JsAddress {
        let new_address = get_next_js_value_address();
        self.values.insert(new_address, value);
        return new_address;
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub enum JsReference {
    Variable(String),
    Property { object_address: JsAddress, member: String },
    Index { object_address: JsAddress, index: usize },
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub enum JsValue {
    Number(i64), //TODO: number type is wrong here, we need different rust types depending on what kind of number it is? (floats?)
                 //      or a more complex type maybe?
    String(String),
    Boolean(bool),
    Object(JsObject),
    Array(JsArray), //TODO: this should become an optional member on object
    Address(JsAddress),
    Undefined,
}
impl JsValue {
    pub fn is_thruty(self, js_interpreter: &mut JsInterpreter) -> bool {
        match js_interpreter.deref(&self) {
            JsValue::Number(number) => { return number != 0 },
            JsValue::String(string) => { return !string.is_empty() } ,
            JsValue::Boolean(bool) => { return bool; },
            JsValue::Object(_) => todo!(),  //TODO: implement
            JsValue::Array(_) => todo!(),  //TODO: implement
            JsValue::Address(_) => { panic!("unreachable"); },  //we should not be able to have an address after dereferencing
            JsValue::Undefined => { return false; },
        }
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub struct JsObject {
    pub members: HashMap<String, JsAddress>,
    pub callable: Option<JsFunction>,
}
impl JsObject {
    pub fn make_function(func: JsFunction) -> JsObject {
        return JsObject { members: HashMap::new(), callable: Some(func) };
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub struct JsArray {
    pub elements: Vec<JsAddress>,
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub struct JsFunction {
    pub script: Option<Rc<Script>>,
    pub argument_names: Vec<String>,
    pub builtin: Option<JsBuiltinFunction>,
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub enum JsBuiltinFunction {
    ConsoleLog,
    #[cfg(test)] TesterExport,
}


pub enum JsError {
    //NOTE: these are runtime errors, not parse-time errors (i.e. these are errors you can catch in a script)
    ReferenceError, //TODO: give the specific errors extra information (like here, what reference, and on what position in the script etc)
}
