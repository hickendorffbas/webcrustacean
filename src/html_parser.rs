//TODO: eventually this should become the actual parser, and be renamed to remove "next gen" from it

use std::collections::HashMap;
use std::rc::Rc;

use crate::debug::debug_log_warn;
use crate::dom::{
    Document,
    DocumentDomNode,
    DomNode,
    ElementDomNode,
    TextDomNode,
    get_next_dom_node_interal_id, AttributeDomNode,
};
use crate::html_lexer::HtmlToken;


#[cfg(test)]
mod tests;


pub fn parse(html_tokens: Vec<HtmlToken>) -> Document {
    let mut all_nodes = HashMap::new();

    let mut children = Vec::new();
    let root_node_internal_id = get_next_dom_node_interal_id();

    let mut current_token_idx = 0;

    while current_token_idx < html_tokens.len() {
        children.push(parse_node(&html_tokens, &mut current_token_idx, root_node_internal_id, &mut all_nodes));
        current_token_idx += 1;
    }
    let root_node = DomNode::Document(DocumentDomNode { internal_id: root_node_internal_id, children: Some(children)});


    let rc_root_node = Rc::new(root_node);
    all_nodes.insert(root_node_internal_id, Rc::clone(&rc_root_node));

    return Document { document_node: rc_root_node, all_nodes };
}


fn parse_node(html_tokens: &Vec<HtmlToken>, current_token_idx: &mut usize, parent_id: usize,
              all_nodes: &mut HashMap<usize, Rc<DomNode>>) -> Rc<DomNode> {
    let node_being_build_internal_id = get_next_dom_node_interal_id();

    let mut tag_being_parsed = None;
    let mut children = Vec::new();
    let mut last_child_was_whitespace = false;

    'token_loop: while *current_token_idx < html_tokens.len() {  //TODO: we are going to have to break from this loop when we have parsed 1 node exactly
        let current_token = html_tokens.get(*current_token_idx).unwrap();

        match current_token {
            HtmlToken::OpenTag { name } => {
                if tag_being_parsed.is_none() {
                    tag_being_parsed = Some(name.clone());
                } else {
                    let new_node = parse_node(html_tokens, current_token_idx, node_being_build_internal_id, all_nodes);
                    children.push(new_node);
                    last_child_was_whitespace = false;
                }
            },
            HtmlToken::OpenTagEnd => {
                //I think I can just ignore this for now, it would just seperate attributes from tag children
            },
            HtmlToken::Attribute(token) => {
                let id_of_attr_node = get_next_dom_node_interal_id();

                let new_node = DomNode::Attribute(AttributeDomNode {
                    internal_id: id_of_attr_node,
                    name: token.name.clone(),
                    value: token.value.clone(),
                    parent_id: node_being_build_internal_id,
                });

                let rc_node = Rc::new(new_node);
                let rc_clone_node = Rc::clone(&rc_node);
                children.push(rc_node);
                last_child_was_whitespace = false;
                all_nodes.insert(id_of_attr_node, rc_clone_node);

            },
            HtmlToken::CloseTag { name } => {
                if tag_being_parsed.is_none() || name != tag_being_parsed.as_ref().unwrap() {
                    //TODO: this is a case that can happen in the real world of course, figure out how to handle this...
                    debug_log_warn("We are not closing the tag we opened, something is wrong!".to_owned());
                }

                let new_node = DomNode::Element(ElementDomNode {
                    internal_id: node_being_build_internal_id,
                    name: tag_being_parsed,
                    children: Some(children),
                    parent_id,
                });

                let rc_node = Rc::new(new_node);
                all_nodes.insert(node_being_build_internal_id, Rc::clone(&rc_node));
                return rc_node;
            },
            HtmlToken::Text(text) => {
                //TODO: check how this is done in actual DOM's, but I think we should include whitespace in here, instead of seperate nodes
                let id_for_text_node = get_next_dom_node_interal_id();
                let new_node = DomNode::Text(TextDomNode {
                    internal_id: id_for_text_node,
                    text_content: text.to_string(), //TODO: using to_string here feels wrong...
                    parent_id: node_being_build_internal_id
                });

                let rc_node = Rc::new(new_node);
                let rc_clone_node = Rc::clone(&rc_node);

                if tag_being_parsed.is_some() {
                    children.push(rc_node);
                    last_child_was_whitespace = false;
                    all_nodes.insert(id_for_text_node, rc_clone_node);
                } else {
                    all_nodes.insert(id_for_text_node, rc_clone_node);
                    return rc_node;
                }
            },
            HtmlToken::Whitespace(_) => {
                if last_child_was_whitespace {
                    *current_token_idx += 1;
                    continue 'token_loop;
                }

                let id_for_whitespace_node = get_next_dom_node_interal_id();
                let new_node = DomNode::Text(TextDomNode {
                    internal_id: id_for_whitespace_node,
                    //Since html does not care about what the whitespace is, but we can't
                    //have no whitespace at all (between words for example), we emit a space
                    text_content: " ".to_owned(),
                    parent_id: if tag_being_parsed.is_some() { node_being_build_internal_id } else { parent_id },
                });
                let rc_node = Rc::new(new_node);
                let rc_clone_node = Rc::clone(&rc_node);

                if tag_being_parsed.is_some() {
                    children.push(rc_node);
                    last_child_was_whitespace = true;

                    all_nodes.insert(id_for_whitespace_node, rc_clone_node);
                } else {
                    all_nodes.insert(id_for_whitespace_node, rc_clone_node);
                    return rc_node;
                }
            },
            HtmlToken::Comment(_) => {},
            HtmlToken::Doctype(_) => {
                //for now we ignore, eventually we should probably distinguish html5 and other html variants here
            }
            HtmlToken::Entity(_) => {
                //TODO: implement this
            }
        }

        *current_token_idx += 1;
    }

    panic!("this should not happen");
}
