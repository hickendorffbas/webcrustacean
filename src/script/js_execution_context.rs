use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::js_ast::Script;
use super::js_interpreter::JsInterpreter;


pub type JsAddress = usize;


static NEXT_JS_VALUE_ADDRESS: AtomicUsize = AtomicUsize::new(1);
pub fn get_next_js_value_address() -> JsAddress { NEXT_JS_VALUE_ADDRESS.fetch_add(1, Ordering::Relaxed) }


pub struct JsExecutionContext {
    variables: HashMap<String, JsAddress>,
    values: HashMap<JsAddress, JsValue>,
}
impl JsExecutionContext {
    pub fn new() -> JsExecutionContext {
        //TODO: I don't think we need to create the objects on every new context, we should just set references to objects
        //      we create in the interpreter (assuming we need to have the names available at all, scoping rules would probably
        //      require us to look into higher stack frames when a var is not found anyway...)

        let mut variables = HashMap::new();
        let mut values = HashMap::new();

        let console_log_function = JsValue::Function(JsFunction {
            argument_names: Vec::new(), //Note that this function _does_ take an argument, but it does not have a name
            script: None,
            builtin: Some(JsBuiltinFunction::ConsoleLog),
        });

        let console_log_address = get_next_js_value_address();
        values.insert(console_log_address, console_log_function);

        let console_builtin = JsValue::Object(JsObject {
            members: HashMap::from([(String::from("log"), console_log_address)])
        });
        let console_object_address = get_next_js_value_address();
        values.insert(console_object_address, console_builtin);

        variables.insert(String::from("console"), console_object_address);


        #[cfg(test)] {
            let tester_export_function = JsValue::Function(JsFunction {
                argument_names: Vec::new(), //Note that this function _does_ take an argument, but it does not have a name
                script: None,
                builtin: Some(JsBuiltinFunction::TesterExport),
            });

            let tester_export_address = get_next_js_value_address();
            values.insert(tester_export_address, tester_export_function);

            let tester_builtin = JsValue::Object(JsObject {
                members: HashMap::from([(String::from("export"), tester_export_address)])
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

    pub fn update_variable(&mut self, name: String, address: usize) {
        self.variables.insert(name, address);
    }

    pub fn add_new_value(&mut self, value: JsValue) -> JsAddress {
        let new_address = get_next_js_value_address();
        self.values.insert(new_address, value);
        return new_address;
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub enum JsValue {
    Number(i32), //TODO: number type is wrong here, we need different rust types depending on what kind of number it is? (floats?)
                 //      or a more complex type maybe?
    String(String),
    #[allow(dead_code)] Boolean(bool), //TODO: use
    Object(JsObject),
    Array(JsArray),
    Function(JsFunction),
    Address(JsAddress),
    Undefined,
}
impl JsValue {
    pub fn deref(self, js_interpreter: &JsInterpreter) -> JsValue {
        match self {
            JsValue::Address(variable) => {

                //TODO: we might also need to look into higher stack items (for globals), not sure if this is always the case
                let current_context = js_interpreter.context_stack.iter().last().unwrap();

                //TODO: unwrap() here is wrong, we need to report an error that a variable or property does not exist
                //      or maybe we should return an option or result here, and handle it on the recieving side...
                return current_context.values.get(&variable).unwrap().clone();
            },
            _ => { return self }
        }
    }
    pub fn is_thruty(self) -> bool {
        match self {
            JsValue::Number(number) => { return number != 0 },
            JsValue::String(string) => { return !string.is_empty() } ,
            JsValue::Boolean(bool) => { return bool; },
            JsValue::Object(_) => todo!(),  //TODO: implement
            JsValue::Array(_) => todo!(),  //TODO: implement
            JsValue::Function(_) => todo!(),  //TODO: implement
            JsValue::Address(_) => todo!(),  //TODO: implement
            JsValue::Undefined => { return false; },
        }
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub struct JsObject {
    pub members: HashMap<String, JsAddress>,
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
