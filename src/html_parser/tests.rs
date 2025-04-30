use crate::html_lexer;
use crate::html_parser;
use crate::jsonify;
use crate::network::url::Url;


const DEFAULT_URL: &str = "http://www.google.com";


#[test]
fn test_basic_parsing_1() {
    let code = r#"<b>test</b>"#;

    let tokens = html_lexer::lex_html(code);
    let main_url = Url::from(&DEFAULT_URL.to_string());
    let document = html_parser::parse(tokens, &main_url);

    let mut json = String::new();
    jsonify::dom_node_to_json(&document.document_node, &mut json);

    let expected_json = r#"
    {
        "name":"",
        "text":"",
        "image":false,
        "scripts":0,
        "component":false,
        "attributes:":[],
        "children":[
            {
                "name":"b",
                "text":"",
                "image":false,
                "scripts":0,
                "component":false,
                "attributes:":[],
                "children":[
                    {
                    "name":"",
                    "text":"test",
                    "image":false,
                    "scripts":0,
                    "component":false,
                    "attributes:":[],
                    "children":[]
                    }
                ]
            }
        ]
    }
    "#.to_string();

    assert!(jsonify::json_is_equal(&json, &expected_json));
}


#[test]
fn test_text_concatenation() {
    let code = r#"<div>two words</div>"#;

    let tokens = html_lexer::lex_html(code);
    let main_url = Url::from(&DEFAULT_URL.to_string());
    let document = html_parser::parse(tokens, &main_url);

    let mut json = String::new();
    jsonify::dom_node_to_json(&document.document_node, &mut json);

    let expected_json = r#"
    {
        "name":"",
        "text":"",
        "image":false,
        "scripts":0,
        "component":false,
        "attributes:":[],
        "children":[
            {
                "name":"div",
                "text":"",
                "image":false,
                "scripts":0,
                "component":false,
                "attributes:":[],
                "children":[
                    {
                    "name":"",
                    "text":"two words",
                    "image":false,
                    "scripts":0,
                    "component":false,
                    "attributes:":[],
                    "children":[]
                    }
                ]
            }
        ]
    }
    "#.to_string();

    assert!(jsonify::json_is_equal(&json, &expected_json));
}


#[test]
fn test_not_closing_a_tag() {
    let code = r#"<div><b><p></p></div>"#; //the parser should close b after </p>

    let tokens = html_lexer::lex_html(code);
    let main_url = Url::from(&DEFAULT_URL.to_string());
    let document = html_parser::parse(tokens, &main_url);

    let mut json = String::new();
    jsonify::dom_node_to_json(&document.document_node, &mut json);

    let expected_json = r#"
    {
        "name":"",
        "text":"",
        "image":false,
        "scripts":0,
        "component":false,
        "attributes:":[],
        "children":[
            {
                "name":"div",
                "text":"",
                "image":false,
                "scripts":0,
                "component":false,
                "attributes:":[],
                "children":[
                    {
                    "name":"b",
                    "text":"",
                    "image":false,
                    "scripts":0,
                    "component":false,
                    "attributes:":[],
                    "children":[
                        {
                        "name":"p",
                        "text":"",
                        "image":false,
                        "scripts":0,
                        "component":false,
                        "attributes:":[],
                        "children":[]
                        }
                    ]
                    }
                ]
            }
        ]
    }
    "#.to_string();

    assert!(jsonify::json_is_equal(&json, &expected_json));
}


#[test]
fn test_closing_a_tag_we_did_not_open() {
    let code = r#"<div><b></p></b></div>"#; //the </p> should be ignored

    let tokens = html_lexer::lex_html(code);
    let main_url = Url::from(&DEFAULT_URL.to_string());
    let document = html_parser::parse(tokens, &main_url);

    let mut json = String::new();
    jsonify::dom_node_to_json(&document.document_node, &mut json);


    let expected_json = r#"
    {
        "name":"",
        "text":"",
        "image":false,
        "scripts":0,
        "component":false,
        "attributes:":[],
        "children":[
            {
                "name":"div",
                "text":"",
                "image":false,
                "scripts":0,
                "component":false,
                "attributes:":[],
                "children":[
                    {
                    "name":"b",
                    "text":"",
                    "image":false,
                    "scripts":0,
                    "component":false,
                    "attributes:":[],
                    "children":[]
                    }
                ]
            }
        ]
    }
    "#.to_string();

    assert!(jsonify::json_is_equal(&json, &expected_json));
}


#[test]
fn test_missing_last_closing_tag() {
    let code = r#"<html><body></body>"#; //the </html> should be added at the end

    let tokens = html_lexer::lex_html(code);
    let main_url = Url::from(&DEFAULT_URL.to_string());
    let document = html_parser::parse(tokens, &main_url);

    let mut json = String::new();
    jsonify::dom_node_to_json(&document.document_node, &mut json);

    let expected_json = r#"
    {
        "name":"",
        "text":"",
        "image":false,
        "scripts":0,
        "component":false,
        "attributes:":[],
        "children":[
            {
                "name":"html",
                "text":"",
                "image":false,
                "scripts":0,
                "component":false,
                "attributes:":[],
                "children":[
                    {
                    "name":"body",
                    "text":"",
                    "image":false,
                    "scripts":0,
                    "component":false,
                    "attributes:":[],
                    "children":[]
                    }
                ]
            }
        ]
    }
    "#.to_string();

    assert!(jsonify::json_is_equal(&json, &expected_json));
}
