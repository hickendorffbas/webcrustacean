use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::color::Color;
use crate::dom::{ElementDomNode, TagName};
use crate::style::{
    compute_styles,
    css_lexer,
    css_parser,
    resolve_full_styles_for_dom_node,
    StyleContext,
    StyleRule,
    Styles,
};
use crate::style::css_properties::{
    CssPropertyValue,
    Display,
    FontSize,
};


static NEXT_TEST_ID: AtomicUsize = AtomicUsize::new(1);
pub fn get_next_test_id() -> usize { NEXT_TEST_ID.fetch_add(1, Ordering::Relaxed) }


fn make_element_node(dom_node_id: usize, parent_node_id: usize, name: String) -> ElementDomNode {
    return ElementDomNode { internal_id: dom_node_id, parent_id: parent_node_id, text: None, is_document_node: false, dirty: false,
                            name: Some(name), name_for_layout: TagName::Other, children: Vec::new(), attributes: None,
                            image: None, image_load_task_id: None, page_component: None, styles: Styles::new(),
                            inline_style: Vec::new() };
}


#[test]
fn test_basic_style_resolving() {
    let document_node_id = 0;
    let dom_node_id = get_next_test_id();
    let dom_node = Rc::new(RefCell::from(make_element_node(dom_node_id, document_node_id, "b".to_owned())));

    let mut all_dom_nodes = HashMap::new();
    all_dom_nodes.insert(dom_node_id, Rc::clone(&dom_node));

    let style_rules = vec![ StyleRule::make_for_tag_name("b", CssPropertyValue::BackgroundColor(Color::BLUE)) ];
    let style_context = StyleContext { user_agent_sheet: Vec::new(), author_sheet: style_rules };
    let resolved_styles = resolve_full_styles_for_dom_node(&dom_node, &all_dom_nodes, &style_context);

    assert_eq!(resolved_styles.background_color(), Color::BLUE);
}


#[test]
fn test_inherit_style_from_parent() {
    let document_node_id = 0;
    let main_node_id = get_next_test_id();
    let parent_node_id = get_next_test_id();
    let main_node = Rc::new(RefCell::from(make_element_node(main_node_id, parent_node_id, "b".to_owned())));
    let parent_node = Rc::new(RefCell::from(make_element_node(parent_node_id, document_node_id, "h3".to_owned())));
    parent_node.borrow_mut().children.push(Rc::clone(&main_node));

    let mut all_dom_nodes = HashMap::new();
    all_dom_nodes.insert(main_node_id, Rc::clone(&main_node));
    all_dom_nodes.insert(parent_node_id, Rc::clone(&parent_node));

    let style_rules = vec![ StyleRule::make_for_tag_name("h3", CssPropertyValue::FontSize(FontSize::Px(50.0))) ];
    let style_context = StyleContext { user_agent_sheet: Vec::new(), author_sheet: style_rules };

    compute_styles(&parent_node, &all_dom_nodes, &style_context);

    assert_eq!(main_node.borrow().styles.font_size(), FontSize::Px(50.0));
}


#[test]
fn test_cascade() {
    let document_node_id = 0;
    let dom_node_id = get_next_test_id();
    let dom_node = Rc::new(RefCell::from(make_element_node(dom_node_id, document_node_id, "b".to_owned())));

    let mut all_dom_nodes = HashMap::new();
    all_dom_nodes.insert(dom_node_id, Rc::clone(&dom_node));

    let style_rules = vec![
        StyleRule::make_for_tag_name("b", CssPropertyValue::Color(Color::new(255, 0, 0))),
        StyleRule::make_for_tag_name("b", CssPropertyValue::FontSize(FontSize::Px(25.0))),
    ];
    let ua_styles = vec![ StyleRule::make_for_tag_name("b", CssPropertyValue::Color(Color::new(255, 0, 0))) ];

    let style_context = StyleContext { user_agent_sheet: ua_styles, author_sheet: style_rules };

    let resolved_styles = resolve_full_styles_for_dom_node(&dom_node, &all_dom_nodes, &style_context);

    assert_eq!(resolved_styles.font_size(), FontSize::Px(25.0));
    assert_eq!(resolved_styles.color(), Color::new(255, 0, 0));
}


#[test]
fn test_resolve_display_with_multiple_keywords() {
    let document_node_id = 0;
    let dom_node_id = get_next_test_id();
    let dom_node = Rc::new(RefCell::from(make_element_node(dom_node_id, document_node_id, "b".to_owned())));

    let mut all_dom_nodes = HashMap::new();
    all_dom_nodes.insert(dom_node_id, Rc::clone(&dom_node));

    let tokens = css_lexer::lex_css("b { display: block flex; }", 1, 1);
    let style_context = StyleContext { user_agent_sheet: Vec::new(), author_sheet: css_parser::parse_css(&tokens) };
    let resolved_styles = resolve_full_styles_for_dom_node(&dom_node, &all_dom_nodes, &style_context);

    assert_eq!(resolved_styles.display(), Display::Inline); //For now multiple values don't work yet, so we fallback to the default (but without crashing)
}
