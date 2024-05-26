use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::js_parser::Script;


pub type JsAddress = usize;


static NEXT_JS_VALUE_ADDRESS: AtomicUsize = AtomicUsize::new(1);
pub fn get_next_js_value_address() -> JsAddress { NEXT_JS_VALUE_ADDRESS.fetch_add(1, Ordering::Relaxed) }


pub struct JsExecutionContext {
    variables: HashMap<String, JsAddress>,
    values: HashMap<JsAddress, JsValue>,
    #[cfg(test)] pub last_test_data: Option<JsValue>,
}
impl JsExecutionContext {
    pub fn new() -> JsExecutionContext {
        let mut variables = HashMap::new();
        let mut values = HashMap::new();

        let console_log_function = JsValue::Function(JsFunction {
            name: String::from("log"),
            code: None,
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
                name: String::from("export"),
                code: None,
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
            #[cfg(test)] last_test_data: None,
        };
    }

    pub fn get_var_address(&self, name: &String) -> Option<&JsAddress> {
        return self.variables.get(name);
    }

    pub fn get_value(&mut self, address: &JsAddress) -> Option<&mut JsValue> {
        return self.values.get_mut(address);
    }

    pub fn update_value(&mut self, address: usize, value: JsValue) {
        self.values.insert(address, value);
    }

    pub fn update_variable(&mut self, name: String, address: usize) {
        self.variables.insert(name, address);
    }

    pub fn add_new_value(&mut self, value: JsValue) -> JsAddress {
        let new_address = get_next_js_value_address();
        self.values.insert(new_address, value);
        return new_address;
    }

    #[cfg(test)] pub fn export_test_data(&mut self, data: JsValue) {
        self.last_test_data = Some(data);
    }

    #[cfg(test)] pub fn get_last_exported_test_data(&self) -> &JsValue {
        if self.last_test_data.is_some() {
            return self.last_test_data.as_ref().unwrap();
        }
        return &JsValue::Undefined;
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
    Function(JsFunction),
    Address(JsAddress),
    Undefined,
}
impl JsValue {
    pub fn deref(self, js_execution_context: &JsExecutionContext) -> JsValue {
        match self {
            JsValue::Address(variable) => {
                //TODO: unwrap() here is wrong, we need to report an error that a variable or property does not exist
                //      or maybe we should return an option or result here, and handle it on the recieving side...
                return js_execution_context.values.get(&variable).unwrap().clone();
            },
            _ => { return self }
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
pub struct JsFunction {
    #[allow(dead_code)] pub name: String, //TODO: use
    #[allow(dead_code)] pub code: Option<Rc<Script>>, //TODO: use
    pub builtin: Option<JsBuiltinFunction>,
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub enum JsBuiltinFunction {
    ConsoleLog,
    #[cfg(test)] TesterExport,
}
