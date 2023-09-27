use std::collections::HashMap;
use std::rc::Rc;


use crate::style::{
    Selector,
    StyleRule,
    resolve_full_styles_for_layout_node, StyleContext, get_user_agent_style_sheet,
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

    let style_rules = vec![ StyleRule { selector: Selector { nodes: Some(vec!["a".to_owned()]) },
                                        property: "prop".to_owned(), value: "some value".to_owned() } ];

    //TODO: don't use get_user_agent_style_sheet() below, but setup actual testdata for it...
    let style_context = StyleContext { user_agent_sheet: get_user_agent_style_sheet(), author_sheet: style_rules };

    let resolved_styles = resolve_full_styles_for_layout_node(&dom_node, &all_dom_nodes, &style_context);

    assert!(resolved_styles.contains_key("prop"));
    assert_eq!(resolved_styles.get("prop").unwrap(), "some value");
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

    //TODO: don't use get_user_agent_style_sheet() below, but setup actual testdata for it...
    let style_context = StyleContext { user_agent_sheet: get_user_agent_style_sheet(), author_sheet: style_rules };

    let resolved_styles = resolve_full_styles_for_layout_node(&main_node, &all_dom_nodes, &style_context);

    assert!(resolved_styles.contains_key("font-size"));
    assert_eq!(resolved_styles.get("font-size").unwrap(), "50");
}
