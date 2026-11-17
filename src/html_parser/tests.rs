use crate::html_parser::lexer::{Lexer, Token};
use crate::html_parser::{HtmlParser, ParserState};
use crate::jsonify;
use crate::network::url::Url;


fn run_parser(parser: &mut HtmlParser) {

    while !parser.is_done() {
        parser.step();

        match parser.state {
            ParserState::WaitingToStart => {},
            ParserState::ContinueParsing => {},
            ParserState::Done => break,
            _ => {
                panic!("These tests don't support script");
            }
        }
    }
}


fn lex_all(code: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(code.to_owned());
    let mut tokens = Vec::new();
    loop {
        let token = lexer.next_token().token;
        if token == Token::EOF {
            return tokens;
        }
        tokens.push(token);
    }
}


#[test]
fn test_basic_parsing_1() {
    let code = r#"<b>test</b>"#;

    let mut parser = HtmlParser::new(Url::empty());
    parser.start(code.to_owned(), Url::empty());
    run_parser(&mut parser);

    let mut json = String::new();
    jsonify::dom_node_to_json(&parser.document.document_node, &mut json);

    //TODO: instead white a good set of util functions doing asserts on the returned structure
    let expected_json = r#"
    {
        "name":"",
        "text":"",
        "image":false,
        "component":false,
        "attributes:":[],
        "children":[
            {
                "name":"html",
                "text":"",
                "image":false,
                "component":false,
                "attributes:":[],
                "children":[
                    {
                        "name":"b",
                        "text":"",
                        "image":false,
                        "component":false,
                        "attributes:":[],
                        "children":[
                            {
                            "name":"",
                            "text":"test",
                            "image":false,
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

    let mut parser = HtmlParser::new(Url::empty());
    parser.start(code.to_owned(), Url::empty());
    run_parser(&mut parser);

    let mut json = String::new();
    jsonify::dom_node_to_json(&parser.document.document_node, &mut json);

    let expected_json = r#"
    {
        "name":"",
        "text":"",
        "image":false,
        "component":false,
        "attributes:":[],
        "children":[
            {
                "name":"html",
                "text":"",
                "image":false,
                "component":false,
                "attributes:":[],
                "children":[
                    {
                        "name":"div",
                        "text":"",
                        "image":false,
                        "component":false,
                        "attributes:":[],
                        "children":[
                            {
                            "name":"",
                            "text":"two words",
                            "image":false,
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

    let mut parser = HtmlParser::new(Url::empty());
    parser.start(code.to_owned(), Url::empty());
    run_parser(&mut parser);

    let mut json = String::new();
    jsonify::dom_node_to_json(&parser.document.document_node, &mut json);

    let expected_json = r#"
    {
        "name":"",
        "text":"",
        "image":false,
        "component":false,
        "attributes:":[],
        "children":[
            {
                "name":"html",
                "text":"",
                "image":false,
                "component":false,
                "attributes:":[],
                "children":[
                    {
                        "name":"b",
                        "text":"",
                        "image":false,
                        "component":false,
                        "attributes:":[],
                        "children":[
                            {
                            "name":"",
                            "text":"test       ",
                            "image":false,
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
fn test_basic_parsing_attributes() {
    let code = r#"<div color="red">test</div>"#;

    let mut parser = HtmlParser::new(Url::empty());
    parser.start(code.to_owned(), Url::empty());
    run_parser(&mut parser);

    let mut json = String::new();
    jsonify::dom_node_to_json(&parser.document.document_node, &mut json);

    let expected_json = r#"
    {
        "name":"",
        "text":"",
        "image":false,
        "component":false,
        "attributes:":[],
        "children":[
            {
                "name":"html",
                "text":"",
                "image":false,
                "component":false,
                "attributes:":[],
                "children":[
                    {
                        "name":"div",
                        "text":"",
                        "image":false,
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
fn test_simple_html_entity() {
    let code = r#"<div>this is a test with &lt;entities&gt;</div>"#;

    let mut parser = HtmlParser::new(Url::empty());
    parser.start(code.to_owned(), Url::empty());
    run_parser(&mut parser);

    let mut json = String::new();
    jsonify::dom_node_to_json(&parser.document.document_node, &mut json);

    let expected_json = r#"
    {
        "name":"",
        "text":"",
        "image":false,
        "component":false,
        "attributes:":[],
        "children":[
            {
                "name":"html",
                "text":"",
                "image":false,
                "component":false,
                "attributes:":[],
                "children":[
                    {
                        "name":"div",
                        "text":"",
                        "image":false,
                        "component":false,
                        "attributes:":[],
                        "children":[
                            {
                            "name":"",
                            "text":"this is a test with <entities>",
                            "image":false,
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
fn test_comments() {
    let code = r#"<div color="red">test</div><!-- this is a comment <b>node</b> -->"#;

    let mut parser = HtmlParser::new(Url::empty());
    parser.start(code.to_owned(), Url::empty());
    run_parser(&mut parser);

    let mut json = String::new();
    jsonify::dom_node_to_json(&parser.document.document_node, &mut json);

    let expected_json = r#"
    {
        "name":"",
        "text":"",
        "image":false,
        "component":false,
        "attributes:":[],
        "children":[
            {
                "name":"html",
                "text":"",
                "image":false,
                "component":false,
                "attributes:":[],
                "children":[
                    {
                        "name":"div",
                        "text":"",
                        "image":false,
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
fn test_page_with_html() {
    let code = r#"<html>Location:<br /></html>"#;

    let mut parser = HtmlParser::new(Url::empty());
    parser.start(code.to_owned(), Url::empty());
    run_parser(&mut parser);

    let mut json = String::new();
    jsonify::dom_node_to_json(&parser.document.document_node, &mut json);

    let expected_json = r#"
    {
        "name":"",
        "text":"",
        "image":false,
        "component":false,
        "attributes:":[],
        "children":[
            {
                "name":"html",
                "text":"",
                "image":false,
                "component":false,
                "attributes:":[],
                "children":[
                    {
                        "name":"",
                        "text":"Location:",
                        "image":false,
                        "component":false,
                        "attributes:":[],
                        "children":[]
                    },
                    {
                        "name":"br",
                        "text":"",
                        "image":false,
                        "component":false,
                        "attributes:":[],
                        "children": []
                    }
                ]

            }
        ]
    }
    "#.to_string();

    assert!(jsonify::json_is_equal(&json, &expected_json));
}


#[test]
fn test_empty_page() {
    let code = r#"<html></html>"#;

    let mut parser = HtmlParser::new(Url::empty());
    parser.start(code.to_owned(), Url::empty());
    run_parser(&mut parser);

    let mut json = String::new();
    jsonify::dom_node_to_json(&parser.document.document_node, &mut json);

    let expected_json = r#"
    {
        "name":"",
        "text":"",
        "image":false,
        "component":false,
        "attributes:":[],
        "children":[
            {
                "name":"html",
                "text":"",
                "image":false,
                "component":false,
                "attributes:":[],
                "children":[]
            }
        ]
    }
    "#.to_string();

    assert!(jsonify::json_is_equal(&json, &expected_json));
}


#[test]
fn test_unterminated_entity_in_attribute_value_is_kept_literally() {
    let tokens = lex_all(r#"<script src="api.js?a=b&render=explicit" async></script><input style="display: none;">"#);

    assert!(tokens.contains(&Token::Attribute { name: "src".to_owned(), value: "api.js?a=b&render=explicit".to_owned() }));
    assert!(tokens.contains(&Token::StartTag { name: "input".to_owned(), self_closing: true }));
    assert!(tokens.contains(&Token::Attribute { name: "style".to_owned(), value: "display: none;".to_owned() }));
}


#[test]
fn test_tag_and_attribute_names_are_lowercased() {
    let tokens = lex_all(r#"<DIV Class="Keep-Case">a<BR>b</Div><img SRC=X.png HIDDEN>"#);

    assert!(tokens.contains(&Token::StartTag { name: "div".to_owned(), self_closing: false }));
    assert!(tokens.contains(&Token::Attribute { name: "class".to_owned(), value: "Keep-Case".to_owned() }));
    assert!(tokens.contains(&Token::StartTag { name: "br".to_owned(), self_closing: true }));
    assert!(tokens.contains(&Token::EndTag { name: "div".to_owned() }));
    assert!(tokens.contains(&Token::Attribute { name: "src".to_owned(), value: "X.png".to_owned() }));
    assert!(tokens.contains(&Token::Attribute { name: "hidden".to_owned(), value: "hidden".to_owned() }));
}


//TODO: make the json tests use this way of testing instead
fn parent_id_attribute_of_node_with_id(parser: &HtmlParser, id: &str) -> String {
    let all_nodes = &parser.document.all_nodes;
    let node = all_nodes.values().find(|node| node.borrow().get_attribute_value("id").as_deref() == Some(id)).unwrap();
    let parent = all_nodes.get(&node.borrow().parent_id).unwrap();
    return parent.borrow().get_attribute_value("id").unwrap();
}


#[test]
fn test_new_table_cell_implicitly_closes_open_cell() {
    let code = r#"<table><tr id="row"><td id="first">first<td id="second">second</tr></table>"#;

    let mut parser = HtmlParser::new(Url::empty());
    parser.start(code.to_owned(), Url::empty());
    run_parser(&mut parser);

    assert_eq!(parent_id_attribute_of_node_with_id(&parser, "first"), "row");
    assert_eq!(parent_id_attribute_of_node_with_id(&parser, "second"), "row");
}
