use crate::dom::DomNode;
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

    //TODO: the code below could use some cleanups
    let doc_children = enum_as_variant!(document.document_node.as_ref(), DomNode::Document).children.as_ref();
    assert_eq!(doc_children.unwrap().len(), 2);
    let generic_a_node = doc_children.unwrap().get(0).unwrap().as_ref();
    let a_node = enum_as_variant!(generic_a_node, DomNode::Element);
    assert_eq!(a_node.name.clone().unwrap(), "a");

    assert_eq!(a_node.children.as_ref().unwrap().len(), 1);
    let generic_text_node = a_node.children.as_ref().unwrap().get(0).unwrap().as_ref();
    let text_node = enum_as_variant!(generic_text_node, DomNode::Text);
    assert_eq!(text_node.text_content, "text");
}
