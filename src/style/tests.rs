use std::collections::HashMap;
use std::rc::Rc;


use crate::style::{
    Selector,
    StyleRule,
    get_default_styles,
    resolve_full_styles_for_layout_node,
};
use crate::dom::{DocumentDomNode, DomNode, ElementDomNode};
use crate::test_util::get_next_test_id;



#[test]
fn test_basic_style_resolving() {

    let document_node_id = get_next_test_id();
    let main_node_id = get_next_test_id();
    let dom_node = Rc::new(DomNode::Element(ElementDomNode { internal_id: main_node_id, name: Some("a".to_owned()),
                                                             children: Some(Vec::new()), parent_id: document_node_id }));
    let document_node = Rc::new(DomNode::Document(DocumentDomNode { internal_id: document_node_id, children: Some(vec![Rc::clone(&dom_node)])}));

    let mut all_dom_nodes = HashMap::new();
    all_dom_nodes.insert(main_node_id, Rc::clone(&dom_node));
    all_dom_nodes.insert(document_node_id, Rc::clone(&document_node));

    let style_rules = vec![ StyleRule { selector: Selector { wildcard: false, nodes: Some(vec!["a".to_owned()]) },
                                        property: "prop".to_owned(), value: "some value".to_owned() } ];

    let resolved_styles = resolve_full_styles_for_layout_node(&dom_node, &all_dom_nodes, &style_rules);

    assert_eq!(resolved_styles.len(), 1 + get_default_styles().len());
    assert!(resolved_styles.contains_key("prop"));
    assert_eq!(resolved_styles.get("prop").unwrap(), "some value");
}


#[test]
fn test_overwrite_default_style() {

    let document_node_id = get_next_test_id();
    let main_node_id = get_next_test_id();
    let dom_node = Rc::new(DomNode::Element(ElementDomNode { internal_id: main_node_id, name: Some("a".to_owned()),
                                                             children: Some(Vec::new()), parent_id: document_node_id }));
    let document_node = Rc::new(DomNode::Document(DocumentDomNode { internal_id: document_node_id, children: Some(vec![Rc::clone(&dom_node)])}));

    let mut all_dom_nodes = HashMap::new();
    all_dom_nodes.insert(main_node_id, Rc::clone(&dom_node));
    all_dom_nodes.insert(document_node_id, Rc::clone(&document_node));

    let style_rules = vec![ StyleRule { selector: Selector { wildcard: false, nodes: Some(vec!["a".to_owned()]) },
                                        property: "font-size".to_owned(), value: "3".to_owned() } ];

    let resolved_styles = resolve_full_styles_for_layout_node(&dom_node, &all_dom_nodes, &style_rules);

    assert_eq!(resolved_styles.len(), get_default_styles().len());
}


//TODO: add a test with a series of nodes overwriting each others styles
