use crate::network::url::Url;

use super::js_interpreter::JsInterpreter;
use super::js_lexer;
use super::js_parser;
use super::js_values::JsValue;


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
        JsValue::Boolean(value_one) => {
            match two {
                JsValue::Boolean(value_two) => { return value_one == value_two },
                _ => { return false; }
            }
        },
        JsValue::Undefined => {
            match two {
                JsValue::Undefined => { return true },
                _ => { return false; }
            }
        },
        JsValue::Address(_) => todo!(),
    }
}


fn assert_js(code: &str, expected: JsValue) {
    let tokens = js_lexer::lex_js(code, 1, 1);
    let script = js_parser::parse_js(&tokens, &Url::empty());

    let mut interpreter = JsInterpreter::new();
    interpreter.run_script(&script, Vec::new());

    assert!(js_values_are_equal(&interpreter.get_last_exported_test_data(), &expected));
}

#[test]
fn test_basic_assignment_and_export() {
    assert_js("x = 3; tester.export(x + 4);", JsValue::Number(7));
}

#[test]
fn test_binop_associativity() {
    assert_js("x = 12 / 3 * 2; tester.export(x);", JsValue::Number(8));
}

#[test]
fn test_literal_object_notation() {
    assert_js(r#"
x = {"a": 4, "b": 2};
x.a = x.a + 1;
x.c = 5;
tester.export(x.a + x.b + x.c);"#,
    JsValue::Number(12));
}

#[test]
fn test_literal_object_notation_trailing_comma() {
    assert_js(r#"
x = {"a": 4, "b": 2, };
x.a = x.a + 1;
x.c = 5;
tester.export(x.a + x.b + x.c);"#,
    JsValue::Number(12));
}

#[test]
fn test_basic_function_call() {
    assert_js(r#"
function mult(p1, p2) {
    return p1 * p2;
};

x = mult(2, 3);
tester.export(x);"#,
    JsValue::Number(6));
}

#[test]
fn test_basic_function_call_no_args() {
    assert_js(r#"
function get() {
    return 150;
};
x = get();
tester.export(x);"#,
    JsValue::Number(150));
}

#[test]
fn test_string_with_escape() {
    assert_js(r#"
x = "test \" test";
tester.export(x);"#,
    JsValue::String(String::from("test \" test")));
}

#[test]
fn test_not_parsing_comments() {
    assert_js(r#"
x = 1;
// x = 2;
/* x = 3;
    this is extra text */
tester.export(x);"#,
    JsValue::Number(1));
}

