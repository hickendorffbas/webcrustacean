use std::rc::Rc;

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Document {
    pub document_node: Rc<DomNode>,
    pub all_nodes: Vec<Rc<DomNode>>
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub enum DomNode {
    Document(DocumentDomNode),
    Element(ElementDomNode),
    #[allow(dead_code)] //TODO: remove this once we made it non-dead
    Attribute(AttributeDomNode),
    Text(TextDomNode),
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct DocumentDomNode {
    pub internal_id: u32,
    pub children: Option<Vec<Rc<DomNode>>>
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct ElementDomNode {
    pub internal_id: u32,
    pub name: Option<String>, //TODO: remove the option here, an element should always have a name
    pub children: Option<Vec<Rc<DomNode>>>
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct AttributeDomNode {
    pub internal_id: u32,
    pub name: String,
    pub value: String,
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct TextDomNode {
    pub internal_id: u32,
    pub text_content: Option<String>, //TODO: remove the option here, there should always be text in a text node (can be the empty string)
}

