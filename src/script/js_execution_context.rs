use std::collections::HashMap;
use std::rc::Rc;

use super::js_parser::Script;


pub struct JsExecutionContext {
    variables: HashMap<String, JsValue>,
    #[cfg(test)] pub last_test_data: Option<JsValue>,
}
impl JsExecutionContext {
    pub fn new() -> JsExecutionContext {
        let mut variables = HashMap::new();

        let console_log_function = JsValue::Function(JsFunction {
            name: String::from("log"),
            code: None,
            builtin: Some(JsBuiltinFunction::ConsoleLog),
        });

        let console_builtin = JsValue::Object(JsObject {
            members: HashMap::from([(String::from("log"), console_log_function)])
        });
        variables.insert(String::from("console"), console_builtin);


        #[cfg(test)] {
            let tester_export_function = JsValue::Function(JsFunction {
                name: String::from("export"),
                code: None,
                builtin: Some(JsBuiltinFunction::TesterExport),
            });

            let tester_builtin = JsValue::Object(JsObject {
                members: HashMap::from([(String::from("export"), tester_export_function)])
            });
            variables.insert(String::from("tester"), tester_builtin);
        }

        return JsExecutionContext {
            variables,
            #[cfg(test)] last_test_data: None,
        };
    }

    pub fn set_var(&mut self, var_name: String, value: JsValue) {
        self.variables.insert(var_name, value);
    }

    pub fn get_var(&self, var_name: &String) -> Option<&JsValue> {
        return self.variables.get(var_name);
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
    Variable(JsVariable),
    Undefined,
}
impl JsValue {
    pub fn deref(self, js_execution_context: &JsExecutionContext) -> JsValue {
        match self {
            JsValue::Variable(variable) => { return variable.get_value(js_execution_context); },
            _ => { return self }
        }
    }
}



#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub struct JsObject {
    pub members: HashMap<String, JsValue>,
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub struct JsVariable {
    pub name: String,
    pub object_var: Option<Rc<JsVariable>>,
}
impl JsVariable {
    pub fn get_value(&self, js_execution_context: &JsExecutionContext) -> JsValue {

        if self.object_var.is_none() {
            let value = js_execution_context.get_var(&self.name);
            if value.is_some() {
                return value.unwrap().clone();
            } else {
                //TODO: should this be an error?
                return JsValue::Undefined;
            }
        }

        let object = self.object_var.as_ref().unwrap().get_value(js_execution_context);

        match object {
            JsValue::Object(obj) => {
                let value = obj.members.get(&self.name);
                if value.is_some() {
                    return value.unwrap().clone();
                } else {
                    //TODO: should this be an error?
                    return JsValue::Undefined;
                }
            },
            _ => {
                todo!(); //TODO: proper error logging
            }
        }

    }
    pub fn set_value(&mut self, js_execution_context: &JsExecutionContext, value: JsValue) {
        todo!();
    }
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
