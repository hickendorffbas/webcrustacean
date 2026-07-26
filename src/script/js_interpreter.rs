use super::js_ast::Script;
use super::js_execution_context::{
    JsAddress,
    JsError,
    JsExecutionContext,
    JsValue,
};


pub struct JsInterpreter {
    pub context_stack: Vec<JsExecutionContext>,
    current_error: Option<JsError>,
    pub return_value: Option<JsValue>,
    #[cfg(test)] pub last_test_data: Option<JsValue>,
}

impl JsInterpreter {
    pub fn new() -> JsInterpreter {
        return JsInterpreter {
            context_stack: Vec::new(),
            current_error: None,
            return_value: None,
            #[cfg(test)] last_test_data: None,
        };
    }

    pub fn register_return_value(&mut self, return_value: JsValue) {
        self.return_value = Some(return_value);
    }

    pub fn set_error(&mut self, error: JsError) {
        self.current_error = Some(error);
    }

    pub fn run_script(&mut self, script: &Script) {
        debug_assert!(self.context_stack.len() == 0);

        let global_context = JsExecutionContext::new();
        self.context_stack.push(global_context);

        self.run_script_with_context_stack(script);

        self.context_stack.clear();
    }

    pub fn run_script_with_context_stack(&mut self, script: &Script) {
        for statement in script {
            let run_next_statement = statement.execute(self);

            if !run_next_statement {
                if self.context_stack.len() == 0 {
                    todo!() //TODO: report some error, there is nothing to return to...
                } else {
                    return;
                }
            }

        }
    }

    pub fn get_var_address(&self, name: &String) -> Option<JsAddress> {
        for context in self.context_stack.iter().rev() {
            match context.get_var_address(name) {
                Some(address) => return Some(*address),
                None => continue,
            }
        }
        return None;
    }

    pub fn deref(&mut self, value: &JsValue) -> JsValue {
        match value {
            JsValue::Address(address) => {

                for context in self.context_stack.iter_mut().rev() {
                    let adress_in_current_context = context.get_value(&address);
                    match adress_in_current_context {
                        Some(value) => return value.clone(),
                        None => continue,
                    }
                }

                todo!(); //TODO: check if this is allowed to fail
            },
            _ => { return value.clone() }
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
