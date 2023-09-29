use std::collections::HashMap;
use std::rc::Rc;


use crate::style::{
    Selector,
    StyleContext,
    StyleRule,
    resolve_full_styles_for_layout_node,
};
use crate::dom::{
    DocumentDomNode,
    DomNode,
    ElementDomNode
};
use crate::test_util::get_next_test_id;


fn check_style(resolved_styles: &HashMap<String, String>, property: &str, value: &str) {
    assert!(resolved_styles.contains_key(property));
    assert_eq!(resolved_styles.get(property).unwrap(), value);
}


#[test]
fn test_basic_style_resolving() {
    let document_node_id = get_next_test_id();
    let dom_node_id = get_next_test_id();
    let dom_node = Rc::new(DomNode::Element(ElementDomNode { internal_id: dom_node_id, name: Some("a".to_owned()),
                                                             children: Some(Vec::new()), parent_id: document_node_id }));
    let document_node = Rc::new(DomNode::Document(DocumentDomNode { internal_id: document_node_id, children: Some(vec![Rc::clone(&dom_node)])}));

    let mut all_dom_nodes = HashMap::new();
    all_dom_nodes.insert(dom_node_id, Rc::clone(&dom_node));
    all_dom_nodes.insert(document_node_id, Rc::clone(&document_node));

    let style_rules = vec![ StyleRule { selector: Selector { nodes: Some(vec!["a".to_owned()]) },
                                        property: "prop".to_owned(), value: "some value".to_owned() } ];

    let style_context = StyleContext { user_agent_sheet: Vec::new(), author_sheet: style_rules };

    let resolved_styles = resolve_full_styles_for_layout_node(&dom_node, &all_dom_nodes, &style_context);

    check_style(&resolved_styles, "prop", "some value");
}


#[test]
fn test_inherit_style_from_parent() {
    let document_node_id = get_next_test_id();
    let main_node_id = get_next_test_id();
    let parent_node_id = get_next_test_id();
    let main_node = Rc::new(DomNode::Element(ElementDomNode { internal_id: main_node_id, name: Some("a".to_owned()),
                                                              children: Some(Vec::new()), parent_id: parent_node_id }));
    let parent_node = Rc::new(DomNode::Element(ElementDomNode { internal_id: parent_node_id, name: Some("h3".to_owned()),
                                                                children: Some(vec![Rc::clone(&main_node)]),parent_id: document_node_id }));
    let document_node = Rc::new(DomNode::Document(DocumentDomNode { internal_id: document_node_id, children: Some(vec![Rc::clone(&parent_node)])}));

    let mut all_dom_nodes = HashMap::new();
    all_dom_nodes.insert(main_node_id, Rc::clone(&main_node));
    all_dom_nodes.insert(parent_node_id, Rc::clone(&parent_node));
    all_dom_nodes.insert(document_node_id, Rc::clone(&document_node));

    let style_rules = vec![ StyleRule { selector: Selector { nodes: Some(vec!["h3".to_owned()]) },
                                        property: "font-size".to_owned(), value: "50".to_owned() } ];

    let style_context = StyleContext { user_agent_sheet: Vec::new(), author_sheet: style_rules };

    let resolved_styles = resolve_full_styles_for_layout_node(&main_node, &all_dom_nodes, &style_context);

    check_style(&resolved_styles, "font-size", "50");
}


#[test]
fn test_cascade() {
    let document_node_id = get_next_test_id();
    let dom_node_id = get_next_test_id();
    let dom_node = Rc::new(DomNode::Element(ElementDomNode { internal_id: dom_node_id, name: Some("a".to_owned()),
                                                             children: Some(Vec::new()), parent_id: document_node_id }));
    let document_node = Rc::new(DomNode::Document(DocumentDomNode { internal_id: document_node_id, children: Some(vec![Rc::clone(&dom_node)])}));

    let mut all_dom_nodes = HashMap::new();
    all_dom_nodes.insert(dom_node_id, Rc::clone(&dom_node));
    all_dom_nodes.insert(document_node_id, Rc::clone(&document_node));

    let style_rules = vec![ StyleRule { selector: Selector { nodes: Some(vec!["a".to_owned()]) },
                                        property: "color".to_owned(), value: "red".to_owned() },
                            StyleRule { selector: Selector { nodes: Some(vec!["a".to_owned()]) },
                                        property: "font-size".to_owned(), value: "25".to_owned() } ];
    let ua_styles = vec![ StyleRule { selector: Selector { nodes: Some(vec!["a".to_owned()]) },
                                      property: "color".to_owned(), value: "red".to_owned() } ];

    let style_context = StyleContext { user_agent_sheet: ua_styles, author_sheet: style_rules };

    let resolved_styles = resolve_full_styles_for_layout_node(&dom_node, &all_dom_nodes, &style_context);

    check_style(&resolved_styles, "color", "red");
    check_style(&resolved_styles, "font-size", "25");
}
