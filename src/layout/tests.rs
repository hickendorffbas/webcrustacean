use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::dom::{Document, ElementDomNode};
use crate::jsonify::{
    dom_node_from_json,
    json_is_equal,
    layout_node_to_json,
};
use crate::layout::{build_full_layout, compute_layout};
use crate::network::url::Url;
use crate::platform::fonts::FontContext;
use crate::style::StyleContext;


#[test]
fn test_basic_paragraph_layout() {

    //TODO: this should become a file (or maybe only when big?)
    //      possibly always, so we can define a function taking 2 filenames, and maybe a list of styles etc, so all the other code can be hidden
    let dom_json = r#"
        { "text": "this is a test" }
    "#.to_owned();

    let main_dom_node = Rc::from(RefCell::from(dom_node_from_json(&dom_json)));

    let mut all_nodes = HashMap::new();
    build_all_nodes_from_document_node(&main_dom_node, &mut all_nodes);

    let style_context = StyleContext { user_agent_sheet: Vec::new() , author_sheet: Vec::new() };
    let font_context = FontContext::new();

    let document = Document {
        all_nodes: all_nodes,
        document_node: main_dom_node,
        style_context: style_context,
        base_url: Url::empty(),
    };

    let expected_layout_tree_json = r#"
        {
            "color": [255, 255, 255],
            "location": [0, 0, 87, 19],
            "childs": [
                {
                    "color": [255, 255, 255],
                    "boxes": [
                        {
                            "text": "this is a test",
                            "position": [0, 0, 87, 19]
                        }
                    ]
                }
            ]
        }"#;

    let tree = build_full_layout(&document, &font_context);
    compute_layout(&tree.root_node, 0.0, 0.0, &font_context, 0.0, false, true, 1000.0);
    let tree_json = layout_node_to_json(&tree.root_node.borrow());

    assert!(json_is_equal(&tree_json, &String::from(expected_layout_tree_json)));
}


fn build_all_nodes_from_document_node(dom_node: &Rc<RefCell<ElementDomNode>>, all_nodes_map: &mut HashMap<usize, Rc<RefCell<ElementDomNode>>>) {

    if dom_node.borrow().children.is_some() {
        for child in dom_node.borrow().children.as_ref().unwrap() {
            build_all_nodes_from_document_node(&child, all_nodes_map);
        }
    }

    let new_rc = Rc::clone(dom_node);
    all_nodes_map.insert(dom_node.borrow().internal_id, new_rc);
}
