use threadpool::ThreadPool;

use crate::dom::ElementDomNode;
use crate::html_parser;
use crate::network::url::Url;
use crate::resource_loader::ResourceThreadPool;
use crate::script::js_execution_context::JsExecutionContext;
use crate::test_util::*;

//TODO: I don't think it makes sense that we need to make this to test the parser, the structure seems wrong
fn test_resource_pool() -> ResourceThreadPool {
    return ResourceThreadPool { pool: ThreadPool::new(1) };
}



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
    let parse_result = html_parser::parse(tokens, &main_url, &mut test_resource_pool(), &mut JsExecutionContext::new());
    let document = parse_result;
    let doc_node = &document.document_node.borrow();
    assert_eq!(doc_node.children.as_ref().unwrap().len(), 2);

    let generic_a_node = doc_node.children.as_ref().unwrap()[0].borrow();
    assert_element_name_is(&generic_a_node, "a");

    let a_children = generic_a_node.children.as_ref().unwrap();
    assert_eq!(a_children.len(), 1);

    assert_text_on_node_is(&a_children[0].borrow(), "text");
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
    let parse_result = html_parser::parse(tokens, &main_url, &mut test_resource_pool(), &mut JsExecutionContext::new());
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

    let main_url = Url::from(&String::from("http://www.google.com"));
    let parse_result = html_parser::parse(tokens, &main_url, &mut test_resource_pool(), &mut JsExecutionContext::new());
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

    let main_url = Url::from(&String::from("http://www.google.com"));
    let parse_result = html_parser::parse(tokens, &main_url, &mut test_resource_pool(), &mut JsExecutionContext::new());
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


    let main_url = Url::from(&String::from("http://www.google.com"));
    let parse_result = html_parser::parse(tokens, &main_url, &mut test_resource_pool(), &mut JsExecutionContext::new());
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
