use std::collections::LinkedList;
use std::rc::Rc;

use crate::dom::{AttributeDomNode, Document, DocumentDomNode, DomNode, ElementDomNode, TextDomNode};
use crate::debug::debug_print_html_node;

//TODO: we now have these custom nodes, which are good for partially filling them while parsing the document. I think we should call them
//      "partial"-something, and not HTML specifically, because the html nodes are really just the DOM nodes, but they have less Optional's

#[cfg_attr(debug_assertions, derive(Debug))]
pub enum HtmlNodeType {
    Text,
    Tag
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Attribute<'document> {
    pub key: &'document str,
    pub value: Vec<&'document str>
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct HtmlNode<'document> {
    pub node_type: HtmlNodeType,
    pub text_content: Option<Vec<&'document str>>,
    pub tag_name: Option<String>,
    pub attributes: Option<Vec<Attribute<'document>>>,
    pub children: Option<Vec<HtmlNode<'document>>>
}

pub fn parse_document(document: &str) -> Document {
    let tokens = tokenize(document);
    let html_root_node = build_html_nodes(tokens);
    debug_print_html_node(&html_root_node, "HTML_NODES_AFTER_PARSING");
    return convert_html_nodes_to_dom(html_root_node);
}

fn build_html_nodes<'document>(tokens: Vec<Token<'document>>) -> HtmlNode<'document> {
    let mut nodes: Vec<HtmlNode> = Vec::new();

    let mut open_nodes: LinkedList<HtmlNode> = LinkedList::new();

    let mut parsing_inside_tag = false;
    let mut parsing_inside_closing_tag = false;

    let mut unparsed_text_nodes: Vec<&str> = Vec::new();

    let mut opt_last_token: Option<&Token> = None;
    let mut opt_parsed_tag_name: Option<String> = None;
    let mut parsed_attributes: Vec<Attribute> = Vec::new();

    let mut parsed_attribute_key: Option<&str> = None;
    let mut in_quotes = false;

    for (_idx, token) in tokens.iter().enumerate() {

        match token.token_type {
            TokenType::Lt => {
                parsing_inside_tag = true;

                if !unparsed_text_nodes.is_empty() {
                    let text_nodes = Option::from(unparsed_text_nodes);

                    let new_node = HtmlNode {
                        node_type: HtmlNodeType::Text,
                        text_content: text_nodes,
                        attributes: None,
                        tag_name: None,
                        children: None
                    };
                    let mut optional_last_open_node = open_nodes.iter_mut().last();

                    match optional_last_open_node {
                        Some(ref mut last_open_node) => match last_open_node.children {
                            Some(ref mut children) => children.push(new_node),
                            None => panic!("Trying to push on a node that has no room for children")
                        },
                        None => panic!("ERROR: No open tag to put the text on: {:?}", new_node.text_content)
                    }

                    unparsed_text_nodes = Vec::new();
                }

            }

            TokenType::Gt => {

                if parsing_inside_tag {
                    if parsing_inside_closing_tag {

                        unparsed_text_nodes = Vec::new(); //we assume this was the tag content

                        //TODO: check if the tag that is closed is the one we are popping here, instead of just popping blindly
                        close_node(&mut open_nodes, &mut nodes);

                    } else {

                        if let Some(parsed_tag_name) = opt_parsed_tag_name {
                            let new_node = HtmlNode {
                                node_type: HtmlNodeType::Tag,
                                text_content: None,
                                attributes: Option::from(parsed_attributes),
                                tag_name: Option::from(parsed_tag_name.trim().to_lowercase()),
                                children: Some(Vec::new())
                            };
                            open_nodes.push_back(new_node);
                            opt_parsed_tag_name = None;
                            parsed_attributes = Vec::new();
                        }
        
                        if let Some(last_token) = opt_last_token {
                            if let TokenType::ForwardSlash = last_token.token_type {
                                //This means it is a self closing tag i.e.  <bla />
                                close_node(&mut open_nodes, &mut nodes);
                            }
                        }

                    }
                }

                parsing_inside_tag = false;
                parsing_inside_closing_tag = false;
            }

            TokenType::Text => {
                let mut recorded_tag_name = false;
                if let Some(last_token) = opt_last_token {
                    if parsing_inside_tag && matches!(last_token.token_type, TokenType::Lt) {
                        opt_parsed_tag_name = Option::from(token.value.to_owned());
                        recorded_tag_name = true;
                    }
                }

                if !recorded_tag_name {
                    unparsed_text_nodes.push(token.value);
                }
            }

            TokenType::ForwardSlash => {
                if let Some(last_token) = opt_last_token {
                    if parsing_inside_tag && matches!(last_token.token_type, TokenType::Lt) {
                        parsing_inside_closing_tag = true;
                    }
                }

                if in_quotes || !parsing_inside_tag {
                    unparsed_text_nodes.push("/");
                }
            }

            TokenType::Equals => {
                parsed_attribute_key = unparsed_text_nodes.pop();
            }

            TokenType::DoubleQuote => {
                in_quotes = !in_quotes;

                if !in_quotes && parsed_attribute_key.is_some() {
                    parsed_attributes.push(Attribute {
                        key: parsed_attribute_key.unwrap(),
                        value: unparsed_text_nodes
                    });

                    parsed_attribute_key = None;
                    unparsed_text_nodes = Vec::new();
                }

            }


            TokenType::Space => {}
            TokenType::Newline => {}
        }

        opt_last_token = Some(&token);
    }

    //TODO: this is a bit bad. We assume there is only one top node (i.e. <html>), this might not be true, we probably need to wrap it in some html document node
    //TODO: make this a proper debug-only, off-switchable assert
    if nodes.len() != 1 {
        panic!("Did not get exactly 1 root node after parsing HTML");
    }

    return nodes.remove(0); //removing to take ownership (as opposed to using get())
}

