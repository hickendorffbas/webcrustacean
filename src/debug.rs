use std::rc::Rc;

use crate::dom::{Document, DomNode};

const INDENT_AMOUNT: u32 = 2;

#[allow(dead_code)]
pub fn debug_print_dom_tree(document: &Document, dump_name: &str) {
    if cfg!(debug_assertions) {
        println!("== dumping html node tree for {}", dump_name);
        debug_print_html_node_tree_with_indent(&document.document_node, 0);
        println!("== done dumping for {}", dump_name);
    }
}

#[cfg(not(debug_assertions))]
fn debug_print_html_node_tree_with_indent(dom_node: &Rc<DomNode>, indent_cnt: u32) {}
#[allow(dead_code)]
#[cfg(debug_assertions)]
fn debug_print_html_node_tree_with_indent(dom_node: &Rc<DomNode>, indent_cnt: u32) {
    let mut indent = String::new();
    for _ in 0..indent_cnt {
        indent.push(' ');
    }

    match dom_node.as_ref() {
        DomNode::Document(node) => {
            println!("{}{} ({})", indent, "<*Document>", node.internal_id);

            if node.children.is_some() {
                for child in node.children.clone().unwrap() {
                    debug_print_html_node_tree_with_indent(&child, indent_cnt + INDENT_AMOUNT)
                }
            }
        }
        DomNode::Element(node) => {
            println!("{}{} ({})", indent, node.name.clone().unwrap_or("".to_owned()), node.internal_id);

            if node.children.is_some() {
                for child in node.children.clone().unwrap() {
                    debug_print_html_node_tree_with_indent(&child, indent_cnt + INDENT_AMOUNT)
                }
            }
        }
        DomNode::Attribute(_) => {
            //TODO: eventually I should print these too
            panic!("TODO: implement debug printing of attribute DOM nodes")
        }
        DomNode::Text(node) => {
            println!("{}{} ({})", indent, node.text_content.clone().unwrap_or("".to_owned()), node.internal_id);
        }
    }
}
