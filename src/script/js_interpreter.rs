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

    pub fn get_var_address(&self, name: &String) -> Option<&JsAddress> {
        //TODO: for now we check just the last stack frame, but we need to walk them up until we find it...

        let current_context = self.context_stack.iter().last().unwrap();
        return current_context.get_var_address(name);
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
