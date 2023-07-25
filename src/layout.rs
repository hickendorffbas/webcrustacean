use std::rc::Rc;

use crate::{
    Font,
    FontCache,
    FONT_SIZE,
    HORIZONTAL_ELEMENT_SPACING,
    LAYOUT_MARGIN_HORIZONTAL,
    SCREEN_WIDTH,
    VERTICAL_ELEMENT_SPACING
};
use crate::debug::debug_print_dom_tree;
use crate::dom::{Document, DomNode};
use crate::renderer::{get_text_dimension, Position};


pub struct LayoutNode {
    pub text: Option<String>, //eventually we need different kinds of layout nodes, text is just one type
    pub position: Position,
    pub bold: bool,
    pub font_size: u16,
    pub optional_link_url: Option<String>, //TODO: this is a stupid hack because layout nodes don't remember what DOM node they are built from,
                                           //      we should store that on them somehow, but can't get it working ownershipwise currently
    pub children: Option<Vec<LayoutNode>>
}

pub struct LayoutState {
    //TODO: I should learn how real browsers are doing this, are they keeping this state as well though the whole layout tree building?
    // I (probably) have to build this every time we process an HtmlNode, that is probably slow....

    //TODO: maybe this should become a stack like structure, push a value on when a value change (per attribute)
        // you would need to know when to pop then, maybe by referencing an id on the htmlnode?

    pub bold: bool,
    pub font_size: u16,
    pub visible: bool
}

pub fn build_layout_tree(document_node: &Document, font_cache: &mut FontCache) -> LayoutNode {
    //TODO: I really don't want to accept the different font here of course, in any case these should be my own Font objects, but also looked up differently
    let mut layout_nodes: Vec<LayoutNode> = Vec::new();
    let mut next_position = Position {x: 10, y: 10};

    debug_print_dom_tree(&document_node, "START_OF_BUILD_LAYOUT_LIST");

    layout_nodes.append(&mut build_header_nodes(&mut next_position));

    let layout_state = LayoutState {bold : false, font_size: FONT_SIZE, visible: true};
    layout_nodes.append(&mut layout_children(&document_node.document_node, document_node, &mut next_position, &layout_state, font_cache)); //TODO: understand the &mut in the argument better!

    return LayoutNode {  //this is the root layout node
        text: None,
        position: Position { x: 0, y: 0 }, //TODO: we need width and hight eventually on this as
        bold: false, //TODO: this should probably not be a top-level attribute of the layout node, but in text properties or something
        font_size: FONT_SIZE, //TODO: this should probably not be a top-level attribute of the layout node, but in text properties or something
        optional_link_url: None,
        children: Some(layout_nodes),
    }

}

