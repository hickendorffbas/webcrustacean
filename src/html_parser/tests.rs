use crate::{html_parser::Parser, jsonify};


#[test]
fn test_basic_parsing_1() {
    let code = r#"<b>test</b>"#;

    let mut parser = Parser::new(code.to_owned());
    parser.parse();

    let mut json = String::new();
    jsonify::dom_node_to_json(&parser.document.document_node, &mut json);

    //TODO: instead white a good set of util functions doing asserts on the returned structure
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
        ]
    }
    "#.to_string();

    assert!(jsonify::json_is_equal(&json, &expected_json));
}


#[test]
fn test_text_concatenation() {
    let code = r#"<div>two words</div>"#;

    let mut parser = Parser::new(code.to_owned());
    parser.parse();

    let mut json = String::new();
    jsonify::dom_node_to_json(&parser.document.document_node, &mut json);

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
        ]
    }
    "#.to_string();

    assert!(jsonify::json_is_equal(&json, &expected_json));
}


#[test]
fn test_handling_whitespace() {
    let code = r#"     <b>test       </b >        "#;

    let mut parser = Parser::new(code.to_owned());
    parser.parse();

    let mut json = String::new();
    jsonify::dom_node_to_json(&parser.document.document_node, &mut json);

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
                        "name":"b",
                        "text":"",
                        "image":false,
                        "scripts":0,
                        "component":false,
                        "attributes:":[],
                        "children":[
                            {
                            "name":"",
                            "text":"test       ",
                            "image":false,
                            "scripts":0,
                            "component":false,
                            "attributes:":[],
                            "children":[]
                            }
                        ]
                    },
                    {
                        "name":"",
                        "text":"        ",
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

    println!("{}", json);

    assert!(jsonify::json_is_equal(&json, &expected_json));
}


#[test]
fn test_basic_parsing_attributes() {
    let code = r#"<div color="red">test</div>"#;

    let mut parser = Parser::new(code.to_owned());
    parser.parse();

    let mut json = String::new();
    jsonify::dom_node_to_json(&parser.document.document_node, &mut json);

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
                        "name":"div",
                        "text":"",
                        "image":false,
                        "scripts":0,
                        "component":false,
                        "attributes:":[
                            {
                                "name": "color",
                                "value": "red"
                            }
                        ],
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
        ]
    }
    "#.to_string();

    assert!(jsonify::json_is_equal(&json, &expected_json));
}
