use std::collections::LinkedList;

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
impl<'document> HtmlNode<'document> { //TODO: why do I need 2 lifetimes here?
    pub fn find_attribute_value(&self, key_to_find: &str) -> Option<Vec<&'document str>> {

        match &self.attributes {
            Some(attrs) => {
                for att in attrs {
                    if att.key == key_to_find {
                        return Some(att.value.clone()); //TODO: can I avoid the clone here? All should be live for the lifetime of the document
                    }
                }
                return None
            },
            None => return None
        }
    }

}


pub fn parse_document(document: &str) -> Vec<HtmlNode> {
    let tokens = tokenize(document);
    return build_html_nodes(&tokens);
}

fn build_html_nodes<'document>(tokens: &Vec<Token<'document>>) -> Vec<HtmlNode<'document>> {
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

                if (!recorded_tag_name) {
                    unparsed_text_nodes.push(token.value);
                }
            }

            TokenType::ForwardSlash => {
                if let Some(last_token) = opt_last_token {
                    if parsing_inside_tag && matches!(last_token.token_type, TokenType::Lt) {
                        parsing_inside_closing_tag = true;
                    }
                }

                if (in_quotes || !parsing_inside_tag) {
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
                    println!("parsed_attributes {:?}", parsed_attributes);

                    parsed_attribute_key = None;
                    unparsed_text_nodes = Vec::new();
                }

            }


            TokenType::Space => {}
            TokenType::Newline => {}
        }

        opt_last_token = Some(&token);
    }


    return nodes;
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
