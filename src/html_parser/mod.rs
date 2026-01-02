use std::cell::RefCell;
use std::rc::Rc;

use crate::dom::{Document, ElementDomNode};
use crate::html_parser::lexer::{Lexer, Token};

mod lexer;
#[cfg(test)] mod tests;


#[derive(Debug, Clone, Copy)]
enum InsertionMode {
    ParsingRootNode,
    Parsing,
}

pub struct Parser {
    lexer: Lexer,
    mode: InsertionMode,
    stack: Vec<Rc<RefCell<ElementDomNode>>>,
    document: Document,
}
impl Parser {
    pub fn new(input: String) -> Self {
        Self {
            lexer: Lexer::new(input),
            mode: InsertionMode::ParsingRootNode,
            stack: Vec::new(),
            document: Document::new_empty(),
        }
    }

    pub fn parse(&mut self) {
        loop {
            let token = self.lexer.next_token();
            println!("token: {:?}", token);
            if token == Token::EOF {
                break;
            }
            self.handle_token(token);
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
            Token::StartTag { name, .. } => {
                let node = ElementDomNode::new(name);
                self.stack.push(Rc::from(RefCell::from(node)));
                self.mode = InsertionMode::Parsing;
            },
            _ => {
                todo!(); //TODO: we probably also need to handle text / data here (if a html file has text before the first tag)
            },
        }
    }

    fn parse_node(&mut self, token: Token) {
        match token {
            Token::StartTag { name, self_closing } => {
                let node = ElementDomNode::new(name);
                self.stack.push(Rc::from(RefCell::from(node)));
            },
            Token::EndTag { .. } => {
                if let Some(node) = self.stack.pop() {
                    if let Some(parent_node) = self.stack.last() {
                        let mut parent = parent_node.borrow_mut();
                        parent.children.as_mut().unwrap().push(node);
                    } else {
                        if self.document.document_node.borrow().children.is_none() {
                            self.document.document_node.borrow_mut().children = Some(Vec::new());
                        }

                        self.document.document_node.borrow_mut().children.as_mut().unwrap().push(node);
                    }
                }
            },
            Token::Text(text) => {
                if let Some(parent_node) = self.stack.last_mut() {
                    let mut parent = parent_node.borrow_mut();
                    let non_breaking_space_positions = None; //TODO: we need to get the entity logic from the old parser still
                    let text_node = ElementDomNode::new_with_text(text, non_breaking_space_positions);

                    if parent.children.is_none() {
                        parent.children = Some(Vec::new());
                    }
                    parent.children.as_mut().unwrap().push(Rc::from(RefCell::from(text_node)));
                }
            },
            Token::EOF => {}
        }
    }

}
