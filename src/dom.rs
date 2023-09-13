use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::style::StyleRule;


static NEXT_DOM_NODE_INTERNAL: AtomicUsize = AtomicUsize::new(1);
pub fn get_next_dom_node_interal_id() -> usize { NEXT_DOM_NODE_INTERNAL.fetch_add(1, Ordering::Relaxed) }


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Document {
    pub document_node: Rc<DomNode>,
    pub all_nodes: HashMap<usize, Rc<DomNode>>,
    pub styles: Vec<StyleRule>,
}
impl Document {
    pub fn has_element_parent_with_name(&self, node: &DomNode, element_name: &str) -> bool {
        match node {
            DomNode::Element(node) => {
                if node.name.is_some() && node.name.as_ref().unwrap() == element_name {
                    return true;
                }
            },
            _ => {},
        };

        let parent_id = node.get_parent_id();
        return parent_id.is_some() && self.has_element_parent_with_name(self.all_nodes.get(&parent_id.unwrap()).unwrap(), element_name);
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub enum DomNode {
    Document(DocumentDomNode),
    Element(ElementDomNode),
    Attribute(AttributeDomNode),
    Text(TextDomNode),
}
impl DomNode {
    pub fn get_parent_id(&self) -> Option<usize> {
        match self {
            DomNode::Document(_) => None,
            DomNode::Element(node) => Some(node.parent_id),
            DomNode::Attribute(node) => Some(node.parent_id),
            DomNode::Text(node) => Some(node.parent_id),
        }
    }
    pub fn get_internal_id(&self) -> usize {
        match self {
            DomNode::Document(node) => { node.internal_id },
            DomNode::Element(node) => { node.internal_id },
            DomNode::Attribute(node) => { node.internal_id },
            DomNode::Text(node) => { node.internal_id },
        }
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct DocumentDomNode {
    pub internal_id: usize,
    pub children: Option<Vec<Rc<DomNode>>>
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct ElementDomNode {
    pub internal_id: usize,
    pub name: Option<String>, //TODO: remove the option here, an element should always have a name
    pub children: Option<Vec<Rc<DomNode>>>,
    pub parent_id: usize
}
impl ElementDomNode {
    pub fn get_attribute_value(&self, attribute_name: &str) -> Option<String> {
        if self.children.is_some() {
            for child in self.children.as_ref().unwrap() {
                match child.as_ref() {
                    DomNode::Attribute(attr_node) => {
                        if attr_node.name == attribute_name {
                            return Some(attr_node.value.clone());
                        }
                    },
                    _ => {},
                }
            }
        }
        return None;
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct AttributeDomNode {
    pub internal_id: usize,
    pub name: String,
    pub value: String,
    pub parent_id: usize,
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct TextDomNode {
    pub internal_id: usize,
    pub text_content: String,
    pub parent_id: usize,
    pub non_breaking_space_positions: Option<HashSet<usize>>, //TODO: might be nice to combine this with text_content in a text struct
}
