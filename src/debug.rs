use crate::dom::Document;
use crate::html_lexer::HtmlTokenWithLocation;
use crate::layout::LayoutNode;


#[cfg(debug_assertions)] use std::rc::Rc;
#[cfg(debug_assertions)] use crate::dom::DomNode;

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
            println!("{}ATTR: ({} = {}) ({}) (parent: {})", indent, node.name, node.value, node.internal_id, node.parent_id);
        }
        DomNode::Text(node) => {
            println!("{}TEXT: {} ({}) (parent: {})", indent, node.text_content, node.internal_id, node.parent_id);
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
pub fn debug_print_layout_tree(_: &LayoutNode) {}
#[allow(dead_code)]
#[cfg(debug_assertions)]
pub fn debug_print_layout_tree(node: &Rc<LayoutNode>) {
    println!("== dumping layout tree");
    debug_print_layout_tree_with_indent(node, 0);
    println!("== done dumping layout tree");
}


#[cfg(debug_assertions)]
fn debug_print_layout_tree_with_indent(node: &Rc<LayoutNode>, indent_cnt: u32) {
    let mut indent = String::new();
    for _ in 0..indent_cnt {
        indent.push(' ');
    }

    let mut rect_str = String::new();
    for rect in node.rects.borrow().iter() {
        rect_str.push_str(format!("LayoutRect({:?} {:?} {})", rect.location, rect.text, if rect.image.is_some() {"IMG"} else {""}, ).as_str());
    }

    println!("{}{:?} ({}) (parent: {}) {:?}", indent, rect_str, node.internal_id, node.parent_id, node.styles);

    if node.children.is_some() {
        for child in node.children.clone().unwrap() {
            debug_print_layout_tree_with_indent(&child, indent_cnt + INDENT_AMOUNT)
        }
    }
}
