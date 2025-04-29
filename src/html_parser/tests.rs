use crate::dom::ElementDomNode;
use crate::html_lexer;
use crate::html_parser;
use crate::jsonify;
use crate::network::url::Url;
use crate::test_util::*;


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

    let tokens = vec![
        html_open("div"),
        html_open_tag_end(),
        html_text("two"),
        html_whitespace(" "),
        html_text("words"),
        html_close("div"),
    ];

    let main_url = Url::from(&DEFAULT_URL.to_string());
    let parse_result = html_parser::parse(tokens, &main_url);
    let document = parse_result;
    let doc_node = &document.document_node.borrow();
    assert_eq!(doc_node.children.as_ref().unwrap().len(), 1);

    let div_node = doc_node.children.as_ref().unwrap()[0].borrow();
    let text_node = div_node.children.as_ref().unwrap()[0].borrow();
    assert_text_on_node_is(&text_node, "two words");
}


#[test]
fn test_not_closing_a_tag() {

    let tokens = vec![
        html_open("div"),
        html_open_tag_end(),

        html_open("b"),
        html_open_tag_end(),

        html_open("p"),
        html_open_tag_end(),

        html_close("p"),

        //the parser should close b here

        html_close("div"),
    ];

    let main_url = Url::from(&DEFAULT_URL.to_string());
    let parse_result = html_parser::parse(tokens, &main_url);
    let document = parse_result;
    let doc_node = &document.document_node.borrow();
    assert_eq!(doc_node.children.as_ref().unwrap().len(), 1);

    //TODO: it would be much nicer if we can just compare with a tree of nodes here, that we layout like in json, or just with tabs

    assert_element_name_is(&doc_node.children.as_ref().unwrap()[0].borrow(), "div");

    let div_node = doc_node.children.as_ref().unwrap()[0].borrow();
    let div_childs = div_node.children.as_ref().unwrap();
    assert_eq!(div_childs.len(), 1);
    assert_element_name_is(&div_childs[0].borrow(), "b");

    let b_node = div_childs[0].borrow();
    let b_childs = b_node.children.as_ref().unwrap();
    assert_eq!(b_childs.len(), 1);
    assert_element_name_is(&b_childs[0].borrow(), "p");
}


#[test]
fn test_closing_a_tag_we_did_not_open() {

    let tokens = vec![
        html_open("div"),
        html_open_tag_end(),

        html_open("b"),
        html_open_tag_end(),

        html_close("p"), //this one should be ignored

        html_close("b"),

        html_close("div"),
    ];

    let main_url = Url::from(&DEFAULT_URL.to_string());
    let parse_result = html_parser::parse(tokens, &main_url);
    let document = parse_result;
    let doc_node = &document.document_node.borrow();
    assert_eq!(doc_node.children.as_ref().unwrap().len(), 1);

    assert_element_name_is(&doc_node.children.as_ref().unwrap()[0].borrow(), "div");

    let div_node = doc_node.children.as_ref().unwrap()[0].borrow();
    let div_childs = div_node.children.as_ref().unwrap();
    assert_eq!(div_childs.len(), 1);
    assert_element_name_is(&div_childs[0].borrow(), "b");
}


#[test]
fn test_missing_last_closing_tag() {

    let tokens = vec![
        html_open("html"),
        html_open_tag_end(),

        html_open("body"),
        html_open_tag_end(),

        html_close("body"), 
    ];


    let main_url = Url::from(&DEFAULT_URL.to_string());
    let parse_result = html_parser::parse(tokens, &main_url);
    let document = parse_result;
    let doc_node = &document.document_node.borrow();
    assert_eq!(doc_node.children.as_ref().unwrap().len(), 1);

    assert_element_name_is(&doc_node.children.as_ref().unwrap()[0].borrow(), "html");

    let html_node = doc_node.children.as_ref().unwrap()[0].borrow();
    let html_childs = html_node.children.as_ref().unwrap();
    assert_eq!(html_childs.len(), 1);
    assert_element_name_is(&html_childs[0].borrow(), "body");

    let body_node = html_childs[0].borrow();
    let body_childs = body_node.children.as_ref().unwrap();
    assert_eq!(body_childs.len(), 0);
}


fn assert_element_name_is(node: &ElementDomNode, name: &str) {
    assert!(node.name.is_some());
    assert_eq!(node.name.as_ref().unwrap(), name);
}


fn assert_text_on_node_is(node: &ElementDomNode, text: &str) {
    assert!(node.text.is_some());
    assert_eq!(node.text.as_ref().unwrap().text_content, text);
}
