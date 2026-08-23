use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use super::js_ast::Script;
use super::js_values::{
    JsAddress,
    JsBuiltinFunction,
    JsError,
    JsFunction,
    JsHeapObject,
    JsObject,
    JsReference,
    JsValue,
};


static NEXT_JS_HEAP_ADDRESS: AtomicUsize = AtomicUsize::new(1);
pub fn get_next_js_heap_address() -> JsAddress { NEXT_JS_HEAP_ADDRESS.fetch_add(1, Ordering::Relaxed) }


pub struct JsInterpreter {
    pub context_stack: Vec<HashMap<String, JsValue>>,
    pub heap: HashMap<JsAddress, JsHeapObject>,
    current_error: Option<JsError>,
    pub return_value: Option<JsValue>,
    #[cfg(test)] pub last_test_data: Option<JsValue>,
}

impl JsInterpreter {
    pub fn new() -> JsInterpreter {
        let mut interpreter = JsInterpreter {
            context_stack: Vec::new(),
            heap: HashMap::new(),
            current_error: None,
            return_value: None,
            #[cfg(test)] last_test_data: None,
        };

        interpreter.start_new_context();
        interpreter.create_builtins();
        return interpreter;
    }

    fn create_builtins(&mut self) {
        let console_log_function = JsHeapObject::Object(JsObject::make_function(JsFunction {
            argument_names: Vec::new(), //Note that this function _does_ take an argument, but it does not have a name
            script: None,
            builtin: Some(JsBuiltinFunction::ConsoleLog),
        }));

        let console_log_address = get_next_js_heap_address();
        self.heap.insert(console_log_address, console_log_function);

        let console_builtin = JsHeapObject::Object(JsObject {
            members: HashMap::from([(String::from("log"), JsValue::Address(console_log_address))]), callable: None,
        });
        let console_object_address = get_next_js_heap_address();
        self.heap.insert(console_object_address, console_builtin);

        self.context_stack.last_mut().unwrap().insert(String::from("console"), JsValue::Address(console_object_address));

        #[cfg(test)] {
            let tester_export_function = JsHeapObject::Object(JsObject::make_function(JsFunction {
                argument_names: Vec::new(), //Note that this function _does_ take an argument, but it does not have a name
                script: None,
                builtin: Some(JsBuiltinFunction::TesterExport),
            }));

            let tester_export_address = get_next_js_heap_address();
            self.heap.insert(tester_export_address, tester_export_function);

            let tester_builtin = JsHeapObject::Object(JsObject {
                members: HashMap::from([(String::from("export"), JsValue::Address(tester_export_address))]), callable: None,
            });
            let tester_object_address = get_next_js_heap_address();
            self.heap.insert(tester_object_address, tester_builtin);

            self.context_stack.last_mut().unwrap().insert(String::from("tester"), JsValue::Address(tester_object_address));
        }
    }

    pub fn start_new_context(&mut self) {
        self.context_stack.push(HashMap::new());
    }

    pub fn register_return_value(&mut self, return_value: JsValue) {
        self.return_value = Some(return_value);
    }

    pub fn set_error(&mut self, error: JsError) {
        self.current_error = Some(error);
    }

    pub fn run_script(&mut self, script: &Script, arguments: Vec<(String, JsValue)>) {
        self.start_new_context();

        for (arg_name, arg_value) in arguments {
            self.set_reference(JsReference::Variable(arg_name), arg_value);
        }
        for statement in script {
            if !statement.execute(self) {
                return;
            }
        }
        self.context_stack.pop();
    }

    pub fn get_from_heap(&self, address: JsAddress) -> &JsHeapObject {
        return self.heap.get(&address).unwrap();
    }

    pub fn set_reference(&mut self, reference: JsReference, value: JsValue) {
        match reference {
            JsReference::Variable(variable_name) => {
                self.context_stack.last_mut().unwrap().insert(variable_name, value);
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

    pub fn add_new_heap_item(&mut self, object: JsHeapObject) -> JsAddress {
        let new_address = get_next_js_heap_address();
        self.heap.insert(new_address, object);
        return new_address;
    }

    pub fn get_by_reference(&mut self, reference: JsReference) -> Option<&mut JsValue> {
        match reference {
            JsReference::Variable(variable_name) => {
                for context in self.context_stack.iter_mut().rev() {
                    let value = context.get_mut(&variable_name);
                    if value.is_some() {
                        return value;
                    }
                }
                return None;
            },
            JsReference::Property { object_address: _, member: _ } => {
                todo!(); //TODO: implement
            },
            JsReference::Index { object_address: _, index: _ } => {
                todo!(); //TODO: implement
            },
        }
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
