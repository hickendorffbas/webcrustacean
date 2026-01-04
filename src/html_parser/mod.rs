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
            if token == Token::EOF {
                break;
            }
            self.handle_token(token);
        }
    }

    fn handle_token(&mut self, token: Token) {
        println!("handling: {:?}, {:?}", token, self.mode);
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
                    self.stack.push(root);
                    self.mode = InsertionMode::Parsing;
                } else {
                    //We implicitly generate a root here, and then re-handle the token:
                    let root = Rc::new(RefCell::new(ElementDomNode::new("html".to_owned())));
                    self.document.document_node.borrow_mut().children = Some(vec![root.clone()]);
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
                self.stack.push(root);

                self.mode = InsertionMode::Parsing;
                self.handle_token(Token::Text(text_value));
            },
            _ => {
                todo!(); //TODO: are there any other cases left here?
            },
        }
    }

    fn parse_node(&mut self, token: Token) {
        match token {
            Token::StartTag { name, .. } => {
                let node = ElementDomNode::new(name);
                self.stack.push(Rc::from(RefCell::from(node)));
            },
            Token::EndTag { .. } => {
                if self.stack.len() <= 1 {
                    //This endtag would close the root, we don't allow that, because a document should only have one root
                    return;
                }

                //TODO: we don't check here what tag is popped, so <div> bla bla </b> bla bla </div> will break

                if let Some(node) = self.stack.pop() {
                    let parent_node = self.stack.last().unwrap();
                    let mut parent = parent_node.borrow_mut();

                    if parent.children.is_none() {
                        parent.children = Some(Vec::new())
                    }
                    parent.children.as_mut().unwrap().push(node);
                }
            },
            Token::Text(text) => {
                if let Some(parent_node) = self.stack.last() {
                    let mut parent = parent_node.borrow_mut();
                    let non_breaking_space_positions = None; //TODO: we need to get the entity logic from the old parser still
                    let text_node = ElementDomNode::new_with_text(text, non_breaking_space_positions);

                    if parent.children.is_none() {
                        parent.children = Some(Vec::new());
                    }
                    parent.children.as_mut().unwrap().push(Rc::from(RefCell::from(text_node)));
                } else {
                    todo!(); //TODO: this should be an error, since we already parsed a root
                }
            },
            Token::EOF => {
                while self.stack.len() > 1 {
                    let node = self.stack.pop().unwrap();
                    let parent_node = self.stack.last().unwrap();
                    let mut parent = parent_node.borrow_mut();
                    parent.children.as_mut().unwrap().push(node);
                }
            }
        }
    }

}
