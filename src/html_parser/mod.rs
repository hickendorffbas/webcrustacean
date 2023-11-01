use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::dom::{
    AttributeDomNode,
    Document,
    ElementDomNode,
    get_next_dom_node_interal_id, DomText, TagName,
};
use crate::html_lexer::{HtmlToken, HtmlTokenWithLocation};
use crate::network::url::Url;
use crate::style::{
    StyleRule,
    StyleContext,
    css_lexer,
    css_parser,
    get_user_agent_style_sheet,
};


#[cfg(test)] mod tests;


const SELF_CLOSING_TAGS: [&str; 6] = ["br", "hr", "img", "input", "link", "meta"];


pub fn parse(html_tokens: Vec<HtmlTokenWithLocation>, main_url: &Url) -> RefCell<Document> {
    let mut all_nodes = HashMap::new();
    let mut document_style_rules = Vec::new();

    let mut document_children = Vec::new();
    let mut current_token_idx = 0;

    let document_node_id = get_next_dom_node_interal_id();

    while current_token_idx < html_tokens.len() {
        let mut tag_stack = Vec::new();
        document_children.push(parse_node(&html_tokens, &mut current_token_idx, document_node_id, main_url, &mut all_nodes,
                                          &mut document_style_rules, &mut tag_stack));
        current_token_idx += 1;
    }

    let mut document_node = ElementDomNode {
        internal_id: document_node_id,
        parent_id: 0,
        is_document_node: true,
        text: None,
        name: None,
        name_for_layout: TagName::Other,
        children: Some(document_children),
        attributes: None,
        image: None,
    };
    document_node.update(main_url);

    let rc_doc_node = Rc::new(document_node);
    let rc_doc_node_clone = Rc::clone(&rc_doc_node);
    all_nodes.insert(document_node_id, rc_doc_node);

    let style_context = StyleContext {
        user_agent_sheet: get_user_agent_style_sheet(),
        author_sheet: document_style_rules,
    };
    return RefCell::new(Document { all_nodes, style_context, document_node: rc_doc_node_clone });
}


