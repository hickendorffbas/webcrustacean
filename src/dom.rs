use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use image::RgbaImage;

use crate::network::url::Url;
use crate::platform::Platform;
use crate::resource_loader::{
    self,
    CookieStore,
    ResourceRequestJobTracker,
    ResourceRequestResult,
    ResourceThreadPool,
};
use crate::script::js_ast::Script;
use crate::style::StyleContext;
use crate::ui_components::{
    Button,
    PageComponent,
    TextField
};


static NEXT_DOM_NODE_INTERNAL_ID: AtomicUsize = AtomicUsize::new(1);
pub fn get_next_dom_node_interal_id() -> usize { NEXT_DOM_NODE_INTERNAL_ID.fetch_add(1, Ordering::Relaxed) }


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
    pub fn update_all_dom_nodes(&mut self, resource_thread_pool: &mut ResourceThreadPool, cookie_store: &CookieStore) -> bool {
        //returns whether there are dirty nodes after the update

        return self.document_node.borrow_mut().update(resource_thread_pool, self, cookie_store);
    }
    pub fn find_parent_with_name(&self, start_node: &ElementDomNode, name_to_match: &str) -> Option<Rc<RefCell<ElementDomNode>>> {
        let mut node_id_to_check = start_node.parent_id;

        while node_id_to_check != 0 {
            let node_to_check = self.all_nodes[&node_id_to_check].clone();

            if node_to_check.borrow().name.is_some() && node_to_check.borrow().name.as_ref().unwrap().as_str() == name_to_match {
                return Some(node_to_check);
            }

            node_id_to_check = node_to_check.borrow().parent_id;
        }

        return None;
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub enum TagName {
    Br,
    Img,
    Input,
    Script,
    Style,
    Table,
    Title,

    Other,
}
impl TagName {
    pub fn from_string(tag_being_parsed: &String) -> TagName {
        return match tag_being_parsed.as_str() {

            "br" => TagName::Br,
            "img" => TagName::Img,
            "input" => TagName::Input,
            "script" => TagName::Script,
            "style" => TagName::Style,
            "table" => TagName::Table,
            "title" => TagName::Title,

            _ => {
                //this is not an error, since we only translate tags that we need to do something for in the layout tree
                TagName::Other
            }
        }
    }
}


#[derive(PartialEq)]
pub enum NavigationAction {
    None,
    Get(Url),
    Post(PostData),
}


//TODO: this should be moved to a network related module (just a seperate thing in network?)
#[derive(PartialEq)]
pub struct PostData {
    pub url: Url,
    pub fields: HashMap<String, String>,
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct ElementDomNode {
    //TODO: we are already getting many optional fiels here again, so we need something similar as in layout nodes. Probably just enum variants
    //      check the DOM spec, there is also types and subtypes defined there. Staying close to that will make the JS implementation easier for DOM manipulation
    //      that might it also make it easier to add methods for specific elements, like submitting a form

    pub internal_id: usize,
    pub parent_id: usize,
    pub is_document_node: bool,

    pub dirty: bool,

    pub text: Option<DomText>,
    pub name: Option<String>,
    pub name_for_layout: TagName,

    pub children: Option<Vec<Rc<RefCell<ElementDomNode>>>>,
    pub attributes: Option<Vec<Rc<RefCell<AttributeDomNode>>>>,

    pub image: Option<Rc<RgbaImage>>,
    pub img_job_tracker: Option<ResourceRequestJobTracker<ResourceRequestResult<RgbaImage>>>,

    pub scripts: Option<Vec<Rc<Script>>>,

    pub page_component: Option<Rc<RefCell<PageComponent>>>,
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
    pub fn post_construct(&mut self, platform: &mut Platform) {
        //here we set things up that don't need to happen every update step, but that we don't want to do during html parsing

        if self.name.is_some() && self.name.as_ref().unwrap() == "input" {

            let mut input_type = self.get_attribute_value("type");
            if input_type.is_none() {
                input_type = Some(String::from("text"));
            }
            let mut input_value = self.get_attribute_value("value");
            if input_value.is_none() {
                input_value = Some(String::from(""));
            }

            match input_type.unwrap().as_str() {
                "text" => {
                    //We create the component at (0,0) with size (1,1), the layout pass will update that to the correct positions and sizes
                    let mut text_field = TextField::new(0.0, 0.0, 21.0, 1.0, false);
                    text_field.set_text(platform, input_value.unwrap());
                    self.page_component = Some(Rc::from(RefCell::from(PageComponent::TextField(text_field))));
                },
                "submit" => {
                    if input_value == Some(String::from("")) {
                        input_value = Some(String::from("Submit"));
                    }

                    //We create the component at (0,0) with size (1,1), the layout pass will update that to the correct positions and sizes
                    let button = Button::new(0.0, 0.0, 1.0, 1.0, input_value.unwrap());
                    self.page_component = Some(Rc::from(RefCell::from(PageComponent::Button(button))));
                },
                _ =>  {
                    //Ignoring other values for now
                }
            }
        }

        if self.children.is_some() {
            for child in self.children.as_ref().unwrap() {
                child.borrow_mut().post_construct(platform);
            }
        }
    }
    fn update(&mut self, resource_thread_pool: &mut ResourceThreadPool, document: &Document, cookie_store: &CookieStore) -> bool {
        //returns whether there are dirty nodes after the update (being itself, or any of the children)

        let mut any_child_dirty = false;

        if self.children.is_some() {
            for child in self.children.as_ref().unwrap() {
                let child_dirty = child.borrow_mut().update(resource_thread_pool, document, cookie_store);
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

                     //TODO: eventually store the threadpool on a more general context object
                    self.img_job_tracker = Some(resource_loader::schedule_load_image(&image_url, &cookie_store, resource_thread_pool));

                } else {
                    let try_recv_result = self.img_job_tracker.as_ref().unwrap().receiver.try_recv();
                    if try_recv_result.is_ok() {
                        match try_recv_result.unwrap() {
                            ResourceRequestResult::NotFound => {
                                //TODO: should I get a fallback image here?
                            },
                            ResourceRequestResult::Success(received_image) => {
                                self.image = Some(Rc::from(received_image.body));
                                self.dirty = true;
                            },
                        }
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

    pub fn click(&self, document: &Document) -> NavigationAction {

        if self.page_component.is_some() {
            self.page_component.as_ref().unwrap().borrow_mut().click();
        }

        let possible_link_parent = document.find_parent_with_name(self, "a");

        if possible_link_parent.is_some() {
            let link_parent = possible_link_parent.unwrap();
            let opt_href = link_parent.borrow().get_attribute_value("href");
            if opt_href.is_some() {
                return NavigationAction::Get(Url::from_base_url(&opt_href.unwrap(), Some(&document.base_url)));
            }
        }

        if self.name.is_some() {
            let name = self.name.as_ref().unwrap();

            if name.as_str() == "input" {
                let input_type = self.get_attribute_value("type");
                if input_type.is_some() && input_type.unwrap().as_str() == "submit" {
                    return self.submit_form(document);
                }
            }
        }

        return NavigationAction::None;
    }

    pub fn submit_form(&self, document: &Document) -> NavigationAction {
        let possible_form_parent = document.find_parent_with_name(self, "form");
        if possible_form_parent.is_some() {

            let mut all_fields = HashMap::new();
            possible_form_parent.as_ref().unwrap().borrow().collect_all_inputs(&mut all_fields);

            let post_url_text = possible_form_parent.unwrap().borrow().get_attribute_value("action");
            if post_url_text.is_some() {
                let postdata = PostData {
                    url: Url::from_base_url(&post_url_text.unwrap(), Some(&document.base_url)),
                    fields: all_fields,
                };

                return NavigationAction::Post(postdata);
            }
        }
        return NavigationAction::None;
    }

    fn collect_all_inputs(&self, fields: &mut HashMap<String, String>) {

        if self.name.is_some() && self.name.as_ref().unwrap().as_str() == "input" && self.page_component.is_some() {

            let input_name = self.get_attribute_value("name");
            if input_name.is_some() { //According to spec, elements without name should not be sent

                let component = self.page_component.as_ref().unwrap().borrow();
                let input_value = match component.deref() {
                    PageComponent::Button(_) => {
                        //TODO: should a non-pressed button also have its value sent? (the key should be sent in any case, but maybe with empty value)
                        String::new()
                    },
                    PageComponent::TextField(text_field) => {
                        text_field.text.clone()
                    },
                };

                fields.insert(input_name.unwrap(), input_value);
            }
        }

        if self.children.is_some() {
            for child in self.children.as_ref().unwrap() {
                child.borrow().collect_all_inputs(fields);
            }
        }
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
            page_component: None,
        };
    }

    pub fn dom_property_display(&self) -> DomPropertyDisplay {
        //TODO: check styles for any setting of the display property, and return if set

        if self.name.is_some() {
            let node_name = self.name.as_ref().unwrap();

            if node_name == "a" ||  //TODO: should we check a static array of str here?
               node_name == "b" ||
               node_name == "br" ||
               node_name == "i" ||
               node_name == "img" ||
               node_name == "span" {
                    return DomPropertyDisplay::Inline;
            }
            return DomPropertyDisplay::Block;

        }
        if self.text.is_some() {
            return DomPropertyDisplay::Inline;
        }
        if self.is_document_node {
            return DomPropertyDisplay::Block;
        }

        panic!("No other cases should exist")
    }

    pub fn mark_all_as_dirty(&mut self) {
        self.dirty = true;

        if self.children.is_some() {
            for child in self.children.as_ref().unwrap() {
                child.borrow_mut().mark_all_as_dirty();
            }
        }
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(PartialEq)]
pub enum DomPropertyDisplay {
    Block,
    Inline,
    #[allow(dead_code)] None,  //TODO: add this case (needs to be parsed from css property)
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct AttributeDomNode {
    pub name: String,
    pub value: String,
    #[allow(dead_code)] pub parent_id: usize,  //TODO: if we really don't use this, we might want to remove it and make attributes an HashMap<String, String>
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct DomText {
    pub text_content: String,
    pub non_breaking_space_positions: Option<HashSet<usize>>,
}


pub fn find_dom_node_for_component(component: &PageComponent, document: &Document) -> Rc<RefCell<ElementDomNode>> {

    for node in document.all_nodes.values() {
        if node.borrow().page_component.is_some() {
            if node.borrow().page_component.as_ref().unwrap().borrow().get_id() == component.get_id() {
                return node.clone();
            }
        }
    }

    //We panic here, since if we have a component, it should be somewhere in the DOM, otherwise we have a bug
    panic!("Component not found");
}
