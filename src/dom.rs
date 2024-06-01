use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use image::DynamicImage;

use crate::network::url::Url;
use crate::resource_loader::{self, ResourceThreadPool, ResourceRequestJobTracker};
use crate::script::js_ast::Script;
use crate::style::StyleContext;


static NEXT_DOM_NODE_INTERNAL: AtomicUsize = AtomicUsize::new(1);
pub fn get_next_dom_node_interal_id() -> usize { NEXT_DOM_NODE_INTERNAL.fetch_add(1, Ordering::Relaxed) }


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Document {
    pub document_node: Rc<RefCell<ElementDomNode>>,
    pub all_nodes: HashMap<usize, Rc<RefCell<ElementDomNode>>>,
    pub style_context: StyleContext,
    pub base_url: Url, //The url this DOM was loaded from
}
impl Document {
    pub fn new_empty() -> Document {
        return Document { document_node: Rc::from(RefCell::from(ElementDomNode::new_empty())),
            all_nodes: HashMap::new(), style_context: StyleContext { user_agent_sheet: vec![], author_sheet: vec![] }, base_url: Url::empty() };
    }
    pub fn update_all_dom_nodes(&mut self, resource_thread_pool: &mut ResourceThreadPool) -> bool {
        //returns whether there are dirty nodes after the update

        return self.document_node.borrow_mut().update(resource_thread_pool, self);
    }
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

    pub dirty: bool,

    pub text: Option<DomText>,
    pub name: Option<String>,
    pub name_for_layout: TagName,

    pub children: Option<Vec<Rc<RefCell<ElementDomNode>>>>,
    pub attributes: Option<Vec<Rc<RefCell<AttributeDomNode>>>>,

    pub image: Option<Rc<DynamicImage>>,
    pub img_job_tracker: Option<ResourceRequestJobTracker<DynamicImage>>,

    pub scripts: Option<Vec<Script>>,
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
    fn update(&mut self, resource_thread_pool: &mut ResourceThreadPool, document: &Document) -> bool {
        //returns whether there are dirty nodes after the update (being itself, or any of the children)

        let mut any_child_dirty = false;

        if self.children.is_some() {
            for child in self.children.as_ref().unwrap() {
                let child_dirty = child.borrow_mut().update(resource_thread_pool, document);
                if child_dirty {
                    any_child_dirty = true;
                }
            }
        }

        if self.image.is_none() && self.name.is_some() && self.name.as_ref().unwrap() == "img" {
            let image_src = self.get_attribute_value("src");

            if image_src.is_some() {
                if self.img_job_tracker.is_none() {
                    let image_url = Url::from_base_url(&image_src.unwrap(), Some(&document.base_url));

                    self.img_job_tracker = Some(resource_loader::schedule_load_image(&image_url, resource_thread_pool)); //TODO: eventually store the threadpool
                                                                                                                         //      on a more general context object

                } else {
                    let try_recv_result = self.img_job_tracker.as_ref().unwrap().receiver.try_recv();
                    if try_recv_result.is_ok() {
                        self.image = Some(Rc::from(try_recv_result.unwrap()));
                        self.dirty = true;
                        self.img_job_tracker = None;
                    }

                }

            } else {
                self.image = Some(Rc::from(resource_loader::fallback_image()));
                self.dirty = true;
            }
        }

        return any_child_dirty || self.dirty;
    }
    pub fn new_empty() -> ElementDomNode {
        return ElementDomNode {
            internal_id: 0,
            parent_id: 0,
            is_document_node: true,
            dirty: false,
            text: None,
            name: None,
            name_for_layout: TagName::Other,
            children: None,
            attributes: None,
            image: None,
            img_job_tracker: None,
            scripts: None,
        };
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
