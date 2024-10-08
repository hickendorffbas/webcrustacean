use std::cell::RefCell;
use std::rc::Rc;

use crate::dom::{Document, ElementDomNode};

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

    pub fn run_scripts_in_document(&mut self, document: &RefCell<Document>) {
        let mut all_scripts = Vec::new();
        self.collect_all_scripts_for_node(&document.borrow().document_node.borrow(), &mut all_scripts);

        for script in all_scripts {
            //TODO: we have collected the internal id of the node the script is on as well, check if we need that (for scripts that modify the dom)

            self.run_script(&script.1)
        }

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

    fn collect_all_scripts_for_node(&mut self, dom_node: &ElementDomNode, all_scripts: &mut Vec<(usize, Rc<Script>)>) {

        if dom_node.scripts.is_some() {
            for script in dom_node.scripts.as_ref().unwrap() {
                all_scripts.push( (dom_node.internal_id, script.clone()) )
            }
        }

        if dom_node.children.is_some() {
            for child in dom_node.children.as_ref().unwrap() {
                self.collect_all_scripts_for_node(&child.borrow(), all_scripts);
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
