
use crate::dom::ElementDomNode;
use crate::html_parser;
use crate::network::url::Url;
use crate::test_util::*;


#[test]
fn test_basic_parsing_1() {

    let tokens = vec![
        html_open("a"),
        html_open_tag_end(),
        html_text("text"),
        html_close("a"),
        html_whitespace(" "),
    ];

    let main_url = Url::from(&String::from("http://www.google.com")); //TODO: would be nice if we can define these as (lazy?) consts?
    let document = html_parser::parse(tokens, &main_url);
    assert_eq!(document.document_node.children.as_ref().unwrap().len(), 2);

    let generic_a_node = document.document_node.children.as_ref().unwrap()[0].as_ref();
    assert_element_name_is(generic_a_node, "a");

    let a_children = generic_a_node.children.as_ref().unwrap();
    assert_eq!(a_children.len(), 1);

    assert_text_on_node_is(a_children[0].as_ref(), "text");
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

    let main_url = Url::from(&String::from("http://www.google.com"));
    let document = html_parser::parse(tokens, &main_url);
    assert_eq!(document.document_node.children.as_ref().unwrap().len(), 1);

    let div_node = document.document_node.children.as_ref().unwrap()[0].as_ref();
    let text_node = div_node.children.as_ref().unwrap()[0].as_ref();
    assert_text_on_node_is(text_node, "two words");
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

    let main_url = Url::from(&String::from("http://www.google.com"));
    let document = html_parser::parse(tokens, &main_url);
    assert_eq!(document.document_node.children.as_ref().unwrap().len(), 1);

    //TODO: it would be much nicer if we can just compare with a tree of nodes here, that we layout like in json, or just with tabs

    assert_element_name_is(document.document_node.children.as_ref().unwrap()[0].as_ref(), "div");

    let div_childs = document.document_node.children.as_ref().unwrap()[0].as_ref().children.as_ref().unwrap();
    assert_eq!(div_childs.len(), 1);
    assert_element_name_is(div_childs[0].as_ref(), "b");

    let b_childs = div_childs[0].as_ref().children.as_ref().unwrap();
    assert_eq!(b_childs.len(), 1);
    assert_element_name_is(b_childs[0].as_ref(), "p");
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

    let main_url = Url::from(&String::from("http://www.google.com"));
    let document = html_parser::parse(tokens, &main_url);
    assert_eq!(document.document_node.children.as_ref().unwrap().len(), 1);

    assert_element_name_is(document.document_node.children.as_ref().unwrap()[0].as_ref(), "div");

    let div_childs = document.document_node.children.as_ref().unwrap()[0].as_ref().children.as_ref().unwrap();
    assert_eq!(div_childs.len(), 1);
    assert_element_name_is(div_childs[0].as_ref(), "b");
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

    let main_url = Url::from(&String::from("http://www.google.com"));
    let document = html_parser::parse(tokens, &main_url);
    assert_eq!(document.document_node.children.as_ref().unwrap().len(), 1);

    assert_element_name_is(document.document_node.children.as_ref().unwrap()[0].as_ref(), "html");

    let html_childs = document.document_node.children.as_ref().unwrap()[0].as_ref().children.as_ref().unwrap();
    assert_eq!(html_childs.len(), 1);
    assert_element_name_is(html_childs[0].as_ref(), "body");

    let body_childs = html_childs[0].as_ref().children.as_ref().unwrap();
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
