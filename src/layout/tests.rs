use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;


use crate::dom::{Document, ElementDomNode};
use crate::jsonify::{
    compare_json,
    dom_node_from_json,
    layout_node_to_json,
};
use crate::layout::build_full_layout;
use crate::network::url::Url;
use crate::platform::fonts::FontContext;
use crate::style::StyleContext;



#[test]
fn test_basic_paragraph_layout() {

    //TODO: this should become a file (or maybe only when big?)
    let dom_json = r#"
        { "text": "this is a test" }
    "#.to_owned();

    let main_dom_node = Rc::from(RefCell::from(dom_node_from_json(&dom_json)));

    let mut all_nodes = HashMap::new();
    build_all_nodes_from_document_node(&main_dom_node, &mut all_nodes);

    let document = Document {
        all_nodes: all_nodes,
        document_node: main_dom_node,
        style_context: StyleContext { user_agent_sheet: Vec::new() , author_sheet: Vec::new() },
        base_url: Url::empty(),
    };

    //TODO: this expected should at least also contain a position for the test to be useful, but that is not generated in the json yet
    let expected_layout_tree_json = r#"
        {
            "color": [255, 255, 255],
            "rects": [
                { "text": null }
            ],
            "childs": [
                {
                "color": [255, 255, 255],
                "rects": [
                    { "text": "this is a test" }
                ],
                "childs": []
                }
            ]
        }
    "#;

    let tree = build_full_layout(&document, &FontContext::new(), &Url::empty());
    let tree_json = layout_node_to_json(&tree.root_node.borrow());

    println!("got: {}", tree_json);

    assert!(compare_json(&tree_json, &String::from(expected_layout_tree_json)));
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