fn convert_html_nodes_to_dom(html_node: HtmlNode) -> Document {
    let mut document_dom_nodes: Vec<Rc<DomNode>> = Vec::new();
    let mut next_node_internal_id: usize = 0;

    let id_of_node_being_built = next_node_internal_id;
    next_node_internal_id += 1;
    let new_node = convert_html_node_to_dom_node(html_node, &mut document_dom_nodes, &mut next_node_internal_id, id_of_node_being_built);
    let rc_for_document_node = Rc::clone(&new_node);
    document_dom_nodes.push(new_node);

    let document_node = DomNode::Document(DocumentDomNode{
        internal_id: id_of_node_being_built,
        children: Some(vec![rc_for_document_node]),
    });

    let rc_document_node = Rc::new(document_node);
    let rc_clone_document_node = Rc::clone(&rc_document_node);
    document_dom_nodes.push(rc_document_node);

    debug_assert!(document_dom_nodes.len() == next_node_internal_id, "Id seting of DOM nodes went wrong");

    return Document {
        document_node: rc_clone_document_node,
        all_nodes: document_dom_nodes,
    };
}

fn convert_html_node_to_dom_node(html_node: HtmlNode, document_dom_nodes: &mut Vec<Rc<DomNode>>, next_node_internal_id: &mut usize, parent_id: usize) -> Rc<DomNode> {
    let new_node = match html_node.node_type {
        HtmlNodeType::Text => {
            let new_node = DomNode::Text(TextDomNode {
                internal_id: *next_node_internal_id,
                text_content: html_node.text_content.map(|s| s.join(" ")).unwrap_or("".to_owned()),
                parent_id,
            });
            *next_node_internal_id += 1;
            new_node
        },
        HtmlNodeType::Tag => {
            let id_of_node_being_built = *next_node_internal_id;
            *next_node_internal_id += 1;

            let mut dom_children: Option<Vec<Rc<DomNode>>> = if html_node.attributes.is_some() || html_node.children.is_some() {
                Some(Vec::new())
            } else {
                None
            };

            if html_node.attributes.is_some() {
                for attr in html_node.attributes.unwrap() {
                    let new_node = Rc::new(DomNode::Attribute(AttributeDomNode {
                        internal_id: *next_node_internal_id,
                        name: attr.key.to_owned(),
                        value: attr.value.join(" "),
                        parent_id: id_of_node_being_built,
                    }));
                    *next_node_internal_id += 1;

                    dom_children.as_mut().unwrap().push(Rc::clone(&new_node));
                    document_dom_nodes.push(new_node);
                }
            }

            if html_node.children.is_some() {
                for child in html_node.children.unwrap() {
                    let new_node = convert_html_node_to_dom_node(child, document_dom_nodes, next_node_internal_id, id_of_node_being_built);
                    dom_children.as_mut().unwrap().push(Rc::clone(&new_node));
                    document_dom_nodes.push(new_node);
                }
            }

            let new_node = DomNode::Element(ElementDomNode {
                internal_id: id_of_node_being_built,
                name: html_node.tag_name,
                children: dom_children,
                parent_id,
            });

            new_node
        },
    };

    return Rc::new(new_node);
}

