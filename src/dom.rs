use std::rc::Rc;

//TODO: would probably be nice to have a trait or something to get parent and interal id etc. from all nodes regardless of type

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Document {
    pub document_node: Rc<DomNode>,
    pub all_nodes: Vec<Rc<DomNode>>
}

impl Document {
    #[allow(dead_code)] //TODO: not currently used because we are still working out problems with adding clickboxes to the right level in the layout tree
    pub fn has_parent_with_tag_name(&self, dom_node: &DomNode, tag_name: &str) -> bool {

        fn node_has_tag_name(dom_node: &DomNode, tag_name: &str) -> bool {
            match dom_node {
                DomNode::Element(node) => return node.name.is_some() && node.name.as_ref().unwrap() == tag_name,
                _ => return false,
            }
        }

        fn inner_has_parent_with_tag_name(document: &Document, parent_id: usize, tag_name: &str, dom_node: &DomNode) -> bool {
            return node_has_tag_name(document.all_nodes.get(parent_id).unwrap().as_ref(), tag_name)
                   || document.has_parent_with_tag_name(dom_node, tag_name);
        }

        match dom_node {
            DomNode::Document(_) => return false,
            DomNode::Element(node) => return inner_has_parent_with_tag_name(self, node.parent_id, tag_name, dom_node),
            DomNode::Attribute(node) => return inner_has_parent_with_tag_name(self, node.parent_id, tag_name, dom_node),
            DomNode::Text(node) => return inner_has_parent_with_tag_name(self, node.parent_id, tag_name, dom_node),
        }

    }
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub enum DomNode {
    Document(DocumentDomNode),
    Element(ElementDomNode),
    Attribute(AttributeDomNode),
    Text(TextDomNode),
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
    pub text_content: Option<String>, //TODO: remove the option here, there should always be text in a text node (can be the empty string)
    pub parent_id: usize,
}
