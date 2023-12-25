use std::rc::Rc;
use std::cell::RefCell;

use crate::dom::Document;
use crate::html_lexer::HtmlTokenWithLocation;
use crate::layout::LayoutNode;

#[cfg(debug_assertions)] use crate::dom::ElementDomNode;

#[cfg(debug_assertions)] const INDENT_AMOUNT: u32 = 2;


//TODO: a few of these should probably output json (they are trees, mostly), so I can nicely format them, and collapse parts.


#[allow(dead_code)]
#[cfg(not(debug_assertions))]
pub fn debug_print_dom_tree(_: &Document, _: &str) {}
#[allow(dead_code)]
#[cfg(debug_assertions)]
pub fn debug_print_dom_tree(document: &Document) {
    debug_print_dom_node_tree_with_indent(&document.document_node, 0);
}


#[cfg(debug_assertions)]
fn debug_print_dom_node_tree_with_indent(dom_node: &Rc<RefCell<ElementDomNode>>, indent_cnt: u32) {
    let dom_node = dom_node.borrow();

    let mut indent = String::new();
    for _ in 0..indent_cnt {
        indent.push(' ');
    }

    if dom_node.text.is_some() {
        debug_assert!(dom_node.children.is_none());
        debug_assert!(dom_node.attributes.is_none());

        println!("{}TEXT: \"{}\" ({}) (parent: {})", indent, dom_node.text.as_ref().unwrap().text_content, dom_node.internal_id, dom_node.parent_id);

    } else {
        debug_assert!(dom_node.text.is_none());

        println!("{}{} ({}) (parent: {})", indent, dom_node.name.clone().unwrap_or("".to_owned()), dom_node.internal_id, dom_node.parent_id);

        if dom_node.attributes.is_some() {
            for att in dom_node.attributes.as_ref().unwrap() {
                let att = att.borrow();
                println!("{}ATTR: ({} = {}) (parent: {})", indent, att.name, att.value, att.parent_id);
            }
        }

        if dom_node.children.is_some() {
            for child in dom_node.children.clone().unwrap() {
                debug_print_dom_node_tree_with_indent(&child, indent_cnt + INDENT_AMOUNT);
            }
        }
    }
}


#[cfg(not(debug_assertions))]
pub fn debug_log_warn<S: AsRef<str>>(_: S) {}
#[cfg(debug_assertions)]
pub fn debug_log_warn<S: AsRef<str>>(warning_text: S) {
    println!("WARN: {}", warning_text.as_ref());
}


#[allow(dead_code)]
#[cfg(not(debug_assertions))]
pub fn debug_print_html_tokens(_: &Vec<HtmlTokenWithLocation>) {}
#[allow(dead_code)]
#[cfg(debug_assertions)]
pub fn debug_print_html_tokens(tokens: &Vec<HtmlTokenWithLocation>) {
    let mut buffer = String::new();

    //TODO: this is a quick and dirty way, it prints a lot of overhead (like HtmlToken all the time), could be a lot nicer

    for token in tokens {
        buffer = format!("{} {:?} {}:{}", buffer, token.html_token, token.line, token.character);
    }

    println!("tokenlist: {:?}", tokens);
}


#[allow(dead_code)]
#[cfg(not(debug_assertions))]
pub fn debug_print_layout_tree(_: &Rc<RefCell<LayoutNode>>) {}
#[allow(dead_code)]
#[cfg(debug_assertions)]
pub fn debug_print_layout_tree(node: &Rc<RefCell<LayoutNode>>) {
    println!("== dumping layout tree");
    debug_print_layout_tree_with_indent(node, 0);
    println!("== done dumping layout tree");
}


#[cfg(debug_assertions)]
fn debug_print_layout_tree_with_indent(node: &Rc<RefCell<LayoutNode>>, indent_cnt: u32) {
    let mut indent = String::new();
    for _ in 0..indent_cnt {
        indent.push(' ');
    }

    let node = node.borrow();

    let mut rect_str = String::new();
    for rect in node.rects.iter() {
        rect_str.push_str(format!("LayoutRect({:?} {:?} {})", rect.location, rect.text, if rect.image.is_some() {"IMG"} else {""}, ).as_str());
    }

    let visible = if node.visible {
        ""
    } else {
        "!visible"
    };

    println!("{}{:?} ({}) (parent: {}) {:?} {}", indent, rect_str, node.internal_id, node.parent_id, node.styles, visible);

    if node.children.is_some() {
        for child in node.children.clone().unwrap() {
            debug_print_layout_tree_with_indent(&child, indent_cnt + INDENT_AMOUNT)
        }
    }
}
