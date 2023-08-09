use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{
    Font,
    FontCache,
    FONT_SIZE,
    SCREEN_WIDTH,
};
use crate::debug::debug_log_warn;
use crate::dom::{Document, DomNode};
use crate::renderer::{Color, get_text_dimension};  //TODO: color should probably not come from the renderer, position probably also not


//The hight of the header of bbrowser, so below this point the actual page is rendered:
static HEADER_HIGHT: f32 = 50.0;


//TODO: I need to understand orderings with atomics a bit better
static NEXT_LAYOUT_NODE_INTERNAL: AtomicUsize = AtomicUsize::new(1);
pub fn get_next_layout_node_interal_id() -> usize { NEXT_LAYOUT_NODE_INTERNAL.fetch_add(1, Ordering::Relaxed) }


pub struct FullLayout {
    pub root_node: Rc<LayoutNode>,
    pub all_nodes: HashMap<usize, Rc<LayoutNode>>,
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct LayoutNode {
    pub internal_id: usize,
    pub text: Option<String>, //eventually we need different kinds of layout nodes, text is just one type
    pub location: RefCell<ComputedLocation>,
    pub display: Display, //TODO: eventually we don't want every css construct as a member on this struct ofcourse
    pub visible: bool,
    pub font_bold: bool,
    pub font_color: Color,
    pub font_size: u16,
    pub optional_link_url: Option<String>,
    pub children: Option<Vec<Rc<LayoutNode>>>,
    pub parent_id: usize,
}
impl LayoutNode {
    pub fn all_childnodes_have_given_display(&self, display: Display) -> bool {
        if self.children.is_none() {
            return true;
        }
        return self.children.as_ref().unwrap().iter().all(|node| node.display == display);
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub enum ComputedLocation {
    NotYetComputed,
    Computed(Rect)
}
impl ComputedLocation {
    pub fn x_y_as_int(&self) -> (u32, u32) {
        //TODO: for now we use this to get pixel values, but we actually should convert units properly somewhere (before the rederer, I guess)
        return match self {
            ComputedLocation::NotYetComputed => panic!("Node has not yet been computed"),
            ComputedLocation::Computed(node) => { (node.x as u32, node.y as u32) },
        }
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(PartialEq)]
pub enum Display { //TODO: this is a CSS property, of which we will have many, we should probably define those somewhere else
    Block,
    Inline
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}


pub fn build_full_layout(document_node: &Document, font_cache: &mut FontCache) -> FullLayout {
    let mut top_level_layout_nodes: Vec<Rc<LayoutNode>> = Vec::new();
    let mut all_nodes: HashMap<usize, Rc<LayoutNode>> = HashMap::new();

    let id_of_node_being_built = get_next_layout_node_interal_id();

    top_level_layout_nodes.append(&mut build_header_nodes(&mut all_nodes, id_of_node_being_built));

    let document_layout_node = build_layout_tree(&document_node.document_node, document_node, font_cache, &mut all_nodes, id_of_node_being_built);
    top_level_layout_nodes.push(document_layout_node);

    let root_node = LayoutNode {
        internal_id: id_of_node_being_built,
        text: None,
        location: RefCell::new(ComputedLocation::NotYetComputed),
        display: Display::Block,
        visible: true,
        font_bold: false, //TODO: this should probably not be a top-level attribute of the layout node, but in text properties or something
        font_color: Color::BLACK, //TODO: this should probably not be a top-level attribute of the layout node, but in text properties or something
        font_size: FONT_SIZE, //TODO: this should probably not be a top-level attribute of the layout node, but in text properties or something
        optional_link_url: None,
        children: Some(top_level_layout_nodes),
        parent_id: id_of_node_being_built,  //this is the top node, so it does not really have a parent, we set it to ourselves
    };

    let (root_width, root_height) = compute_layout(&root_node, font_cache, 0.0, HEADER_HIGHT);
    let root_location = ComputedLocation::Computed(
        Rect { x: 0.0, y: 0.0, width: root_width, height: root_height }  //TODO: the 0.0 here is not correct, because of the header
    );
    root_node.location.replace(root_location);


    let rc_root_node = Rc::new(root_node);
    all_nodes.insert(id_of_node_being_built, Rc::clone(&rc_root_node));

    return FullLayout { root_node: rc_root_node, all_nodes }
}


//TODO: need to find a way to make good tests for this
fn compute_layout(node: &LayoutNode, font_cache: &mut FontCache, top_left_x: f32, top_left_y: f32) -> (f32, f32) {
    if !node.visible {
        return (0.0, 0.0);
    }

    if node.children.is_some() {
        let all_block = node.all_childnodes_have_given_display(Display::Block);
        let all_inline = node.all_childnodes_have_given_display(Display::Inline);

        if all_block {
            return apply_block_layout(node, font_cache, top_left_x, top_left_y);
        }
        if all_inline {
            return apply_inline_layout(node, font_cache, top_left_x, top_left_y);
        }

        //TODO: we still need to implement this somewhere earlier in the process (when building the layout tree)
        panic!("Not all children are either inline or block, earlier in the process this should already have been fixed with anonymous blocks");
    }

    if node.text.is_some() {

        //TODO: ideally I just store the font (a reference!) on the node, so I can compute it in the first pass...
        let own_font = Font::new(node.font_bold, node.font_size);
        let font = font_cache.get_font(&own_font);
        let dimension = get_text_dimension(node.text.as_ref().unwrap(), &font);


        return (dimension.width as f32, dimension.height as f32);

    }

    panic!("A node that has no text and no children should not exist"); //TODO: does not exist _yet_, but something like an image would fit here..
}



fn apply_block_layout(node: &LayoutNode, font_cache: &mut FontCache, top_left_x: f32, top_left_y: f32) -> (f32, f32) {
    let mut cursor_y = top_left_y;
    let mut max_width: f32 = 0.0;

    for child in node.children.as_ref().unwrap() {
        let (child_width, child_height) = compute_layout(child, font_cache, top_left_x, cursor_y);
        cursor_y += child_height;
        max_width = max_width.max(child_width);
    }

    let child_location = ComputedLocation::Computed(
        Rect { x: top_left_x, y: top_left_y, width: max_width, height: cursor_y }
    );
    node.location.replace(child_location);

    return (max_width, cursor_y);
}


fn apply_inline_layout(node: &LayoutNode, font_cache: &mut FontCache, top_left_x: f32, top_left_y: f32) -> (f32, f32) {
    let mut cursor_x = top_left_x;
    let mut cursor_y = top_left_y;
    let mut max_width: f32 = 0.0;
    let mut max_height_of_line: f32 = 0.0;

    for child in node.children.as_ref().unwrap() {
        let (child_width, child_height) = compute_layout(child, font_cache, cursor_x, cursor_y);

        if cursor_x != top_left_x && (cursor_x + child_width) > SCREEN_WIDTH as f32 {
            // we need to wrap the element to the next line
            //TODO: we need to support splitting in the element in two (or more?) rects, for text-wrapping

            cursor_x = top_left_x;
            cursor_y += max_height_of_line;

            let (new_child_width, new_child_height) = compute_layout(child, font_cache, cursor_x, cursor_y);
            cursor_x += new_child_width;

            max_width = max_width.max(cursor_x);
            max_height_of_line = new_child_height;

        } else {
            // we append the items to the current line

            cursor_x += child_width;
            max_width = max_width.max(cursor_x);
            max_height_of_line = max_height_of_line.max(child_height);
        }

    }
    let our_height = cursor_y + max_height_of_line;

    let child_location = ComputedLocation::Computed(
        Rect { x: top_left_x, y: top_left_y, width: max_width, height: our_height }
    );
    node.location.replace(child_location);

    return (max_width, our_height);
}


fn build_layout_tree(main_node: &DomNode, document: &Document, font_cache: &mut FontCache,
                     all_nodes: &mut HashMap<usize, Rc<LayoutNode>>, parent_id: usize) -> Rc<LayoutNode> {
    let mut partial_node_text = None;
    let mut partial_node_font_bold = false;
    let mut partial_node_font_color = Color::BLACK;
    let mut partial_node_font_size = FONT_SIZE;
    let mut partial_node_visible = true;
    let mut partial_node_optional_link_url = None;
    let mut partial_node_display = Display::Block;


    let mut childs_to_recurse_on: &Option<Vec<Rc<DomNode>>> = &None;
    match main_node {
        DomNode::Document(node) => {
            childs_to_recurse_on = &node.children;
        },
        DomNode::Element(node) => {

            match &node.name.as_ref().unwrap()[..] {

                "a" => {
                    partial_node_optional_link_url = node.get_attribute_value("href");
                    partial_node_display = Display::Inline;
                }

                "b" => {
                    partial_node_font_bold = true;
                    partial_node_display = Display::Inline;
                }

                "br" => {
                    partial_node_display = Display::Inline;
                }

                "body" => {
                    //for now this needs the default for all fields
                }

                "div" =>  {
                    //for now this needs the default for all fields
                }

                "h1" => {
                    partial_node_font_bold = true;
                    partial_node_font_size = FONT_SIZE + 12;
                    partial_node_display = Display::Inline;
                }
                "h2" => {
                    partial_node_font_bold = true;
                    partial_node_font_size = FONT_SIZE + 10;
                    partial_node_display = Display::Inline;
                }
                "h3" => {
                    partial_node_font_bold = true;
                    partial_node_font_size = FONT_SIZE + 8;
                    partial_node_display = Display::Inline;
                }
                "h4" => {
                    partial_node_font_bold = true;
                    partial_node_font_size = FONT_SIZE + 6;
                    partial_node_display = Display::Inline;
                }
                "h5" => {
                    partial_node_font_bold = true;
                    partial_node_font_size = FONT_SIZE + 4;
                    partial_node_display = Display::Inline;
                }
                "h6" => {
                    partial_node_font_bold = true;
                    partial_node_font_size = FONT_SIZE + 2;
                    partial_node_display = Display::Inline;
                }

                "head" => {
                    //for now this needs the default for all fields
                }

                "html" => {
                    //for now this needs the default for all fields
                }

                //TODO: this one might not be neccesary any more after we fix our html parser to not try to parse the javascript
                "script" => { partial_node_visible = false; }

                //TODO: eventually we want to do something else with the title (update the window title or so)
                "title" => { partial_node_visible = false; }

                default => {
                    debug_log_warn(format!("unknown tag: {}", default));
                }
            }

            childs_to_recurse_on = &node.children;
        }
        DomNode::Attribute(_) => {
            partial_node_visible = false;

            //TODO: this is a bit weird, we should error on getting to this point (because they don't need seperate layout),
            //      but then we need to make sure we handle them in their parents node
        },
        DomNode::Text(node) => {
            let text = &node.text_content;

            //TODO: I need a font here, which is annoying.
                //TODO: I now also need font sizes, making it even more annoying

            let parent_bold = false;  //TODO: get this from the actual parent node, instead of hardcoding
            let parent_font_size = FONT_SIZE;  //TODO: get this from the actual parent node, instead of hardcoding

            let own_font = Font::new(parent_bold, parent_font_size); //TODO: the font should just live on the layout_node
            let font = font_cache.get_font(&own_font);
            let _dimension = get_text_dimension(&text, &font); //TODO: unused, should move to the pass where we compute the actual sizes of things

            if document.has_element_parent_with_name(main_node, "a") {
                partial_node_font_color = Color::BLUE;
            }

            partial_node_text = Option::Some(text.to_string());
        }

    }

    let id_of_node_being_built = get_next_layout_node_interal_id();

    let new_childeren = if let Some(ref children) = childs_to_recurse_on {
        let mut temp_children = Vec::new();

        for child in children {
            temp_children.push(build_layout_tree(child, document, font_cache, all_nodes, id_of_node_being_built));
        }

        let all_display_types = temp_children.iter().map(|child| &child.display).collect::<Vec<&Display>>();

        //TODO: the double && below seems suspect...
        if all_display_types.contains(&&Display::Inline) && all_display_types.contains(&&Display::Block) {
            let mut temp_children_with_anonymous_blocks = Vec::new();
            let mut temp_buffer_for_inline_children = Vec::new();

            for child in temp_children {
                match child.display {
                    Display::Block => {
                        if (temp_buffer_for_inline_children.len() > 0) {
                            let anonymous_block_node = build_anonymous_block_layout_node(partial_node_visible, id_of_node_being_built, temp_buffer_for_inline_children);
                            all_nodes.insert(anonymous_block_node.internal_id, Rc::clone(&anonymous_block_node)); //TODO: cleaner to do this in the build method next to the create of the struct
                            temp_children_with_anonymous_blocks.push(anonymous_block_node);

                            temp_buffer_for_inline_children = Vec::new();
                        }

                        temp_children_with_anonymous_blocks.push(child);
                    },
                    Display::Inline => { temp_buffer_for_inline_children.push(child); },
                }
            }

            if (temp_buffer_for_inline_children.len() > 0) {
                let anonymous_block_node = build_anonymous_block_layout_node(partial_node_visible, id_of_node_being_built, temp_buffer_for_inline_children);
                all_nodes.insert(anonymous_block_node.internal_id, Rc::clone(&anonymous_block_node)); //TODO: cleaner to do this in the build method next to the create of the struct
                temp_children_with_anonymous_blocks.push(anonymous_block_node);
            }

            Some(temp_children_with_anonymous_blocks)
        } else {
            Some(temp_children)
        }
    } else {
        None
    };

    let new_node = LayoutNode {
        internal_id: id_of_node_being_built,
        text: partial_node_text,
        location: RefCell::new(ComputedLocation::NotYetComputed),
        display: partial_node_display,
        visible: partial_node_visible,
        font_bold: partial_node_font_bold,
        font_color: partial_node_font_color,
        font_size: partial_node_font_size,
        optional_link_url: partial_node_optional_link_url,
        children: new_childeren,
        parent_id,
    };

    let rc_new_node = Rc::new(new_node);
    all_nodes.insert(id_of_node_being_built, Rc::clone(&rc_new_node));

    return rc_new_node;
}


fn build_anonymous_block_layout_node(visible: bool, parent_id: usize, inline_children: Vec<Rc<LayoutNode>>) -> Rc<LayoutNode> {
    let id_of_node_being_built = get_next_layout_node_interal_id();

    let anonymous_node = LayoutNode {
        internal_id: id_of_node_being_built,
        text: None,
        location: RefCell::new(ComputedLocation::NotYetComputed),
        display: Display::Block,
        visible: visible,
        font_bold: false,
        font_color: Color::BLACK, //TODO: these fields belong in some optional font struct.... they make no sense in this case
                                  //TODO: also, this might be wrong if we have children that inhertit the text styling from their parent?
        font_size: FONT_SIZE,
        optional_link_url: None,
        children: Some(inline_children),
        parent_id,
    };

    return Rc::new(anonymous_node);
}



fn build_header_nodes(all_nodes: &mut HashMap<usize, Rc<LayoutNode>>, parent_id: usize) -> Vec<Rc<LayoutNode>> {
    //TODO: eventually we want to not have this in the same node list I think (maybe not even as layout nodes?)
    let mut layout_nodes: Vec<Rc<LayoutNode>> = Vec::new();

    let node_id = get_next_layout_node_interal_id();

    let rc_node = Rc::new(LayoutNode {
        internal_id: node_id,
        text: Option::from(String::from("BBrowser")),
        location: RefCell::new(ComputedLocation::Computed(
            Rect { x: 10.0, y: 10.0, width: 500.0, height: HEADER_HIGHT }, //TODO: width is bogus, but we don't have the font to compute it
        )),
        display: Display::Inline,
        font_bold: true,
        font_color: Color::BLACK,
        font_size: FONT_SIZE,
        optional_link_url: None,
        children: None,
        visible: true,
        parent_id,
    });

    all_nodes.insert(node_id, Rc::clone(&rc_node));

    layout_nodes.push(rc_node);

    return layout_nodes;
}
