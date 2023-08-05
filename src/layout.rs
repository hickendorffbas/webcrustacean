use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
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
    pub visible: bool,
    pub font_bold: bool,
    pub font_color: Color,
    pub font_size: u16,
    pub optional_link_url: Option<String>,
    pub children: Option<Vec<Rc<LayoutNode>>>,
    pub parent_id: usize,
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
        visible: true,
        font_bold: false, //TODO: this should probably not be a top-level attribute of the layout node, but in text properties or something
        font_color: Color::BLACK, //TODO: this should probably not be a top-level attribute of the layout node, but in text properties or something
        font_size: FONT_SIZE, //TODO: this should probably not be a top-level attribute of the layout node, but in text properties or something
        optional_link_url: None,
        children: Some(top_level_layout_nodes),
        parent_id: id_of_node_being_built,  //this is the top node, so it does not really have a parent, we set it to ourselves
    };

    let (root_width, root_height) = compute_layout(&root_node, font_cache); //TODO: this now no longer takes the HEADER into account, should be moved down
    let root_location = ComputedLocation::Computed(
        Rect { x: 0.0, y: 0.0, width: root_width, height: root_height }  //TODO: the 0.0 here is not correct, because of the header
    );
    root_node.location.replace(root_location);


    let rc_root_node = Rc::new(root_node);
    all_nodes.insert(id_of_node_being_built, Rc::clone(&rc_root_node));

    return FullLayout { root_node: rc_root_node, all_nodes }
}


fn move_node(node: &LayoutNode, x_offset: f32, y_offset: f32) {
    if !node.visible {
        return; //TODO: ideally I would not need this, I guess an invisible node can still have a position....
    }

    let (computed_x, computed_y, computed_width, computed_height) = {
        // we start a new scope so the borrow of the node location goes out of scope before we update it

        let borrowed_node = node.location.borrow();
        let node_loc = &*borrowed_node.deref();  //TODO: why why why why why why why why do I need &* here?

        let computed_location = match node_loc {  //TODO: waarom werkt mijn enum_as_variant marco hier niet?
            ComputedLocation::NotYetComputed => { panic!("Node should have been computed by now"); },
            ComputedLocation::Computed(node) => { node },
        };

        (computed_location.x, computed_location.y, computed_location.width, computed_location.height)
    };

    let new_node_location = ComputedLocation::Computed(
        Rect {
            x: computed_x + x_offset,
            y: computed_y + y_offset,
            width: computed_width,
            height: computed_height
        }
    );
    node.location.replace(new_node_location);

    move_children(node, x_offset, y_offset);
}


fn move_children(node: &LayoutNode, x_offset: f32, y_offset: f32) {
    if !node.visible {
        return; //TODO: ideally I would not need this, I guess an invisible node can still have a position....
    }

    if node.children.is_some() {
        for child in node.children.as_ref().unwrap() {
            move_node(child, x_offset, y_offset);
        }
    }
}


// This function does the layout for everything within the node, and sets the location of everything within to the correct position assuming
// that the node itself is at 0,0 , in other words positions relative to node.
//TODO: need to find a way to make good tests for this
fn compute_layout(node: &LayoutNode, font_cache: &mut FontCache) -> (f32, f32) {
    if !node.visible {
        return (0.0, 0.0);
    }

    if node.children.is_some() {
        let mut cursor_x = 0.0;
        let mut cursor_y = 0.0;

        let mut max_row_height_so_far = 0.0;

        let mut max_x_seen = 0.0;
        let mut max_y_seen = 0.0;

        for child in node.children.as_ref().unwrap() {

            let (child_width, child_height) = compute_layout((*child).as_ref(), font_cache);

            //TODO: this currenty does not work because this node might be moved later, and then cross the SCREEN_WIDTH boundary
            if child_width + cursor_x > SCREEN_WIDTH as f32 {
                if cursor_x != 0.0 {
                    cursor_x = 0.0;
                    cursor_y += max_row_height_so_far;
                    max_row_height_so_far = 0.0;
                } else {
                    // it does not fit, but we are all the way to the left already, so going to a new row does not help
                }
            }

            if child_height > max_row_height_so_far {
                max_row_height_so_far = child_height;
            }

            let child_location = ComputedLocation::Computed(
                Rect { x: cursor_x, y: cursor_y, width: child_width, height: child_height }
            );
            child.location.replace(child_location);
            cursor_x += child_width;
            move_children(child, cursor_x, cursor_y);

            if max_x_seen < cursor_x {
                //note: child width is already included in cursor_x
                max_x_seen = cursor_x;
            }
            if max_y_seen < cursor_y + child_height {
                max_y_seen = cursor_y;
            }
        }

        return (max_x_seen, max_y_seen);
    } else {

        if node.text.is_some() {

            //TODO: ideally I just store the font (a reference!) on the node, so I can compute it in the first pass...
            let own_font = Font::new(node.font_bold, node.font_size);
            let font = font_cache.get_font(&own_font);
            let dimension = get_text_dimension(node.text.as_ref().unwrap(), &font);


            return (dimension.width as f32, dimension.height as f32)

        } else {
            panic!("A node that has no text and no children should not exist"); //TODO: does not exist _yet_, but something like an image would fit here..
        }
    }

}


fn build_layout_tree(main_node: &DomNode, document: &Document, font_cache: &mut FontCache,
                     all_nodes: &mut HashMap<usize, Rc<LayoutNode>>, parent_id: usize) -> Rc<LayoutNode> {
    let mut partial_node_text = None;
    let mut partial_node_font_bold = false;
    let mut partial_node_font_color = Color::BLACK;
    let mut partial_node_font_size = FONT_SIZE;
    let mut partial_node_visible = true;
    let mut partial_node_optional_link_url = None;


    let mut childs_to_recurse_on: &Option<Vec<Rc<DomNode>>> = &None;
    match main_node {
        DomNode::Document(node) => {
            childs_to_recurse_on = &node.children;
        },
        DomNode::Element(node) => {

            match &node.name.as_ref().unwrap()[..] {

                "a" => { partial_node_optional_link_url = node.get_attribute_value("href"); }

                "b" => { partial_node_font_bold = true; }

                "br" => {
                    //TODO: I'm moving the actual positioning to a seperate pass after this one, does anything need to be happening here then?
                }

                "h1" => {
                    partial_node_font_bold = true;
                    partial_node_font_size = FONT_SIZE + 12;
                }
                "h2" => {
                    partial_node_font_bold = true;
                    partial_node_font_size = FONT_SIZE + 10;
                }
                "h3" => {
                    partial_node_font_bold = true;
                    partial_node_font_size = FONT_SIZE + 8;
                }
                "h4" => {
                    partial_node_font_bold = true;
                    partial_node_font_size = FONT_SIZE + 6;
                }
                "h5" => {
                    partial_node_font_bold = true;
                    partial_node_font_size = FONT_SIZE + 4;
                }
                "h6" => {
                    partial_node_font_bold = true;
                    partial_node_font_size = FONT_SIZE + 2;
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
        let mut temp_childeren = Vec::new();

        for child in children {
            temp_childeren.push(build_layout_tree(child, document, font_cache, all_nodes, id_of_node_being_built));
        }

        Some(temp_childeren)
    } else {
        None
    };

    let new_node = LayoutNode {
        internal_id: id_of_node_being_built,
        text: partial_node_text,
        location: RefCell::new(ComputedLocation::NotYetComputed),
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
