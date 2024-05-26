use super::{
    js_execution_context::{JsExecutionContext, JsValue},
    js_lexer,
    js_parser
};


fn js_values_are_equal(one: &JsValue, two: &JsValue) -> bool {
    //we implement this method standalone, rather than via the PartialEq trait, since we use Rc for function objects.
    //TODO: we might still want this method implemented on the actual objects, but for function not with a derive, but an explicit impl

    match one {
        JsValue::Number(num_one) => {
            match two {
                JsValue::Number(num_two) => { return num_one == num_two },
                _ => { return false; }
            }
        },
        JsValue::String(str_one) => {
            match two {
                JsValue::String(str_two) => { return str_one == str_two },
                _ => { return false; }
            }
        },
        JsValue::Boolean(_) => todo!(),
        JsValue::Object(_) => todo!(),
        JsValue::Function(_) => todo!(),
        JsValue::Undefined => {
            match two {
                JsValue::Undefined => { return true },
                _ => { return false; }
            }
        },
        JsValue::Address(_) => todo!(),
    }
}



#[test]
fn test_basic_assignment_and_export() {
    let code = "x = 3; tester.export(x + 4);";

    let tokens = js_lexer::lex_js(code, 1, 1);
    let script = js_parser::parse_js(&tokens);
    let mut js_execution_context = JsExecutionContext::new();
    script.execute(&mut js_execution_context);

    assert!(js_values_are_equal(&js_execution_context.get_last_exported_test_data(), &JsValue::Number(7)));
}


#[test]
fn test_binop_associativity() {
    let code = "x = 12 / 3 * 2; tester.export(x);";

    let tokens = js_lexer::lex_js(code, 1, 1);
    let script = js_parser::parse_js(&tokens);
    let mut js_execution_context = JsExecutionContext::new();
    script.execute(&mut js_execution_context);

    assert!(js_values_are_equal(&js_execution_context.get_last_exported_test_data(), &JsValue::Number(8)));
}
