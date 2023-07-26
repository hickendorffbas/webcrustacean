#[cfg(debug_assertions)] use std::rc::Rc;

#[cfg(debug_assertions)] use crate::{dom::{Document, DomNode}, html_parser::HtmlNode};
#[cfg(debug_assertions)] use crate::html_parser::HtmlNodeType;

#[cfg(debug_assertions)] const INDENT_AMOUNT: u32 = 2;


#[cfg(not(debug_assertions))] use crate::{dom::Document, html_parser::HtmlNode};


#[cfg(not(debug_assertions))]
pub fn debug_print_dom_tree(_: &Document, _: &str) {}
#[allow(dead_code)]
#[cfg(debug_assertions)]
pub fn debug_print_dom_tree(document: &Document, dump_name: &str) {
    println!("== dumping DOM node tree for {}", dump_name);
    debug_print_dom_node_tree_with_indent(&document.document_node, 0);
    println!("== done dumping DOM node tree for {}", dump_name);
}


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
            println!("{}{} ({}) (parent: {})", indent, node.text_content, node.internal_id, node.parent_id);
        }
    }
}


#[cfg(not(debug_assertions))]
pub fn debug_print_html_node(_: &HtmlNode, _: &str) {}
#[allow(dead_code)]
#[cfg(debug_assertions)]
pub fn debug_print_html_node(root_node: &HtmlNode, dump_name: &str) {
    println!("== dumping html node tree for {}", dump_name);
    debug_print_html_node_tree_with_indent(root_node, 0);
    println!("== done dumping html node tree for {}", dump_name);
}


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

            if html_node.children.is_some() {
                for child in html_node.children.as_ref().unwrap() {
                    debug_print_html_node_tree_with_indent(&child, indent_cnt + INDENT_AMOUNT);
                }
            }
        }
    }

}


#[cfg(not(debug_assertions))]
pub fn debug_log_warn(_: String) {}
#[cfg(debug_assertions)]
pub fn debug_log_warn(warning_text: String) {
    println!("WARN: {}", warning_text);
}
