use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::js_ast::Script;


pub type JsAddress = usize;


static NEXT_JS_VALUE_ADDRESS: AtomicUsize = AtomicUsize::new(1);
pub fn get_next_js_heap_address() -> JsAddress { NEXT_JS_VALUE_ADDRESS.fetch_add(1, Ordering::Relaxed) }


pub struct JsExecutionContext {
    variables: HashMap<String, JsValue>,
    heap: HashMap<JsAddress, JsHeapObject>,  //TODO: I think this should be global on the interpreter, not per context, then we need context in less places
}
impl JsExecutionContext {
    pub fn new() -> JsExecutionContext {
        //TODO: I don't think we need to create the objects on every new context, we should just set references to objects
        //      we create in the interpreter (assuming we need to have the names available at all, scoping rules would probably
        //      require us to look into higher stack frames when a var is not found anyway...)

        let mut variables = HashMap::new();
        let mut heap = HashMap::new();

        let console_log_function = JsHeapObject::Object(JsObject::make_function(JsFunction {
            argument_names: Vec::new(), //Note that this function _does_ take an argument, but it does not have a name
            script: None,
            builtin: Some(JsBuiltinFunction::ConsoleLog),
        }));

        let console_log_address = get_next_js_heap_address();
        heap.insert(console_log_address, console_log_function);

        let console_builtin = JsHeapObject::Object(JsObject {
            members: HashMap::from([(String::from("log"), JsValue::Address(console_log_address))]), callable: None,
        });
        let console_object_address = get_next_js_heap_address();
        heap.insert(console_object_address, console_builtin);

        variables.insert(String::from("console"), JsValue::Address(console_object_address));


        #[cfg(test)] {
            let tester_export_function = JsHeapObject::Object(JsObject::make_function(JsFunction {
                argument_names: Vec::new(), //Note that this function _does_ take an argument, but it does not have a name
                script: None,
                builtin: Some(JsBuiltinFunction::TesterExport),
            }));

            let tester_export_address = get_next_js_heap_address();
            heap.insert(tester_export_address, tester_export_function);

            let tester_builtin = JsHeapObject::Object(JsObject {
                members: HashMap::from([(String::from("export"), JsValue::Address(tester_export_address))]), callable: None,
            });
            let tester_object_address = get_next_js_heap_address();
            heap.insert(tester_object_address, tester_builtin);

            variables.insert(String::from("tester"), JsValue::Address(tester_object_address));
        }

        return JsExecutionContext {
            variables,
            heap,
        };
    }

    pub fn get_from_heap(&self, address: JsAddress) -> &JsHeapObject {
        return self.heap.get(&address).unwrap();
    }

    pub fn get_callable_from_heap(&self, address: JsAddress) -> &JsFunction {
        //This function assumes we already know its a callable
        match self.get_from_heap(address) {
            JsHeapObject::Object(js_object) => {
                if js_object.callable.is_some() {
                    return js_object.callable.as_ref().unwrap();
                }
                panic!("Object is not a callable");
            },
            JsHeapObject::Array(_) => {
                panic!("Object is not a callable");
            },
        };
    }

    pub fn get_variable(&mut self, name: &String) -> Option<&mut JsValue> {
        return self.variables.get_mut(name);
    }

    pub fn set_reference(&mut self, reference: JsReference, value: JsValue) {
        match reference {
            JsReference::Variable(variable_name) => {
                self.variables.insert(variable_name, value);
            },
            JsReference::Property { object_address, member } => {
                let object = self.heap.get_mut(&object_address).unwrap();
                match object {
                    JsHeapObject::Object(ref mut js_object) => {
                        js_object.members.insert(member, value);
                    },
                    _ => {
                        todo!(); //TODO: some kind of error?
                    }
                }
            },
            JsReference::Index { object_address, index } => {
                let object = self.heap.get_mut(&object_address).unwrap();
                match object {
                    JsHeapObject::Array(ref mut js_object) => {
                        js_object.elements[index] = value;
                    },
                    _ => {
                        todo!(); //TODO: some kind of error?
                    }
                }
            },
        }
    }

    pub fn add_new_heap_item(&mut self, object: JsHeapObject) -> JsAddress {
        let new_address = get_next_js_heap_address();
        self.heap.insert(new_address, object);
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
    Address(JsAddress),
    Undefined,
}
impl JsValue {
    pub fn is_thruty(self) -> bool {
        match self {
            JsValue::Number(number) => { return number != 0 },
            JsValue::String(string) => { return !string.is_empty() } ,
            JsValue::Boolean(bool) => { return bool; },
            JsValue::Address(_) => { todo!(); }, //TODO: implement (check the heap)
            JsValue::Undefined => { return false; },
        }
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub enum JsHeapObject {
    Object(JsObject),
    Array(JsArray),
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub struct JsObject {
    pub members: HashMap<String, JsValue>,
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
    pub elements: Vec<JsValue>,
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
