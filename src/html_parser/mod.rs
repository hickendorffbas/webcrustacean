use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::debug::debug_log_warn;
use crate::dom::{
    AttributeDomNode,
    Document,
    DocumentDomNode,
    DomNode,
    ElementDomNode,
    TextDomNode,
    get_next_dom_node_interal_id,
};
use crate::html_lexer::{HtmlToken, HtmlTokenWithLocation};
use crate::style::{
    StyleRule,
    StyleContext,
    css_lexer,
    css_parser,
    get_user_agent_style_sheet,
};


#[cfg(test)] mod tests;


const SELF_CLOSING_TAGS: [&str; 6] = ["br", "hr", "img", "input", "link", "meta"];


pub fn parse(html_tokens: Vec<HtmlTokenWithLocation>) -> Document {
    let mut all_nodes = HashMap::new();
    let mut document_style_rules = Vec::new();

    let mut children = Vec::new();
    let root_node_internal_id = get_next_dom_node_interal_id();

    let mut current_token_idx = 0;

    while current_token_idx < html_tokens.len() {
        children.push(parse_node(&html_tokens, &mut current_token_idx, root_node_internal_id, &mut all_nodes, &mut document_style_rules));
        current_token_idx += 1;
    }
    let root_node = DomNode::Document(DocumentDomNode { internal_id: root_node_internal_id, children: Some(children)});

    let rc_root_node = Rc::new(root_node);
    all_nodes.insert(root_node_internal_id, Rc::clone(&rc_root_node));

    let style_context = StyleContext {
        user_agent_sheet: get_user_agent_style_sheet(),
        author_sheet: document_style_rules,
    };
    return Document { document_node: rc_root_node, all_nodes, style_context };
}


fn parse_node(html_tokens: &Vec<HtmlTokenWithLocation>, current_token_idx: &mut usize, parent_id: usize,
              all_nodes: &mut HashMap<usize, Rc<DomNode>>, styles: &mut Vec<StyleRule>) -> Rc<DomNode> {
    let node_being_build_internal_id = get_next_dom_node_interal_id();

    let mut tag_being_parsed = None;
    let mut children = Vec::new();

    while *current_token_idx < html_tokens.len() {
        let current_token = html_tokens.get(*current_token_idx).unwrap();

        match &current_token.html_token {
            HtmlToken::OpenTag { name } => {
                if tag_being_parsed.is_none() {
                    tag_being_parsed = Some(name.clone());
                } else {
                    let new_node = parse_node(html_tokens, current_token_idx, node_being_build_internal_id, all_nodes, styles);
                    children.push(new_node);
                }
            },
            HtmlToken::OpenTagEnd => {
                //Some tags can't have children and therefore also no (self)close tag

                //TODO: did I handle uppercase tags already? (needs to happen in the lexer)
                if tag_being_parsed.is_some() && SELF_CLOSING_TAGS.contains(&tag_being_parsed.as_ref().unwrap().as_str()) {
                    let new_node = DomNode::Element(ElementDomNode {
                        internal_id: node_being_build_internal_id,
                        name: tag_being_parsed,
                        children: Some(children),
                        parent_id,
                    });

                    let rc_node = Rc::new(new_node);
                    all_nodes.insert(node_being_build_internal_id, Rc::clone(&rc_node));
                    return rc_node;
                }
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
                all_nodes.insert(id_of_attr_node, rc_clone_node);

            },
            HtmlToken::CloseTag { name } => {

                if SELF_CLOSING_TAGS.contains(&name.as_str()) {
                    //these tags should never be closed, so we just ignore when that happens anyway
                    *current_token_idx += 1;
                    continue;
                }

                if tag_being_parsed.is_none() || name != tag_being_parsed.as_ref().unwrap() {
                    //TODO: this is a case that can happen in the real world of course, figure out how to handle this...
                    debug_log_warn(format!("We are not closing the tag we opened, something is wrong! ({}) ({}:{})", 
                                           name, current_token.line, current_token.character));
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
            HtmlToken::Text(_) | HtmlToken::Whitespace(_) | HtmlToken::Entity(_) => {
                let parent_for_node = if tag_being_parsed.is_some() { node_being_build_internal_id } else { parent_id };
                let new_node = read_all_text_for_text_node(html_tokens, current_token_idx, parent_for_node);
                let id_for_text_node = new_node.get_internal_id();

                let rc_node = Rc::new(new_node);
                all_nodes.insert(id_for_text_node, Rc::clone(&rc_node));

                if tag_being_parsed.is_some() {
                    children.push(rc_node);
                } else {
                    return rc_node;
                }
            },
            HtmlToken::Comment(_) => {},
            HtmlToken::Doctype(_) => {
                //for now we ignore, eventually we should probably distinguish html5 and other html variants here
            },
            HtmlToken::Style(content) => {
                let style_tokens = css_lexer::lex_css(content, current_token.line, current_token.character);
                styles.append(&mut css_parser::parse_css(&style_tokens));
            },
        }

        *current_token_idx += 1;
    }

    panic!("this should not happen");
}


fn read_all_text_for_text_node(html_tokens: &Vec<HtmlTokenWithLocation>, current_token_idx: &mut usize, parent_id: usize) -> DomNode {
    let mut text_for_node = String::new();
    let mut non_breaking_space_positions: Option<HashSet<usize>> = None;

    'text_token_loop: while *current_token_idx < html_tokens.len() {
        let current_token = html_tokens.get(*current_token_idx).unwrap();

        match &current_token.html_token {
            HtmlToken::Text(text) => {
                text_for_node.push_str(text);
            },
            HtmlToken::Whitespace(_) => {
                text_for_node.push_str(" ");
            },
            HtmlToken::Entity(entity) => {
                match entity.as_str() {
                    "amp" => { text_for_node.push('&'); }
                    "apos" => { text_for_node.push('\'') }
                    "gt" => { text_for_node.push('>'); }
                    "lt" => { text_for_node.push('<'); }
                    "quot" => { text_for_node.push('"') }

                    "nbsp" => {
                        let position = text_for_node.len();
                        non_breaking_space_positions = match non_breaking_space_positions {
                            Some(mut set) => {
                                set.insert(position);
                                Some(set)
                            },
                            None => {
                                let mut set = HashSet::new();
                                set.insert(position);
                                Some(set)
                            },
                        }
                    }

                    _ => {
                        //unknown entity, just use as text
                        text_for_node.push_str(entity);
                    }
                }
            }
            _ => break 'text_token_loop
        }

        *current_token_idx += 1;
    }

    //we now subtract one from the idx, because we break from the above loop because we should not handle that char yet and the main loop will increment it:
    *current_token_idx -= 1;


    return DomNode::Text(TextDomNode {
        internal_id: get_next_dom_node_interal_id(),
        text_content: text_for_node,
        parent_id: parent_id,
        non_breaking_space_positions: non_breaking_space_positions
    });
}
