use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use image::DynamicImage;

use crate::network::url::Url;
use crate::resource_loader;
use crate::style::StyleContext;


static NEXT_DOM_NODE_INTERNAL: AtomicUsize = AtomicUsize::new(1);
pub fn get_next_dom_node_interal_id() -> usize { NEXT_DOM_NODE_INTERNAL.fetch_add(1, Ordering::Relaxed) }


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Document {
    pub document_node: Rc<RefCell<ElementDomNode>>,
    pub all_nodes: HashMap<usize, Rc<RefCell<ElementDomNode>>>,
    pub style_context: StyleContext,
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub enum TagName {
    A,
    B,
    Br,
    Img,
    Script,
    Style,
    Title,

    Other,
}
impl TagName {
    pub fn from_string(tag_being_parsed: &String) -> TagName {
        return match tag_being_parsed.as_str() {

            "a" => TagName::A,
            "b" => TagName::B,
            "br" => TagName::Br,
            "img" => TagName::Img,
            "script" => TagName::Script,
            "style" => TagName::Style,
            "title" => TagName::Title,

            _ => {
                //this is not an error, since we only translate tags that we need to do something for in the layout tree
                TagName::Other
            }
        }
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct ElementDomNode {
    pub internal_id: usize,
    pub parent_id: usize,

    pub is_document_node: bool,
    pub text: Option<DomText>,
    pub name: Option<String>,
    pub name_for_layout: TagName,
    pub children: Option<Vec<Rc<RefCell<ElementDomNode>>>>,
    pub attributes: Option<Vec<Rc<RefCell<AttributeDomNode>>>>,

    pub image: Option<Rc<DynamicImage>>,
}
impl ElementDomNode {
    pub fn get_attribute_value(&self, attribute_name: &str) -> Option<String> {
        if self.attributes.is_some() {
            for att in self.attributes.as_ref().unwrap() {
                if att.borrow().name == attribute_name {
                    return Some(att.borrow().value.clone());
                }
            }
        }
        return None;
    }
    pub fn update(&mut self, main_url: &Url) {

        if self.image.is_none() && self.name.is_some() && self.name.as_ref().unwrap() == "img" {
            let image_src = self.get_attribute_value("src");
            if image_src.is_some() {
                let image_url = Url::from_base_url(&image_src.unwrap(), Some(main_url));
                self.image = Some(Rc::from(resource_loader::load_image(&image_url)));
            } else {
                self.image = Some(Rc::from(resource_loader::fallback_image()));
            }
        }
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
