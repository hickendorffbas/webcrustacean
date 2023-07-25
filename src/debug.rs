use std::rc::Rc;

use crate::{dom::{Document, DomNode}, html_parser::HtmlNode};
use crate::html_parser::HtmlNodeType;

const INDENT_AMOUNT: u32 = 2;

#[allow(dead_code)]
pub fn debug_print_dom_tree(document: &Document, dump_name: &str) {
    if cfg!(debug_assertions) {
        println!("== dumping DOM node tree for {}", dump_name);
        debug_print_dom_node_tree_with_indent(&document.document_node, 0);
        println!("== done dumping DOM node tree for {}", dump_name);
    }
}

#[cfg(not(debug_assertions))]
fn debug_print_dom_node_tree_with_indent(dom_node: &Rc<DomNode>, indent_cnt: u32) {}
#[allow(dead_code)]
#[cfg(debug_assertions)]
fn debug_print_dom_node_tree_with_indent(dom_node: &Rc<DomNode>, indent_cnt: u32) {
    let mut indent = String::new();
    for _ in 0..indent_cnt {
        indent.push(' ');
    }

    match dom_node.as_ref() {
        DomNode::Document(node) => {
            println!("{}{} ({})", indent, "<*Document>", node.internal_id);

            if node.children.is_some() {
                for child in node.children.clone().unwrap() {
                    debug_print_dom_node_tree_with_indent(&child, indent_cnt + INDENT_AMOUNT)
                }
            }
        }
        DomNode::Element(node) => {
            println!("{}{} ({}) (parent: {})", indent, node.name.clone().unwrap_or("".to_owned()), node.internal_id, node.parent_id);

            if node.children.is_some() {
                for child in node.children.clone().unwrap() {
                    debug_print_dom_node_tree_with_indent(&child, indent_cnt + INDENT_AMOUNT)
                }
            }
        }
        DomNode::Attribute(node) => {
            println!("{}ATTR: {} ({} = {}) (parent: {})", indent, node.name, node.value, node.internal_id, node.parent_id);
        }
        DomNode::Text(node) => {
            println!("{}{} ({}) (parent: {})", indent, node.text_content.clone().unwrap_or("".to_owned()), node.internal_id, node.parent_id);
        }
    }
}


//TODO: I'm not sure why we have both the if cfg, and the #[cfg...], seems duplicate?

#[allow(dead_code)]
pub fn debug_print_html_node(root_node: &HtmlNode, dump_name: &str) {
    if cfg!(debug_assertions) {
        println!("== dumping html node tree for {}", dump_name);
        debug_print_html_node_tree_with_indent(root_node, 0);
        println!("== done dumping html node tree for {}", dump_name);
    }
}

#[cfg(not(debug_assertions))]
fn debug_print_html_node_tree_with_indent(html_node: &HtmlNode, indent_cnt: u32) {}
#[allow(dead_code)]
#[cfg(debug_assertions)]
fn debug_print_html_node_tree_with_indent(html_node: &HtmlNode, indent_cnt: u32) {
    let mut indent = String::new();
    for _ in 0..indent_cnt {
        indent.push(' ');
    }

    match html_node.node_type {
        HtmlNodeType::Text => {
            if html_node.text_content.is_some() {
                println!("{}{}", indent, html_node.text_content.clone().map(|s| s.join(" ")).unwrap());
            } else {
                println!("{}{}", indent, "[EMPTY]");
            }
        },
        HtmlNodeType::Tag => {
            println!("{}{}", indent, html_node.tag_name.clone().unwrap());

            if (html_node.children.is_some()) {
                for child in html_node.children.as_ref().unwrap() {
                    debug_print_html_node_tree_with_indent(&child, indent_cnt + INDENT_AMOUNT);
                }
            }
        }
    }

}
