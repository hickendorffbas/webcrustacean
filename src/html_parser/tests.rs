use crate::{html_parser::Parser, jsonify};


#[test]
fn test_basic_parsing_1() {
    let code = r#"<b>test</b>"#;

    let mut parser = Parser::new(code.to_owned());
    parser.parse();

    let mut json = String::new();
    jsonify::dom_node_to_json(&parser.document.document_node, &mut json);

    //TODO: can I find a better format to assert against?
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
