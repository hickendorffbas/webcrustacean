use crate::html_parser::HtmlNode;

const INDENT_AMOUNT: u32 = 2;

#[allow(dead_code)]
pub fn debug_print_html_node_tree(nodes: &Vec<HtmlNode>, dump_name: &str) {
    if cfg!(debug_assertions) {
        println!("== dumping html node tree for {}", dump_name);
        for node in nodes {
            debug_print_html_node_tree_with_indent(node, 0);
        }
        println!("== done dumping for {}", dump_name);
    }
}

#[cfg(not(debug_assertions))]
fn debug_print_html_node_tree_with_indent(node: &HtmlNode, indent_cnt: u32) {}
#[allow(dead_code)]
#[cfg(debug_assertions)]
fn debug_print_html_node_tree_with_indent(node: &HtmlNode, indent_cnt: u32) {
    let mut indent = String::new();
    for _ in 0..indent_cnt {
        indent.push(' ');
    }

    println!("{}{:?} - {:?} - {:?}", indent, node.node_type, node.text_content, node.tag_name);

    match &node.children {
        Some(children) => 
            for child in children.iter() {
                debug_print_html_node_tree_with_indent(child, indent_cnt + INDENT_AMOUNT)
            },
        None => ()
    }

}
