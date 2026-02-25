use std::cell::RefCell;
use std::rc::Rc;

use crate::dom::{
    AttributeDomNode,
    Document,
    ElementDomNode
};
use crate::html_parser::lexer::{Lexer, Token};
use crate::network::url::Url;
use crate::script::{
    js_console,
    js_interpreter,
    js_lexer,
    js_parser,
};
use crate::style::{css_lexer, css_parser};

mod lexer;
#[cfg(test)] mod tests;


#[derive(Debug, Clone, Copy)]
enum InsertionMode {
    ParsingRootNode,
    Parsing,
}


pub struct HtmlParser {
    lexer: Lexer,
    mode: InsertionMode,
    stack: Vec<Rc<RefCell<ElementDomNode>>>,
    pub document: Document,
    self_closing_top_stack_node: bool,
    last_line_idx: usize,
    last_char_idx: usize,
}
impl HtmlParser {
    pub fn new(input: String, base_url: Url) -> Self {
        Self {
            lexer: Lexer::new(input),
            mode: InsertionMode::ParsingRootNode,
            stack: Vec::new(),
            document: Document::new(base_url),
            self_closing_top_stack_node: false,
            last_line_idx: 0,
            last_char_idx: 0,
        }
    }

    pub fn parse(&mut self) {
        loop {
            let token = self.lexer.next_token();
            if token.token == Token::EOF {
                break;
            }

            self.last_line_idx = token.line;
            self.last_char_idx = token.char;

            self.handle_token(token.token);
        }
    }

    fn handle_token(&mut self, token: Token) {
        match self.mode {
            InsertionMode::ParsingRootNode => self.parse_root_node(token),
            InsertionMode::Parsing => self.parse_node(token),
        }
    }

    fn parse_root_node(&mut self, token: Token) {
        match token {
            Token::StartTag { name, self_closing } => {
                if name == "html" {
                    let root = Rc::new(RefCell::new(ElementDomNode::new(name)));
                    self.document.document_node.borrow_mut().children = Some(vec![root.clone()]);
                    self.document.all_nodes.insert(root.borrow().internal_id, root.clone());
                    self.stack.push(root);
                    self.mode = InsertionMode::Parsing;
                } else {
                    //We implicitly generate a root here, and then re-handle the token:
                    let root = Rc::new(RefCell::new(ElementDomNode::new("html".to_owned())));
                    self.document.document_node.borrow_mut().children = Some(vec![root.clone()]);
                    self.document.all_nodes.insert(root.borrow().internal_id, root.clone());
                    self.stack.push(root);

                    self.mode = InsertionMode::Parsing;
                    self.handle_token(Token::StartTag {name, self_closing });
                }
            },
            Token::Text(text_value) => {
                if text_value.trim().is_empty() {
                    //According to spec, whitespace before any content is just ignored
                    return;
                }

                //Since the document does not start with a an html node, we insert a root and re-handle the token:
                let root = Rc::new(RefCell::new(ElementDomNode::new("html".to_owned())));
                self.document.document_node.borrow_mut().children = Some(vec![root.clone()]);
                self.document.all_nodes.insert(root.borrow().internal_id, root.clone());
                self.stack.push(root);

                self.mode = InsertionMode::Parsing;
                self.handle_token(Token::Text(text_value));
            },
            Token::Doctype { content: _content } => {
                //For now we ignore them, in the future our way of parsing should probably depend on the doctype
                return
            }
            _ => {
                panic!("unkown token in html parser");
            },
        }
    }

