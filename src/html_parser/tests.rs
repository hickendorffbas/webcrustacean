use std::rc::Rc;

use crate::dom::{Document, DomNode};
use crate::enum_as_variant;
use crate::html_parser;
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

    let document = html_parser::parse(tokens);

    let document_elements = get_ref_to_document_children(&document);
    assert_eq!(document_elements.len(), 2);

    let generic_a_node = document_elements[0].as_ref();
    assert_element_name_is(generic_a_node, "a");

    let a_children = get_children(generic_a_node);
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

    let document = html_parser::parse(tokens);

    let document_elements = get_ref_to_document_children(&document);
    assert_eq!(document_elements.len(), 1);

    let div_node = get_children(document_elements[0].as_ref());
    assert_text_on_node_is(div_node[0].as_ref(), "two words");
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

    let document = html_parser::parse(tokens);

    let document_elements = get_ref_to_document_children(&document);
    assert_eq!(document_elements.len(), 1);

    //TODO: it would be much nicer if we can just compare with a tree of nodes here, that we layout like in json, or just with tabs

    assert_element_name_is(document_elements[0].as_ref(), "div");

    let div_childs = get_children(document_elements[0].as_ref());
    assert_eq!(div_childs.len(), 1);
    assert_element_name_is(div_childs[0].as_ref(), "b");

    let b_childs = get_children(div_childs[0].as_ref());
    assert_eq!(b_childs.len(), 1);
    assert_element_name_is(b_childs[0].as_ref(), "p");
}


//TODO: add a test for the case where we close a tag that we did not open


fn get_ref_to_document_children(document: &Document) -> &Vec<Rc<DomNode>> {
    return enum_as_variant!(document.document_node.as_ref(), DomNode::Document).children.as_ref().unwrap();
}

fn get_children(node: &DomNode) -> &Vec<Rc<DomNode>> {
    enum_as_variant!(node, DomNode::Element).children.as_ref().unwrap()
}

fn assert_element_name_is(node: &DomNode, name: &str) {
    let element_node = enum_as_variant!(node, DomNode::Element);
    assert_eq!(element_node.name.as_ref().unwrap(), name);
}

fn assert_text_on_node_is(node: &DomNode, text: &str) {
    let text_node = enum_as_variant!(node, DomNode::Text);
    assert_eq!(text_node.text_content, text);
}
