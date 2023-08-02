use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{
    Font,
    FontCache,
    FONT_SIZE,
    HORIZONTAL_ELEMENT_SPACING,
    LAYOUT_MARGIN_HORIZONTAL,
    SCREEN_WIDTH,
    VERTICAL_ELEMENT_SPACING
};
use crate::debug::debug_log_warn;
use crate::dom::{Document, DomNode};
use crate::renderer::{Color, Position, get_text_dimension};  //TODO: color should probably not come from the renderer, position probably also not


//TODO: I need to understand orderings with atomics a bit better
static NEXT_LAYOUT_NODE_INTERNAL: AtomicUsize = AtomicUsize::new(1);
pub fn get_next_layout_node_interal_id() -> usize { NEXT_LAYOUT_NODE_INTERNAL.fetch_add(1, Ordering::Relaxed) }


pub struct FullLayout {
    pub root_node: Rc<LayoutNode>,
    pub all_nodes: HashMap<usize, Rc<LayoutNode>>,
}


pub struct LayoutNode {
    pub internal_id: usize,
    pub text: Option<String>, //eventually we need different kinds of layout nodes, text is just one type
    pub position: Position,
    pub visible: bool,
    pub font_bold: bool,
    pub font_color: Color,
    pub font_size: u16,
    pub optional_link_url: Option<String>,
    pub children: Option<Vec<Rc<LayoutNode>>>,
    pub parent_id: usize,
}


pub fn build_full_layout(document_node: &Document, font_cache: &mut FontCache) -> FullLayout {
    let mut top_level_layout_nodes: Vec<Rc<LayoutNode>> = Vec::new();
    let mut next_position = Position {x: 10, y: 10};
    let mut all_nodes: HashMap<usize, Rc<LayoutNode>> = HashMap::new();

    let id_of_node_being_built = get_next_layout_node_interal_id();

    top_level_layout_nodes.append(&mut build_header_nodes(&mut next_position, &mut all_nodes, id_of_node_being_built));

    let document_layout_node = layout_dom_tree(&document_node.document_node, document_node, &mut next_position, font_cache,
                                               &mut all_nodes, id_of_node_being_built);
    top_level_layout_nodes.push(document_layout_node);

    let root_node = LayoutNode {
        internal_id: id_of_node_being_built,
        text: None,
        position: Position { x: 0, y: 0 }, //TODO: we need width and hight eventually on this as well (probably as big as the viewport?)
        visible: true,
        font_bold: false, //TODO: this should probably not be a top-level attribute of the layout node, but in text properties or something
        font_color: Color::BLACK, //TODO: this should probably not be a top-level attribute of the layout node, but in text properties or something
        font_size: FONT_SIZE, //TODO: this should probably not be a top-level attribute of the layout node, but in text properties or something
        optional_link_url: None,
        children: Some(top_level_layout_nodes),
        parent_id: id_of_node_being_built,  //this is the top node, so it does not really have a parent, we set it to ourselves
    };

    let rc_root_node = Rc::new(root_node);
    all_nodes.insert(id_of_node_being_built, Rc::clone(&rc_root_node));

    return FullLayout { root_node: rc_root_node, all_nodes }
}


fn layout_dom_tree(main_node: &DomNode, document: &Document, next_position: &mut Position, font_cache: &mut FontCache,
                   all_nodes: &mut HashMap<usize, Rc<LayoutNode>>, parent_id: usize) -> Rc<LayoutNode> {
    let mut move_to_next_line_after = false;

    let mut partial_node_text = None;
    let mut partial_node_position = next_position.clone();
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

                "br" => { move_to_next_line(next_position); }

                "h1" => {
                    partial_node_font_bold = true;
                    partial_node_font_size = FONT_SIZE + 12;
                    move_to_next_line(next_position);
                    move_to_next_line_after = true;
                }
                "h2" => {
                    partial_node_font_bold = true;
                    partial_node_font_size = FONT_SIZE + 10;
                    move_to_next_line(next_position);
                    move_to_next_line_after = true;
                }
                "h3" => {
                    partial_node_font_bold = true;
                    partial_node_font_size = FONT_SIZE + 8;
                    move_to_next_line(next_position);
                    move_to_next_line_after = true;
                }
                "h4" => {
                    partial_node_font_bold = true;
                    partial_node_font_size = FONT_SIZE + 6;
                    move_to_next_line(next_position);
                    move_to_next_line_after = true;
                }
                "h5" => {
                    partial_node_font_bold = true;
                    partial_node_font_size = FONT_SIZE + 4;
                    move_to_next_line(next_position);
                    move_to_next_line_after = true;
                }
                "h6" => {
                    partial_node_font_bold = true;
                    partial_node_font_size = FONT_SIZE + 2;
                    move_to_next_line(next_position);
                    move_to_next_line_after = true;
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
            let dimension = get_text_dimension(&text, &font);

            if document.has_element_parent_with_name(main_node, "a") {
                partial_node_font_color = Color::BLUE;
            }

            if next_position.x + dimension.width > SCREEN_WIDTH - LAYOUT_MARGIN_HORIZONTAL {
                move_to_next_line(next_position);
            }

            partial_node_text = Option::Some(text.to_string());
            partial_node_position = next_position.clone();

            //TODO: this does not account for the height. We should track the max height, and add that when we move to the next line
            move_next_position_by_x(next_position, dimension.width);
        }

    }

    let id_of_node_being_built = get_next_layout_node_interal_id();

    let new_childeren = if let Some(ref children) = childs_to_recurse_on {
        let mut temp_childeren = Vec::new();

        for child in children {
            temp_childeren.push(layout_dom_tree(child, document, next_position, font_cache, all_nodes, id_of_node_being_built));
        }

        Some(temp_childeren)
    } else {
        None
    };

    let new_node = LayoutNode {
        internal_id: id_of_node_being_built,
        text: partial_node_text,
        position: partial_node_position, //TODO: this is not correct, it should be dependent on children as well
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

    if move_to_next_line_after {
        move_to_next_line(next_position);
    }

    return rc_new_node;
}


fn build_header_nodes(position: &mut Position, all_nodes: &mut HashMap<usize, Rc<LayoutNode>>, parent_id: usize) -> Vec<Rc<LayoutNode>> {
    //TODO: eventually we want to not have this in the same node list I think (maybe not even as layout nodes?)
    let mut layout_nodes: Vec<Rc<LayoutNode>> = Vec::new();

    let node_id = get_next_layout_node_interal_id();

    let rc_node = Rc::new(LayoutNode {
        internal_id: node_id,
        text: Option::from(String::from("BBrowser")),
        position: position.clone(),
        font_bold: true,
        font_color: Color::BLACK,
        font_size: FONT_SIZE,
        optional_link_url: None,
        children: None,
        visible: true,
        parent_id,
    });
    position.y += 50;

    all_nodes.insert(node_id, Rc::clone(&rc_node));

    layout_nodes.push(rc_node);

    return layout_nodes;
}


fn move_next_position_by_x(next_position: &mut Position, move_amount : u32) {
    if next_position.x + move_amount < SCREEN_WIDTH {
        next_position.x += move_amount + HORIZONTAL_ELEMENT_SPACING;
    } else {
        move_to_next_line(next_position);
    }
}


fn move_to_next_line(next_position: &mut Position) {
    next_position.x = LAYOUT_MARGIN_HORIZONTAL;
    next_position.y += VERTICAL_ELEMENT_SPACING + 30; //TODO: the +30 here is just because we don't track the max height of previous line here
}
