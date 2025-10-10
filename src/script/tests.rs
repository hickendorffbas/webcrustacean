use std::collections::HashMap;

use super::js_execution_context::{JsObject, JsValue};
use super::js_interpreter::JsInterpreter;
use super::js_lexer;
use super::js_parser;


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
        JsValue::Object(obj_one) => {
            match two {
                JsValue::Object(obj_two) => { return obj_one.members == obj_two.members; }
                _ => { return false; }
            }
        },
        JsValue::Array(_) => todo!(),
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

    let mut interpreter = JsInterpreter::new();
    interpreter.run_script(&script);

    assert!(js_values_are_equal(&interpreter.get_last_exported_test_data(), &JsValue::Number(7)));
}


#[test]
fn test_binop_associativity() {
    let code = "x = 12 / 3 * 2; tester.export(x);";

    let tokens = js_lexer::lex_js(code, 1, 1);
    let script = js_parser::parse_js(&tokens);
    let mut interpreter = JsInterpreter::new();
    interpreter.run_script(&script);

    assert!(js_values_are_equal(&interpreter.get_last_exported_test_data(), &JsValue::Number(8)));
}


#[test]
fn test_literal_object_notation() {
    let code = r#"x = {"a": 4, "b": 2};
                  x.a = x.a + 1;
                  x.c = 5;
                  tester.export(x.a + x.b + x.c);"#;

    let tokens = js_lexer::lex_js(code, 1, 1);
    let script = js_parser::parse_js(&tokens);
    let mut interpreter = JsInterpreter::new();
    interpreter.run_script(&script);

    assert!(js_values_are_equal(&interpreter.get_last_exported_test_data(), &JsValue::Number(12)));
}


#[test]
fn test_basic_function_call() {
    let code = r#"function mult(p1, p2) {
            return p1 * p2;
        };

        x = mult(2, 3);
        tester.export(x);"#;

    let tokens = js_lexer::lex_js(code, 1, 1);
    let script = js_parser::parse_js(&tokens);
    let mut interpreter = JsInterpreter::new();
    interpreter.run_script(&script);

    assert!(js_values_are_equal(&interpreter.get_last_exported_test_data(), &JsValue::Number(6)));
}


#[test]
fn test_basic_function_call_no_args() {
    let code = r#"function get() {
            return 150;
        };
        x = get();
        tester.export(x);"#;

    let tokens = js_lexer::lex_js(code, 1, 1);
    let script = js_parser::parse_js(&tokens);
    let mut interpreter = JsInterpreter::new();
    interpreter.run_script(&script);

    assert!(js_values_are_equal(&interpreter.get_last_exported_test_data(), &JsValue::Number(150)));
}


#[test]
fn test_string_with_escape() {
    let code = r#"
        x = "test \" test";
        tester.export(x);"#;

    let tokens = js_lexer::lex_js(code, 1, 1);
    let script = js_parser::parse_js(&tokens);
    let mut interpreter = JsInterpreter::new();
    interpreter.run_script(&script);

    assert!(js_values_are_equal(&interpreter.get_last_exported_test_data(), &JsValue::String(String::from("test \" test"))));
}


#[test]
fn test_not_parsing_comments() {
    let code = r#"
        x = 1;
        // x = 2;
        /* x = 3;
            this is extra text */
        tester.export(x);"#;

    let tokens = js_lexer::lex_js(code, 1, 1);
    let script = js_parser::parse_js(&tokens);
    let mut interpreter = JsInterpreter::new();
    interpreter.run_script(&script);

    assert!(js_values_are_equal(&interpreter.get_last_exported_test_data(), &JsValue::Number(1)));
}


#[test]
fn test_double_slash_in_string_is_not_a_comment() {
    let code = r#"x = "https://www.reddit.com"; tester.export(x);"#;

    let tokens = js_lexer::lex_js(code, 1, 1);
    let script = js_parser::parse_js(&tokens);
    let mut interpreter = JsInterpreter::new();
    interpreter.run_script(&script);

    assert!(js_values_are_equal(&interpreter.get_last_exported_test_data(), &JsValue::String("https://www.reddit.com".to_owned())));
}