fn layout_children(main_node: &DomNode, document: &Document, next_position: &mut Position, layout_state: &LayoutState, font_cache: &mut FontCache) -> Vec<LayoutNode> {
    let mut layout_nodes: Vec<LayoutNode> = Vec::new();

    let new_layout_state : LayoutState;
    let mut move_to_next_line_after = false;

    let mut childs_to_recurse_on: &Option<Vec<Rc<DomNode>>> = &None;
    match main_node {
        DomNode::Document(node) => {
            new_layout_state = LayoutState {..*layout_state}; //TODO: this is a lot of needless copying
            childs_to_recurse_on = &node.children;
        },
        DomNode::Element(node) => {

            match &node.name.clone().unwrap()[..] { //TODO: understand why I need to clone here
                "b" => { new_layout_state = LayoutState {bold: true, ..*layout_state}; }

                "br" => {
                    move_to_next_line(next_position);
                    new_layout_state = LayoutState {..*layout_state}; //TODO: this is a lot of needless copying
                }

                "h1" => { new_layout_state = LayoutState {font_size: FONT_SIZE + 12, bold: true , ..*layout_state};
                            move_to_next_line(next_position); move_to_next_line_after = true; }
                "h2" => { new_layout_state = LayoutState {font_size: FONT_SIZE + 10, bold: true , ..*layout_state};
                            move_to_next_line(next_position); move_to_next_line_after = true; }
                "h3" => { new_layout_state = LayoutState {font_size: FONT_SIZE + 8,  bold: true , ..*layout_state};
                            move_to_next_line(next_position); move_to_next_line_after = true; }
                "h4" => { new_layout_state = LayoutState {font_size: FONT_SIZE + 6,  bold: true , ..*layout_state};
                            move_to_next_line(next_position); move_to_next_line_after = true; }
                "h5" => { new_layout_state = LayoutState {font_size: FONT_SIZE + 4,  bold: true , ..*layout_state};
                            move_to_next_line(next_position); move_to_next_line_after = true; }
                "h6" => { new_layout_state = LayoutState {font_size: FONT_SIZE + 2,  bold: true , ..*layout_state};
                            move_to_next_line(next_position); move_to_next_line_after = true; }


                //TODO: this one might not be neccesary any more after we fix our html parser to not try to parse the javascript
                "script" => { new_layout_state = LayoutState {visible: false, ..*layout_state}; }

                //TODO: eventually we want to do something else with the title (update the window title or so)
                "title" => { new_layout_state = LayoutState {visible: false, ..*layout_state}; }

                _ => {
                    new_layout_state = LayoutState {..*layout_state}; //TODO: this is a lot of needless copying
                }

            }

            childs_to_recurse_on = &node.children;
        }
        DomNode::Attribute(_) => {
            new_layout_state = LayoutState {..*layout_state}; //TODO: this is a lot of needless copying

            //TODO: this is a bit weird, we should error on getting to this point (because they don't need seperate layout),
            //      but then we need to make sure we handle them in their parents node
        },
        DomNode::Text(node) => {
            if layout_state.visible {
                match &node.text_content { //TODO: understand why I need a reference here
                    None => (),
                    Some(text) => {

                        //TODO: I need a font here, which is annoying.
                            //TODO: I now also need font sizes, making it even more annoying

                        let own_font = Font::new(layout_state.bold, layout_state.font_size); //TODO: the font should just live on the layout_node
                        let font = font_cache.get_font(&own_font);
                        let dimension = get_text_dimension(&text, &font);

                        if next_position.x + dimension.width > SCREEN_WIDTH - LAYOUT_MARGIN_HORIZONTAL {
                            move_to_next_line(next_position);
                        }

                        //let optional_link_url = if document.has_parent_with_tag_name(main_node, "a") {

                            //TODO: this still doesn't work, because I need to find the href on that parent a node, not on this node
                            //      the proper way to solve this is to do also layout to higher nodes, but wrapping is a challenge (should probably
                            //      be a list of rects then for higher noes....) and to add the clickBoxes when processing the "a" node....


                            //TODO: the direct unwrap after looking for href below is of course not good
                            //Some(main_node.find_attribute_value("href").unwrap().concat())
                        //} else {
                        //    None
                        //};

                        let new_node = LayoutNode {
                            text: Option::Some(text.to_string()),
                            position: next_position.clone(),
                            bold: layout_state.bold,
                            font_size: layout_state.font_size,
                            optional_link_url: None, //optional_link_url,
                            children: None
                        };
                        layout_nodes.push(new_node);

                        //TODO: this does not account for the height. We should track the max height, and add that when we move to the next line
                        move_next_position_by_x(next_position, dimension.width);
                    }
                }
            }
            new_layout_state = LayoutState {..*layout_state}; //TODO: this is a lot of needless copying
        }
    }

    if let Some(ref children) = childs_to_recurse_on {
        for child in children.iter() {
            layout_nodes.append(&mut layout_children(child, document, next_position, &new_layout_state, font_cache));
        }
    }

    if (move_to_next_line_after) {
        move_to_next_line(next_position);
    }

    return layout_nodes;
}


fn build_header_nodes(position: &mut Position) -> Vec<LayoutNode> {
    //TODO: eventually we want to not have this in the same node list I think (maybe not even as layout nodes?)
    let mut layout_nodes: Vec<LayoutNode> = Vec::new();

    layout_nodes.push(LayoutNode {
        text: Option::from(String::from("BBrowser")),
        position: position.clone(),
        bold: true,
        font_size: FONT_SIZE,
        optional_link_url: None,
        children: None,
    });
    position.y += 50;

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
