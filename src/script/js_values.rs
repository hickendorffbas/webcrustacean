use std::collections::HashMap;
use std::rc::Rc;

use super::js_ast::Script;


pub type JsAddress = usize;


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
    Number(f64),
    String(String),
    Boolean(bool),
    Address(JsAddress),
    Undefined,
}
impl JsValue {
    pub fn is_thruty(self) -> bool {
        match self {
            JsValue::Number(number) => { return number != 0.0 },
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
    TypeError,
}