#[test]
fn test_escaping_the_escape_char() {
    let code = r#"
        x = "\\";
        y = "\\";
        tester.export(y); "#;

    let tokens = js_lexer::lex_js(code, 1, 1);
    let script = js_parser::parse_js(&tokens);
    let mut interpreter = JsInterpreter::new();
    interpreter.run_script(&script);

    assert!(js_values_are_equal(&interpreter.get_last_exported_test_data(), &JsValue::String(String::from("\\"))));
}


#[test]
fn test_create_empty_object() {
    let code = r#" x1 = {};
        tester.export(x1); "#;

    let tokens = js_lexer::lex_js(code, 1, 1);
    let script = js_parser::parse_js(&tokens);
    let mut interpreter = JsInterpreter::new();
    interpreter.run_script(&script);

    assert!(js_values_are_equal(&interpreter.get_last_exported_test_data(), &JsValue::Object(JsObject {members: HashMap::new()})));
}


#[test]
fn test_empty_statement_in_front() {
    let code = r#"; var x=1;
        tester.export(x);"#;

    let tokens = js_lexer::lex_js(code, 1, 1);
    let script = js_parser::parse_js(&tokens);
    let mut interpreter = JsInterpreter::new();
    interpreter.run_script(&script);

    assert!(js_values_are_equal(&interpreter.get_last_exported_test_data(), &JsValue::Number(1)));
}


#[test]
fn test_basic_if_statement() {
    let code = r#" f = 1; b = 0;
        if (f == 1) {
            b = b + 1;
        }
        if (f == 2) {
            b = b + 4;
        } else {
            b = b + 7;
        }
        tester.export(b); "#;

    let tokens = js_lexer::lex_js(code, 1, 1);
    let script = js_parser::parse_js(&tokens);

    let mut interpreter = JsInterpreter::new();
    interpreter.run_script(&script);

    assert!(js_values_are_equal(&interpreter.get_last_exported_test_data(), &JsValue::Number(8)));
}


#[test]
fn test_negative_number() {
    let code = r#"var x = -3;
        x = x + 5;
        tester.export(x);"#;

    let tokens = js_lexer::lex_js(code, 1, 1);
    let script = js_parser::parse_js(&tokens);
    let mut interpreter = JsInterpreter::new();
    interpreter.run_script(&script);

    assert!(js_values_are_equal(&interpreter.get_last_exported_test_data(), &JsValue::Number(2)));
}


#[test]
fn test_index_operator_for_object_properties() {
    let code = r#"var x = { "item": "value", "other": 3};
        tester.export(x["item"]);"#;

    let tokens = js_lexer::lex_js(code, 1, 1);
    let script = js_parser::parse_js(&tokens);
    let mut interpreter = JsInterpreter::new();
    interpreter.run_script(&script);

    assert!(js_values_are_equal(&interpreter.get_last_exported_test_data(), &JsValue::String(String::from("value"))));
}


#[test]
fn test_array() {
    let code = r#"var x = [1, 2];
        tester.export(x[1]);"#;

    let tokens = js_lexer::lex_js(code, 1, 1);
    let script = js_parser::parse_js(&tokens);
    let mut interpreter = JsInterpreter::new();
    interpreter.run_script(&script);

    assert!(js_values_are_equal(&interpreter.get_last_exported_test_data(), &JsValue::Number(2)));
}


#[test]
fn test_new_object_with_newlines() {
    let code = r#"var data = {
        a: 1,
        b: 2
    }; tester.export(data.b);"#;

    let tokens = js_lexer::lex_js(code, 1, 1);
    let script = js_parser::parse_js(&tokens);
    let mut interpreter = JsInterpreter::new();
    interpreter.run_script(&script);

    assert!(js_values_are_equal(&interpreter.get_last_exported_test_data(), &JsValue::Number(2)));
}



#[test]
fn test_anonymous_function() {
    let code = r#"(function (w) { tester.export(w); })(13);"#;

    let tokens = js_lexer::lex_js(code, 1, 1);
    let script = js_parser::parse_js(&tokens);
    let mut interpreter = JsInterpreter::new();
    interpreter.run_script(&script);

    assert!(js_values_are_equal(&interpreter.get_last_exported_test_data(), &JsValue::Number(13)));
}
