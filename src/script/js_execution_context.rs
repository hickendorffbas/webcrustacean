use std::collections::HashMap;


pub struct JsExecutionContext {
    variables: HashMap<String, JsValue>,
}
impl JsExecutionContext {
    pub fn new() -> JsExecutionContext {
        return JsExecutionContext { variables: HashMap::new() };
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
}

pub struct JsObject {
    #[allow(dead_code)] members: HashMap<String, JsValue>, //TODO: use
}