    fn parse_node(&mut self, token: Token) {
        match token {
            Token::StartTag { name, self_closing } => {
                if self.self_closing_top_stack_node {
                    self.close_top_node();
                }

                //TODO: some tags (like "li" and "p") have optional endtags, those need to be closed when seeing the next one of those

                let node = ElementDomNode::new(name);
                self.stack.push(Rc::from(RefCell::from(node)));
                self.self_closing_top_stack_node = self_closing;
            },
            Token::EndTag { name } => {
                if self.stack.len() <= 1 {
                    //This endtag would close the root, we don't allow that, because a document should only have one root
                    return;
                }

                if self.self_closing_top_stack_node {
                    self.close_top_node();
                }

                loop {
                    if self.stack.len() <= 2 {  //we check for 2 nodes on the stack, because node 1 is the root, node 2 the one we should close
                        break;
                    }

                    {
                        let top_stack = self.stack.last().unwrap().borrow();
                        if top_stack.name.is_some() && top_stack.name.as_ref().unwrap() == &name {
                            break;
                        }
                    }

                    self.close_top_node();
                }

                self.close_top_node();
            },
            Token::Text(text) => {
                if self.self_closing_top_stack_node {
                    self.close_top_node();
                }

                if let Some(parent_node) = self.stack.last() {
                    let non_breaking_space_positions = None; //TODO: implement (check old parser?)
                    let text_node = Rc::from(RefCell::from(ElementDomNode::new_with_text(text, non_breaking_space_positions)));
                    self.register_node_as_child(parent_node.clone(), text_node);
                } else {
                    todo!(); //TODO: this should be an error, since we already parsed a root
                }
            },
            Token::Attribute { name, value } => {
                let mut parent_node = self.stack.last().unwrap().borrow_mut();
                let mut attribute_node = AttributeDomNode { name, value, parent_id: parent_node.internal_id };

                if parent_node.attributes.is_none() {
                    parent_node.attributes = Some(Vec::new());
                }
                attribute_node.parent_id = parent_node.internal_id;
                parent_node.attributes.as_mut().unwrap().push(Rc::from(RefCell::from(attribute_node)));
            },
            Token::Doctype { content: _ } => {
                todo!(); //TODO: implement
            },
            Token::EOF => {
                while self.stack.len() > 1 {
                    let node = self.stack.pop().unwrap();
                    let parent_node = self.stack.last().unwrap().clone();
                    self.register_node_as_child(parent_node, node);
                }
            },
        }
    }

    fn close_top_node(&mut self) {
        self.self_closing_top_stack_node = false;

        if let Some(node) = self.stack.pop() {

            if node.borrow().name.is_some() {
                if node.borrow().name.as_ref().unwrap().as_str() == "script" {

                    let mut script_type = node.borrow().get_attribute_value("type");
                    if script_type.is_none() {
                        script_type = Some(String::from("text/javascript"));
                    }

                    if script_type.as_ref().unwrap().as_str() != "text/javascript" {
                        js_console::log_js_error(format!("Unsupported script type: {:}", script_type.unwrap()).as_str());
                    } else {

                        //TODO: check if we have seen an src attribute, if so only download and run that, ignore the content

                        let node_borr = node.borrow();
                        let content_node = node_borr.children.as_ref().unwrap()[0].borrow();
                        let content = &content_node.text.as_ref().unwrap().text_content;

                        //TODO: for now we just execute the script, but we need to juse the correct execution context, so the right stuff is shared on the page

                        let js_tokens = js_lexer::lex_js(content, self.last_line_idx as u32, self.last_char_idx as u32);
                        let script = js_parser::parse_js(&js_tokens);

                        let mut interpreter = js_interpreter::JsInterpreter::new();
                        interpreter.run_script(&script);
                    }
                }
                if node.borrow().name.as_ref().unwrap().as_str() == "style" {

                    let node_borr = node.borrow();
                    let content_node = node_borr.children.as_ref().unwrap()[0].borrow();
                    let content = &content_node.text.as_ref().unwrap().text_content;

                    let style_tokens = css_lexer::lex_css(content, self.last_line_idx as u32, self.last_char_idx as u32);
                    let styles = css_parser::parse_css(&style_tokens);

                    self.document.style_context.author_sheet.extend(styles);
                }
            }

            let parent = self.stack.last().unwrap().clone();
            self.register_node_as_child(parent, node);
        }
    }

    fn register_node_as_child(&mut self, parent_node: Rc<RefCell<ElementDomNode>>, node: Rc<RefCell<ElementDomNode>>) {
        let mut parent = parent_node.borrow_mut();
        if parent.children.is_none() {
            parent.children = Some(Vec::new())
        }
        node.borrow_mut().parent_id = parent.internal_id;
        self.document.all_nodes.insert(node.borrow().internal_id, node.clone());
        parent.children.as_mut().unwrap().push(node);
    }
}
