use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;


use crate::dom::{ElementDomNode, TagName};
use crate::style::{
    CssProperty,
    resolve_full_styles_for_dom_node,
    StyleContext,
    StyleRule,
};
use crate::test_util::get_next_test_id;


fn check_style(resolved_styles: &HashMap<CssProperty, String>, property: &CssProperty, value: &str) {
    assert!(resolved_styles.contains_key(property));
    assert_eq!(resolved_styles.get(property).unwrap(), value);
}


#[test]
fn test_basic_style_resolving() {
    let document_node_id = 0;
    let dom_node_id = get_next_test_id();
    let dom_node = Rc::new(RefCell::from(ElementDomNode { internal_id: dom_node_id, parent_id: document_node_id, text: None, is_document_node: false, dirty: false,
                                                          name: Some("b".to_owned()), name_for_layout: TagName::Other, children: Some(Vec::new()),
                                                          attributes: None, image: None, img_job_tracker: None, scripts: None, page_component: None, styles: HashMap::new() }));

    let mut all_dom_nodes = HashMap::new();
    all_dom_nodes.insert(dom_node_id, Rc::clone(&dom_node));

    let style_rules = vec![ StyleRule::make_for_tag_name("b", CssProperty::BackgroundColor, "some value") ];
    let style_context = StyleContext { user_agent_sheet: Vec::new(), author_sheet: style_rules };
    let resolved_styles = resolve_full_styles_for_dom_node(&dom_node, &all_dom_nodes, &style_context);

    check_style(&resolved_styles, &CssProperty::BackgroundColor, "some value");
}


#[test]
fn test_inherit_style_from_parent() {
    let document_node_id = 0;
    let main_node_id = get_next_test_id();
    let parent_node_id = get_next_test_id();
    let main_node = Rc::new(RefCell::from(ElementDomNode { internal_id: main_node_id, parent_id: parent_node_id, text: None, is_document_node: false, dirty: false,
                                                           name: Some("b".to_owned()), name_for_layout: TagName::Other, children: Some(Vec::new()),
                                                           attributes: None, image: None, img_job_tracker: None, scripts: None, page_component: None, styles: HashMap::new() }));
    let parent_node = Rc::new(RefCell::from(ElementDomNode { internal_id: parent_node_id, parent_id: document_node_id, text: None, dirty: false,
                                                             is_document_node: false, name: Some("h3".to_owned()), name_for_layout: TagName::Other,
                                                             children: Some(vec![Rc::clone(&main_node)]), attributes: None, image: None, img_job_tracker: None,
                                                             scripts: None, page_component: None, styles: HashMap::new() }));

    let mut all_dom_nodes = HashMap::new();
    all_dom_nodes.insert(main_node_id, Rc::clone(&main_node));
    all_dom_nodes.insert(parent_node_id, Rc::clone(&parent_node));

    let style_rules = vec![ StyleRule::make_for_tag_name("h3", CssProperty::FontSize, "50") ];
    let style_context = StyleContext { user_agent_sheet: Vec::new(), author_sheet: style_rules };
    let resolved_styles = resolve_full_styles_for_dom_node(&main_node, &all_dom_nodes, &style_context);

    check_style(&resolved_styles, &CssProperty::FontSize, "50");
}


#[test]
fn test_cascade() {
    let document_node_id = 0;
    let dom_node_id = get_next_test_id();
    let dom_node = Rc::new(RefCell::from(ElementDomNode { internal_id: dom_node_id, parent_id: document_node_id, text: None, is_document_node: false, dirty: false,
                                                          name: Some("b".to_owned()), name_for_layout: TagName::Other, children: Some(Vec::new()),
                                                          attributes: None, image: None, img_job_tracker: None, scripts: None, page_component: None, styles: HashMap::new() }));

    let mut all_dom_nodes = HashMap::new();
    all_dom_nodes.insert(dom_node_id, Rc::clone(&dom_node));

    let style_rules = vec![
        StyleRule::make_for_tag_name("b", CssProperty::Color, "red"),
        StyleRule::make_for_tag_name("b", CssProperty::FontSize, "25"),
    ];
    let ua_styles = vec![ StyleRule::make_for_tag_name("b", CssProperty::Color, "red") ];

    let style_context = StyleContext { user_agent_sheet: ua_styles, author_sheet: style_rules };

    let resolved_styles = resolve_full_styles_for_dom_node(&dom_node, &all_dom_nodes, &style_context);

    check_style(&resolved_styles, &CssProperty::Color, "red");
    check_style(&resolved_styles, &CssProperty::FontSize, "25");
}
