use std::collections::HashMap;
use std::rc::Rc;

use super::js_parser::Script;


pub struct JsExecutionContext {
    variables: HashMap<String, JsValue>,
}
impl JsExecutionContext {
    pub fn new() -> JsExecutionContext {

        let console_log_function = JsValue::Function(JsFunction {
            name: String::from("log"),
            code: None,
            builtin: Some(JsBuiltinFunction::ConsoleLog),
        });

        let console_builtin = JsValue::Object(JsObject {
            members: HashMap::from([(String::from("log"), console_log_function)])
        });

        return JsExecutionContext { variables: HashMap::from([(String::from("console"), console_builtin)]) };
    }
    pub fn set_var(&mut self, var_name: String, value: JsValue) {
        self.variables.insert(var_name, value);
    }
}

pub enum JsValue {
    Number(i32), //TODO: number type is wrong here, we need different rust types depending on what kind of number it is? (floats?)
                 //      or a more complex type maybe?
    String(String),
    #[allow(dead_code)] Boolean(bool), //TODO: use
    #[allow(dead_code)] Object(JsObject), //TODO: use
    Function(JsFunction),
}

pub struct JsObject {
    #[allow(dead_code)] members: HashMap<String, JsValue>, //TODO: use
}

pub struct JsFunction {
    #[allow(dead_code)] name: String, //TODO: use
    #[allow(dead_code)] code: Option<Rc<Script>>, //TODO: use
    #[allow(dead_code)] builtin: Option<JsBuiltinFunction>, //TODO: use
}

enum JsBuiltinFunction {
    ConsoleLog,
}
