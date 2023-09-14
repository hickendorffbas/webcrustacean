use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::dom::{TextDomNode, DomNode};
use crate::layout::{
    Display,
    LayoutNode,
    LayoutRect,
};
use crate::style;


#[test]
fn test_basic_style_resolving() {
    //TODO: this test feels more verbose than neeeded
    //TODO: this test is not valid anymore... (we need to use the styleRule's now)

    let child_styles = vec![
        style::Style { property: "font-weight".to_owned(), value: "bold".to_owned() },
        style::Style { property: "not-present-in-parent".to_owned(), value: "value".to_owned() }
    ];
    let child_node = build_new_layout_node(2, 1, None, child_styles);
    let rc_child_node = Rc::new(child_node);
    let rc_child_clone = Rc::clone(&rc_child_node);
    let rc_child_2nd_clone = Rc::clone(&rc_child_node);  //TODO: yikes :(

    let styles = vec![
        style::Style { property: "font-weight".to_owned(), value: "2em".to_owned() },
        style::Style { property: "not-overridden".to_owned(), value: "yes".to_owned() },
    ];
    let children = Some(vec![rc_child_node]);
    let layout_node = build_new_layout_node(1, 1, children, styles);

    let mut all_nodes = HashMap::new();
    let rc_node = Rc::new(layout_node);
    all_nodes.insert(rc_node.internal_id, rc_node);
    all_nodes.insert(rc_child_clone.internal_id, rc_child_clone);

    let resolved_styles = style::resolve_full_styles_for_layout_node(&rc_child_2nd_clone, &all_nodes, &Vec::new());

    assert_eq!(resolved_styles.len(), 3);
    assert!(resolved_styles.iter().any(|el| { el.property == "font-weight".to_owned() && el.value == "bold".to_owned() }));
    assert!(resolved_styles.iter().any(|el| { el.property == "not-present-in-parent".to_owned() && el.value == "value".to_owned() }));
    assert!(resolved_styles.iter().any(|el| { el.property == "not-overridden".to_owned() && el.value == "yes".to_owned() }));
}

fn build_new_layout_node(id: usize, parent_id: usize, children: Option<Vec<Rc<LayoutNode>>>, styles: Vec<style::Style>) -> LayoutNode {

    let text_dom_node = DomNode::Text(TextDomNode{ internal_id: 1, text_content: "test".to_owned(), parent_id: 0, non_breaking_space_positions: None });

    return LayoutNode {
        internal_id: id,
        display: Display::Block,
        visible: true,
        line_break: false,
        children: children,
        parent_id: parent_id,
        styles: RefCell::new(styles),
        optional_link_url: None,
        rects: RefCell::new(vec![LayoutRect::get_default_non_computed_rect()]),
        from_dom_node: Some(Rc::new(text_dom_node))
    };
}
