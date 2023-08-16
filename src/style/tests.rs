use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::layout::{
    ComputedLocation,
    Display,
    LayoutNode
};
use crate::style;

use super::Style;


#[test]
fn test_basic_style_resolving() {
    //TODO: this test feels more verbose than neeeded

    let child_styles = vec![
        Style { name: "font-weight".to_owned(), value: "bold".to_owned() },
        Style { name: "not-present-in-parent".to_owned(), value: "value".to_owned() }
    ];
    let child_node = build_new_layout_node(2, 1, None, child_styles);
    let rc_child_node = Rc::new(child_node);
    let rc_child_clone = Rc::clone(&rc_child_node);
    let rc_child_2nd_clone = Rc::clone(&rc_child_node);  //TODO: yikes :(

    let styles = vec![
        Style { name: "font-weight".to_owned(), value: "2em".to_owned() },
        Style { name: "not-overridden".to_owned(), value: "yes".to_owned() },
    ];
    let children = Some(vec![rc_child_node]);
    let layout_node = build_new_layout_node(1, 1, children, styles);

    let mut all_nodes = HashMap::new();
    let rc_node = Rc::new(layout_node);
    all_nodes.insert(rc_node.internal_id, rc_node);
    all_nodes.insert(rc_child_clone.internal_id, rc_child_clone);

    let resolved_styles = style::resolve_full_styles_for_layout_node(&rc_child_2nd_clone, &all_nodes);

    assert_eq!( resolved_styles.len(), 3);
    assert!(resolved_styles.iter().any(|el| { el.name == "font-weight".to_owned() && el.value == "bold".to_owned() }));
    assert!(resolved_styles.iter().any(|el| { el.name == "not-present-in-parent".to_owned() && el.value == "value".to_owned() }));
    assert!(resolved_styles.iter().any(|el| { el.name == "not-overridden".to_owned() && el.value == "yes".to_owned() }));
}

fn build_new_layout_node(id: usize, parent_id: usize, children: Option<Vec<Rc<LayoutNode>>>, styles: Vec<Style>) -> LayoutNode {
    return LayoutNode {
        internal_id: id,
        text: None,
        location: RefCell::new(ComputedLocation::NotYetComputed),
        display: Display::Block,
        visible: true,
        optional_link_url: None,
        children: children,
        parent_id: parent_id,
        styles: styles,
    };
}