fn parse_node(html_tokens: &Vec<HtmlTokenWithLocation>, current_token_idx: &mut usize, parent_id: usize, main_url: &Url,
              all_nodes: &mut HashMap<usize, Rc<ElementDomNode>>, styles: &mut Vec<StyleRule>, tag_stack: &mut Vec<String>) -> Rc<ElementDomNode> {
    let node_being_build_internal_id = get_next_dom_node_interal_id();

    let mut tag_being_parsed = None;
    let mut children = Vec::new();
    let mut attributes = Vec::new();

    while *current_token_idx < html_tokens.len() {
        let current_token = html_tokens.get(*current_token_idx).unwrap();

        match &current_token.html_token {
            HtmlToken::OpenTag { name } => {
                if tag_being_parsed.is_none() {
                    tag_being_parsed = Some(name.clone());
                } else {
                    tag_stack.push(tag_being_parsed.clone().unwrap());
                    let new_node = parse_node(html_tokens, current_token_idx, node_being_build_internal_id, main_url, all_nodes, styles, tag_stack);
                    children.push(new_node);
                }
            },
            HtmlToken::OpenTagEnd => {
                //Some tags can't have children and therefore also no (self)close tag

                //TODO: did I handle uppercase tags already? (needs to happen in the lexer)
                if tag_being_parsed.is_some() && SELF_CLOSING_TAGS.contains(&tag_being_parsed.as_ref().unwrap().as_str()) {
                    let mut new_node = ElementDomNode {
                        internal_id: node_being_build_internal_id,
                        name_for_layout: TagName::from_string(&tag_being_parsed.as_ref().unwrap()),
                        name: tag_being_parsed,
                        children: Some(children),
                        parent_id,
                        text: None,
                        attributes: Some(attributes),
                        is_document_node: false,
                        image: None,
                    };
                    new_node.update(main_url);

                    let rc_node = Rc::new(new_node);
                    all_nodes.insert(node_being_build_internal_id, Rc::clone(&rc_node));
                    return rc_node;
                }
            },
            HtmlToken::Attribute(token) => {
                let attribute = AttributeDomNode {
                    name: token.name.clone(),
                    value: token.value.clone(),
                    parent_id: node_being_build_internal_id,
                };
                attributes.push(Rc::new(attribute));

            },
            HtmlToken::CloseTag { name } => {

                if SELF_CLOSING_TAGS.contains(&name.as_str()) {
                    //these tags should never be closed, so we just ignore when that happens anyway
                    *current_token_idx += 1;
                    continue;
                }

                let mut tag_to_close = tag_being_parsed.clone();

                if tag_being_parsed.is_none() || name != tag_being_parsed.as_ref().unwrap() {

                    //TODO is tag_being_parsed the same as the last item in tag_stack? Do I need both?

                    if tag_stack.contains(&name) {
                        //we are trying to close a tag we know about, but not the one we are parsing now, so we close that one instead
                        //and then we are setting the current token one back, so we will retry closing this tag one recursion level higher.
                        tag_to_close = Some(tag_being_parsed.unwrap().clone());
                        *current_token_idx -= 1;
                    } else {
                        //we are closing a tag whe have never opened, we should ignore it
                        *current_token_idx += 1;
                        continue;
                    }

                }

                let mut new_node = ElementDomNode {
                    internal_id: node_being_build_internal_id,
                    name_for_layout: TagName::from_string(&tag_to_close.as_ref().unwrap()),
                    name: tag_to_close,
                    children: Some(children),
                    parent_id,
                    text: None,
                    attributes: Some(attributes),
                    is_document_node: false,
                    image: None,
                };
                new_node.update(main_url);

                let rc_node = Rc::new(new_node);
                all_nodes.insert(node_being_build_internal_id, Rc::clone(&rc_node));
                return rc_node;
            },
            HtmlToken::Text(_) | HtmlToken::Whitespace(_) | HtmlToken::Entity(_) => {
                let parent_for_node = if tag_being_parsed.is_some() { node_being_build_internal_id } else { parent_id };
                let text_node = read_all_text_for_text_node(html_tokens, current_token_idx, parent_for_node, main_url);

                if tag_being_parsed.is_some() {
                    children.push(Rc::new(text_node));
                } else {
                    return Rc::new(text_node);
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
            HtmlToken::Script(_) => {
                //for now we ignore this
            },
        }

        *current_token_idx += 1;
    }

    if tag_being_parsed.is_some() {
        let mut new_node = ElementDomNode { //TODO: I probably want a ::new() function, because I'm going to have a lot of fields that
                                            //      are constructed on :update()
            internal_id: node_being_build_internal_id,
            name_for_layout: TagName::from_string(&tag_being_parsed.as_ref().unwrap()),
            name: tag_being_parsed,
            children: Some(children),
            parent_id,
            text: None,
            attributes: Some(attributes),
            is_document_node: false,
            image: None,
        };
        new_node.update(main_url);

        let rc_node = Rc::new(new_node);
        all_nodes.insert(node_being_build_internal_id, Rc::clone(&rc_node));

        return rc_node;
    }

    panic!("this should not happen (leaving the parse loop without returning)");
}


fn read_all_text_for_text_node(html_tokens: &Vec<HtmlTokenWithLocation>, current_token_idx: &mut usize, parent_id: usize, main_url: &Url) -> ElementDomNode {
    let mut text_content = String::new();
    let mut non_breaking_space_positions: Option<HashSet<usize>> = None;

    'text_token_loop: while *current_token_idx < html_tokens.len() {
        let current_token = html_tokens.get(*current_token_idx).unwrap();

        match &current_token.html_token {
            HtmlToken::Text(text) => {
                text_content.push_str(text);
            },
            HtmlToken::Whitespace(_) => {
                text_content.push_str(" ");
            },
            HtmlToken::Entity(entity) => {
                match entity.as_str() {
                    "amp" => { text_content.push('&'); }
                    "apos" => { text_content.push('\'') }
                    "gt" => { text_content.push('>'); }
                    "lt" => { text_content.push('<'); }
                    "quot" => { text_content.push('"') }

                    "nbsp" => {
                        let position = text_content.len();
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
                        text_content.push_str(entity);
                    }
                }
            }
            _ => break 'text_token_loop
        }

        *current_token_idx += 1;
    }

    //we now subtract one from the idx, because we break from the above loop because we should not handle that char yet and the main loop will increment it:
    *current_token_idx -= 1;

    let dom_text = DomText { text_content, non_breaking_space_positions };

    let mut node = ElementDomNode {
        internal_id: get_next_dom_node_interal_id(),
        parent_id: parent_id,
        text: Some(dom_text),
        name: None,
        name_for_layout: TagName::Other,
        children: None,
        attributes: None,
        is_document_node: false,
        image: None,
    };
    node.update(main_url);
    return node;
}