#[test]
fn test_double_slash_in_string_is_not_a_comment() {
    assert_js(r#"x = "https://www.reddit.com"; tester.export(x);"#, JsValue::String("https://www.reddit.com".to_owned()));
}

#[test]
fn test_escaping_the_escape_char() {
    assert_js(r#"
        x = "\\";
        y = "\\";
        tester.export(y); "#,
    JsValue::String(String::from("\\")));
}

// TODO: fix by getting length of object and asserting that
// #[test]
// fn test_create_empty_object() {
//     assert_js(r#" x1 = {}; tester.export(x1); "#, JsValue::Object(JsObject {members: HashMap::new(), callable: None}));
// }

#[test]
fn test_empty_statement_in_front() {
    assert_js(r#"; var x=1;
        tester.export(x);"#,
    JsValue::Number(1));
}

#[test]
fn test_basic_if_statement() {
    assert_js(r#" f = 1; b = 0;
        if (f == 1) {
            b = b + 1;
        }
        if (f == 2) {
            b = b + 4;
        } else {
            b = b + 7;
        }
        tester.export(b); "#,
    JsValue::Number(8));
}

#[test]
fn test_negative_number() {
    assert_js(r#"var x = -3;
        x = x + 5;
        tester.export(x);"#,
    JsValue::Number(2));
}

#[test]
fn test_index_operator_for_object_properties() {
    assert_js(r#"var x = { "item": "value", "other": 3}; tester.export(x["item"]);"#,
    JsValue::String(String::from("value")));
}

#[test]
fn test_array() {
    assert_js(r#"var x = [1, 2]; tester.export(x[1]);"#, JsValue::Number(2));
}

#[test]
fn test_muti_dimensional_array() {
    assert_js(r#"var x = [[1, 5], [2, 3]]; tester.export(x[1][1]);"#, JsValue::Number(3));
}

#[test]
fn test_new_object_with_newlines() {
    assert_js(r#"var data = {
            a: 1,
            b: 2
        }; tester.export(data.b);"#,
    JsValue::Number(2));
}

#[test]
fn test_anonymous_function() {
    assert_js(r#"(function (w) { tester.export(w); })(13);"#, JsValue::Number(13));
}

#[test]
fn test_comma_operator() {
     assert_js(r#"var x = (1, 2, 5); tester.export(x);"#, JsValue::Number(5));
}

#[test]
fn test_ternary_true() {
    assert_js(r#"var n = 4; tester.export(n == 4 ? 1 : 2);"#, JsValue::Number(1));
}

#[test]
fn test_ternary_false() {
    assert_js(r#"var n = 4; tester.export(n == 3 ? 1 : 2);"#, JsValue::Number(2));
}

#[test]
fn test_ternary() {
    assert_js(r#"var n = 4; tester.export(n == 3 ? 1 : 2);"#, JsValue::Number(2));
}

#[test]
fn test_while() {
    assert_js(r#"
a = 1; b = 1;
while (a == 1) {
    tester.export(b);
    if (b == 5) {
        a = 2;
    };
    b = b + 1;
}"#,
    JsValue::Number(5));
}

#[test]
fn test_not_operator() {
    assert_js(r#"var n = 4; tester.export(!(n == 3));"#, JsValue::Boolean(true));
}

#[test]
fn test_literal_bool() {
    assert_js(r#"var n = false;
        if (!n) {
            tester.export("A");
        } else {
            tester.export("B");
        }"#,
    JsValue::String("A".to_owned()));
}

#[test]
fn test_empty_anonymous_function_expression() {
    let code = r#"var log = function() {};"#;

    let tokens = js_lexer::lex_js(code, 1, 1);
    let script = js_parser::parse_js(&tokens, &Url::empty());
    let mut interpreter = JsInterpreter::new();
    interpreter.run_script(&script, Vec::new());

    //Note: no assert, this just checks for not crashing
    //TODO: when we have enough of js implemented, this could assert some property of the created anonymous function
}

#[test]
fn test_functions_are_also_just_objects() {
    assert_js(r#"
var func = function() {};
func.x = 3;
tester.export(func.x);"#,
    JsValue::Number(3));
}

#[test]
fn test_bitshift() {
    assert_js(r#"
a = 5; // 000000000000000101
b = 2;
tester.export(a << b);"#,
    JsValue::Number(20));
}

#[test]
fn test_hexadecimal() {
    assert_js(r#"
let hexNumber = 0x1A - 3;
tester.export(hexNumber);"#,
    JsValue::Number(23));
}

#[test]
fn test_compound_assign_add() {
    assert_js(r#"
let x = 6;
x += 4;
tester.export(x);"#,
    JsValue::Number(10));
}

#[test]
fn test_automatically_new_statement_after_if_content_block() {
    assert_js(r#"
var x = 3;
if (x > 4) {
    x += 1;
} (function() {
    tester.export(x);
}());"#,
    JsValue::Number(3));
}

#[test]
fn multiple_var_decl_with_newline() {
    assert_js(r#"
var x = 1,
    y = 2;
tester.export(x + y);"#,
    JsValue::Number(3));
}

#[test]
fn for_loop_adding_variable() {
    assert_js(r#"
var nn = 1;
for (var i = 0; i < 4; i++) {
    nn = nn + i;
}
tester.export(nn);"#,
    JsValue::Number(7));
}

#[test]
fn arrays_with_computed_index_as_lvalue() {
    assert_js(r#"var x = [1,2,3];
x[1 + 1] = 4;
tester.export(x[2]);"#,
    JsValue::Number(4));
}

#[test]
fn allow_empty_else_block_with_newlines_in_it() {
    assert_js(r#"var x = true;
if (x) {
    tester.export(1);
} else {

}"#,
    JsValue::Number(1));
}

#[test]
fn strict_equals() {
    assert_js(r#"
const num = 0;
const str = "0";
var x = 0;

if (num === num) { x += 1; }
if (str === str) { x += 2; }
if (num === str) { x += 4; }
tester.export(x);"#,
    JsValue::Number(3));
}

#[test]
fn basic_in_operator() {
    assert_js(r#"var obj = {"a": 3, "b": 2}; tester.export("a" in obj);"#, JsValue::Boolean(true));
}

#[test]
fn for_in() {
    assert_js(r#"var x = {"a": 1, "b": 2, "c": 3}; var y = 12;
for (var n in x) {
    y += x[n];
}
tester.export(y);"#,
    JsValue::Number(18));
}

#[test]
fn add_strings() {
    assert_js(r#"a = "Hello "; b = "world"; tester.export(a + b);"#, JsValue::String("Hello world".to_owned()));
}

#[test]
fn test_modulus() {
    assert_js(r#"tester.export(15 % 7);"#, JsValue::Number(1));
}

#[test]
fn reference_assign() {
    assert_js(r#"
x = {"a": 3};
x.a = x.a + 2;
tester.export(x.a);
"#,
    JsValue::Number(5));
}

#[test]
fn greater_or_equal() {
    assert_js(r#"a = 0; x = 4; if (x >= 3) { a++; } if (x >= 7) { a = a + 5; } tester.export(a); "#, JsValue::Number(1));
}

#[test]
fn else_if() {
    assert_js(r#"a = 5; if (a == 2) { tester.export(1); } else if (a == 5) { tester.export(2); } else { tester.export(3) }"#, JsValue::Number(2));
}
