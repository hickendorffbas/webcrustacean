use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::style::StyleContext;


static NEXT_DOM_NODE_INTERNAL: AtomicUsize = AtomicUsize::new(1);
pub fn get_next_dom_node_interal_id() -> usize { NEXT_DOM_NODE_INTERNAL.fetch_add(1, Ordering::Relaxed) }


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Document {
    pub document_node: Rc<ElementDomNode>,
    pub all_nodes: HashMap<usize, Rc<ElementDomNode>>,
    pub style_context: StyleContext,
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct ElementDomNode {
    pub internal_id: usize,
    pub parent_id: usize,

    pub is_document_node: bool,
    pub text: Option<DomText>,
    pub name: Option<String>,
    pub children: Option<Vec<Rc<ElementDomNode>>>,
    pub attributes: Option<Vec<Rc<AttributeDomNode>>>,
}
impl ElementDomNode {
    pub fn get_attribute_value(&self, attribute_name: &str) -> Option<String> {
        if self.attributes.is_some() {
            for att in self.attributes.as_ref().unwrap() {
                if att.name == attribute_name {
                    return Some(att.value.clone());
                }
            }
        }
        return None;
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct AttributeDomNode {
    pub name: String,
    pub value: String,
    pub parent_id: usize,  //TODO: if we don't use this a lot, we might want to remove it and make attributes an HashMap<String, String>
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct DomText {
    pub text_content: String,
    pub non_breaking_space_positions: Option<HashSet<usize>>,
}