fn close_node<'document>(open_nodes: &mut LinkedList<HtmlNode<'document>>, nodes: &mut Vec<HtmlNode<'document>>) {
    let node_being_closed = open_nodes.pop_back().expect("popping from empty stack!");
    let mut optional_parent = open_nodes.iter_mut().last();

    match optional_parent {
        Some(ref mut parent) => match parent.children {
            Some(ref mut children) => children.push(node_being_closed),
            None => panic!("Trying to push on a node that has no room for children")
        },
        None => nodes.push(node_being_closed)
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
enum TokenType {
    Gt,
    Lt,
    Newline,
    ForwardSlash,
    Space,
    Text,
    DoubleQuote,
    Equals,
}

#[cfg_attr(debug_assertions, derive(Debug))]
struct Token<'document> {
    token_type: TokenType,
    value: &'document str
}


fn tokenize(text: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut parsed_until_idx = -1;

    for (idx, character) in text.chars().enumerate() {

        match character {
            '<' => { parsed_until_idx = add_token(text, TokenType::Lt, idx, parsed_until_idx, &mut tokens); }
            '>' => { parsed_until_idx = add_token(text, TokenType::Gt, idx, parsed_until_idx, &mut tokens); }
            '/' => { parsed_until_idx = add_token(text, TokenType::ForwardSlash, idx, parsed_until_idx, &mut tokens); }
            '\n' => { parsed_until_idx = add_token(text, TokenType::Newline, idx, parsed_until_idx, &mut tokens); }
            ' ' => { parsed_until_idx = add_token(text, TokenType::Space, idx, parsed_until_idx, &mut tokens); }
            '"' => { parsed_until_idx = add_token(text, TokenType::DoubleQuote, idx, parsed_until_idx, &mut tokens); }
            '=' => { parsed_until_idx = add_token(text, TokenType::Equals, idx, parsed_until_idx, &mut tokens); }
            '\r' => {
                //For now we ignore these on purpose
                flush_unparsed_text_to_token(text, idx, parsed_until_idx, &mut tokens);
                parsed_until_idx = idx as i32;
             }

            _ => ()
        }

    }

    flush_unparsed_text_to_token(text, text.len(), parsed_until_idx, &mut tokens);

    return tokens;
}


fn add_token<'document>(text: &'document str, p_token_type: TokenType, current_idx: usize, parsed_until_idx: i32, tokens: &mut Vec<Token<'document>>) -> i32 {
    flush_unparsed_text_to_token(text, current_idx, parsed_until_idx, tokens);
    tokens.push(
        Token { token_type: p_token_type, value: "" }
    );
    return current_idx as i32; //returns until where we have parsed
}


fn flush_unparsed_text_to_token<'document>(text: &'document str, end_parse_idx: usize, parsed_until_idx: i32, tokens: &mut Vec<Token<'document>>) {
    let start_parse_idx: usize = (parsed_until_idx + 1) as usize;

    if start_parse_idx < end_parse_idx {
        tokens.push(
            Token { token_type: TokenType::Text, value: &text[start_parse_idx..end_parse_idx] }
        );
    }
}
