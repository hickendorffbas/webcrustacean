use std::cell::RefCell;
use std::iter::Peekable;
use std::rc::Rc;
use std::str::CharIndices;


use crate::color::Color;
use crate::dom::{
    DomText,
    ElementDomNode,
    get_next_dom_node_interal_id
};
use crate::layout::{LayoutNode, LayoutRect};


//TODO: this function should have some tests by itself
pub fn compare_json(json1: &String, json2: &String) -> bool {
    //compare the strings, but ignore whitespace (when not in quotes)

    let mut in_quotes = false;

    let mut iter1 = json1.chars().peekable();
    let mut iter2 = json2.chars().peekable();

    while iter1.peek().is_some() {

        if !in_quotes {
            while iter1.peek().is_some() && iter1.peek().unwrap().is_whitespace() {
                iter1.next();
            }
            while iter2.peek().is_some() && iter2.peek().unwrap().is_whitespace() {
                iter2.next();
            }
        }
    
        if iter1.peek().is_some() && iter2.peek().is_none() ||
           iter1.peek().is_none() && iter2.peek().is_some() {
                return false;
        }

        let char1 = iter1.next().unwrap();
        let char2 = iter2.next().unwrap();

        if char1 == '"' {
            in_quotes = !in_quotes;
        }

        if char1 != char2 {
            return false;
        }
    }

    //remove any remaning whitespace from json2
    while iter2.peek().is_some() && iter2.peek().unwrap().is_whitespace() {
        iter2.next();
    }

    //if we have nothing left, they were equal
    return iter2.peek().is_none();
}



pub fn layout_node_to_json(layout_node: &LayoutNode) -> String {

    let mut buffer = String::new();

    buffer += "{";

    buffer += "\"color\":";
    buffer += color_to_json(&layout_node.background_color).as_str();

    buffer += ", \"rects\":";
    buffer += rects_to_json(&layout_node.rects).as_str();

    buffer += ", \"childs\":";
    buffer += childs_to_json(&layout_node.children).as_str();

    buffer += "}";


    return buffer;
}


pub fn color_to_json(color: &Color) -> String {
    let r = color.r;
    let g = color.g;
    let b = color.b;

    return format!("[{r}, {g}, {b}]");   
}


pub fn rects_to_json(rects: &Vec<LayoutRect>) -> String {
    let mut buffer = String::new();
    buffer.push('[');

    let mut first = true;

    for rect in rects {
        if !first {
            buffer.push(',');
        }

        buffer += "{ \"text\": ";

        if rect.text_data.is_none() {
            buffer += "null";
        } else {
            buffer += "\"";
            buffer += rect.text_data.as_ref().unwrap().text.as_str();
            buffer += "\"";
        }

        buffer.push('}');
        first = false;
    }

    buffer.push(']');
    return buffer;
}


fn childs_to_json(childs: &Option<Vec<Rc<RefCell<LayoutNode>>>>) -> String {
    let mut buffer = String::new();
    buffer.push('[');

    if childs.is_some() {

        let mut first = true;

        for child in childs.as_ref().unwrap() {
            let our_child = child.borrow();
            let node_json = layout_node_to_json(&our_child);
            
            buffer += node_json.as_str();
            if !first {
                buffer.push(',');
            }
            first = false;
        }

    }

    buffer.push(']');
    return buffer;
}



struct ParserState<'a>  {
    iterator: Peekable<CharIndices<'a>>,
    original_string: String,
    consumed_until_idx: usize,
    irrelevant_chars: [char;3]
}
impl ParserState<'_> {
    fn make_for(text: &String) -> ParserState {
        return ParserState {
            iterator: text.char_indices().peekable().clone(),
            original_string: text.clone(),
            consumed_until_idx: 0,
            irrelevant_chars: [' ', '\n', '\t'], //TODO: we should get this passed in if we make this a generic parser state (not just for json)
        }
    }
    fn next(&mut self) -> Option<char> {
        let optional_next = self.iterator.next();
        if optional_next.is_none() {
            return None;
        }
        let (idx, next_char) = optional_next.unwrap();
        self.consumed_until_idx = idx;
        return Some(next_char);
    }
    fn consume_until<'a>(&'a mut self, until: char) -> Option<&'a str> {
        let start_idx = self.consumed_until_idx + 1;

        loop {
            let next = self.next();
            if next.is_none() {
                return None;
            }
    
            if next.unwrap() == until {
                return Some(&self.original_string[start_idx..self.consumed_until_idx]);
            }
        }
    }
    fn consume_until_next_relevant_char(&mut self) {
        loop {
            let next = self.peek_next_char();
            if next.is_none() {
                return;
            }
    
            if !self.irrelevant_chars.contains(&next.unwrap()) {
                return;
            }
            self.next();
        }
    }
    fn peek_next_char(&mut self) -> Option<char> {
        let peeked = self.iterator.peek();
        if peeked.is_none() {
            return None;
        }
        let (_, peeked) = peeked.unwrap();
        return Some(*peeked);
    }
}


pub fn dom_node_from_json(json_data: &String) -> ElementDomNode {
    let mut parser_state = ParserState::make_for(&json_data);
    return parse_dom_node_from_json(&mut parser_state);
}


fn parse_dom_node_from_json(parser_state: &mut ParserState) -> ElementDomNode {
    let mut dom_node = ElementDomNode::new_empty();
    dom_node.internal_id = get_next_dom_node_interal_id();

    parser_state.consume_until('{');
    parser_state.consume_until_next_relevant_char();

    while parser_state.peek_next_char() != Some('}') {

        parser_state.consume_until('"');
        let key_name = String::from(parser_state.consume_until('"').unwrap());
        parser_state.consume_until(':');

        match key_name.as_str() {

            "text" => {
                parser_state.consume_until('"');
                let text = String::from(parser_state.consume_until('"').unwrap());
                dom_node.text = Some(DomText { text_content: text, non_breaking_space_positions: None });
            },

            _ => { panic!("unrecognized key in json!")},
        }

        parser_state.consume_until_next_relevant_char();
    }

    parser_state.consume_until('}');

    return dom_node;
}

