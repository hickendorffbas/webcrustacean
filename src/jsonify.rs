use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::Peekable;
use std::rc::Rc;
use std::str::CharIndices;


use crate::color::Color;
use crate::dom::{
    DomText,
    ElementDomNode,
    get_next_dom_node_interal_id
};
use crate::layout::{
    CssBox,
    CssTextBox,
    LayoutNode,
    LayoutNodeContent,
};


#[derive(PartialEq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum JsonValue {
    Object(HashMap<String, JsonValue>),
    Int(i32),
    String(String),
    List(Vec<JsonValue>),
    Boolean(bool),
}


pub fn json_is_equal(json1: &String, json2: &String) -> bool {
    let parsed1 = parse_json(json1);
    let parsed2 = parse_json(json2);
    return parsed1 == parsed2;
}


pub fn parse_json(json: &String) -> JsonValue {
    return parse_json_with_state(&mut ParserState::make_for(&json));
}


fn parse_json_with_state(parser_state: &mut ParserState) -> JsonValue {
    //Because this parser is not meant for webcontent, just internal things like tests and debugging, we panic on invalid json for now
    //TODO: this parser does need some basic tests

    parser_state.consume_until_next_relevant_char();
    let peek = parser_state.peek_next_char().unwrap();

    if peek == '{' {
        parser_state.next();
        parser_state.consume_until_next_relevant_char();

        let mut object_map = HashMap::new();
        while parser_state.peek_next_char().is_some() && parser_state.peek_next_char().unwrap() != '}' {

            parser_state.consume_until_next_relevant_char();

            if parser_state.peek_next_char().unwrap() == ',' {
                parser_state.next();
            }
            parser_state.consume_until_next_relevant_char();

            parser_state.next(); //eat the opening quote of the key

            let mut key_buffer = String::new();
            while parser_state.peek_next_char().unwrap() != '"' {
                key_buffer.push(parser_state.next().unwrap());
            }

            parser_state.next(); // eat the closing quote of the key
            parser_state.consume_until_next_relevant_char();
            parser_state.next(); // eat the colon

            let value = parse_json_with_state(parser_state);
            object_map.insert(key_buffer, value);

            parser_state.consume_until_next_relevant_char();
        }
        parser_state.next(); //eat the closing brace

        return JsonValue::Object(object_map);
    }

    if peek == '[' {
        parser_state.next();
        parser_state.consume_until_next_relevant_char();

        let mut values = Vec::new();
        while parser_state.peek_next_char().is_some() && parser_state.peek_next_char().unwrap() != ']' {
            if parser_state.peek_next_char().unwrap() == ',' {
                parser_state.next();
            }

            values.push(parse_json_with_state(parser_state));
            parser_state.consume_until_next_relevant_char();
        }
        parser_state.next(); //eat the closing bracket

        return JsonValue::List(values);
    }

    if peek == '"' {
        parser_state.next(); //eat the opening quote
        return JsonValue::String(parser_state.consume_until('"').unwrap().to_owned());
    }

    let mut buffer = String::new();
    while parser_state.peek_next_char().is_some() {
        let next_char = parser_state.peek_next_char().unwrap();

        if next_char == ',' || next_char == '}' || next_char == ']' {
            if buffer.parse::<i32>().is_ok() {
                return JsonValue::Int(buffer.parse::<i32>().unwrap())
            }

            if buffer == "true" || buffer == "false" {
                return JsonValue::Boolean(buffer == "true")
            }

            panic!("Invalid JSON");
        }

        buffer.push(parser_state.next().unwrap());
    }

    panic!("Invalid JSON");
}


pub fn layout_node_to_json(layout_node: &LayoutNode) -> String {

    let mut buffer = String::new();

    buffer += "{";

    match &layout_node.content {
        LayoutNodeContent::TextLayoutNode(text_layout_node) => {
            buffer += "\"color\":";
            buffer += color_to_json(&text_layout_node.background_color).as_str();

            buffer += ", \"boxes\":";
            buffer += css_text_boxes_to_json(&text_layout_node.css_text_boxes).as_str();
        },
        LayoutNodeContent::AreaLayoutNode(area_layout_node) => {
            buffer += "\"color\":";
            buffer += color_to_json(&area_layout_node.background_color).as_str();

            buffer += ", \"location\":";
            buffer += css_box_to_json(&area_layout_node.css_box).as_str();

            buffer += ", \"childs\":";
            buffer += childs_to_json(&layout_node.children).as_str();
        },
        LayoutNodeContent::ImageLayoutNode(_) => todo!(),  //TODO: implement
        LayoutNodeContent::ButtonLayoutNode(_) => todo!(),  //TODO: implement
        LayoutNodeContent::TextInputLayoutNode(_) => todo!(),  //TODO: implement
        LayoutNodeContent::TableLayoutNode(_) => todo!(),  //TODO: implement
        LayoutNodeContent::TableCellLayoutNode(_) => todo!(),  //TODO: implement
        LayoutNodeContent::NoContent => { },
    }

    buffer += "}";

    return buffer;
}


pub fn color_to_json(color: &Color) -> String {
    let r = color.r;
    let g = color.g;
    let b = color.b;

    return format!("[{r}, {g}, {b}]");   
}


pub fn css_text_boxes_to_json(css_text_boxes: &Vec<CssTextBox>) -> String {
    let mut buffer = String::new();
    buffer.push('[');

    let mut first = true;

    for css_text_box in css_text_boxes {
        if !first {
            buffer.push(',');
        }

        buffer += "{ \"text\": \"";
        buffer += css_text_box.text.as_str();
        buffer += "\", ";

        buffer += "\"position\":";
        buffer += css_box_to_json(&css_text_box.css_box).as_str();

        buffer.push('}');
        first = false;
    }

    buffer.push(']');
    return buffer;
}


fn css_box_to_json(css_box: &CssBox) -> String {
    return format!("[{:.0}, {:.0}, {:.0}, {:.0}]", css_box.x, css_box.y, css_box.width, css_box.height);
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


pub fn dom_node_to_json(node: &Rc<RefCell<ElementDomNode>>, buffer: &mut String) {
    let node = node.borrow();

    buffer.push_str("{\"name\": \"");
    match &node.name {
        Some(node_name) => buffer.push_str(node_name.as_str()),
        None => {},
    }

    buffer.push_str("\", \"text\": \"");
    match &node.text {
        Some(node_text) => buffer.push_str(node_text.text_content.as_str()),
        None => {},
    }

    buffer.push_str("\", \"image\": ");
    buffer.push_str(node.image.is_some().to_string().as_str());

    buffer.push_str(", \"scripts\": ");
    if node.scripts.is_some() {
        buffer.push_str(node.scripts.as_ref().unwrap().len().to_string().as_str());
    } else {
        buffer.push('0');
    }

    buffer.push_str(", \"component\": ");
    buffer.push_str(node.page_component.is_some().to_string().as_str());

    buffer.push_str(", \"attributes:\": [");
    if node.attributes.is_some() {
        for attribute in node.attributes.as_ref().unwrap() {
            buffer.push_str("{\"name\": ");
            buffer.push_str(&attribute.borrow().name);
            buffer.push_str("\", \"value\": \"");
            buffer.push_str(&attribute.borrow().value);
            buffer.push_str("\"}");
        }
    }

    buffer.push_str("], \"children\": [");
    if node.children.is_some() {
        for child in node.children.as_ref().unwrap() {
            dom_node_to_json(child, buffer);
        }
    }

    buffer.push_str("]}");
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

