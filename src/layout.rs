use crate::{
    Font,
    FontCache,
    FONT_SIZE,
    HORIZONTAL_ELEMENT_SPACING,
    LAYOUT_MARGIN_HORIZONTAL,
    SCREEN_WIDTH,
    VERTICAL_ELEMENT_SPACING
};
use crate::debug::debug_print_html_node_tree;
use crate::html_parser::HtmlNode;
use crate::html_parser::HtmlNodeType;
use crate::renderer::{get_text_dimension, Position};


pub struct LayoutNode {
    pub text: Option<String>, //eventually we need different kinds of layout nodes, text is just one type
    pub position: Position,
    pub bold: bool,
    pub font_size: u16,
    pub optional_link_url: Option<String>, //TODO: this is a stupid hack because layout nodes don't remember what DOM node they are built from,
                                           //      we should store that on them somehow, but can't get it working ownershipwise currently
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

pub struct ClickBox {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub target_url: String //TODO: this should become more general, not everything you click on is a url!
}


pub fn build_layout_list(html_nodes: &Vec<HtmlNode>, font_cache: &mut FontCache) -> Vec<LayoutNode> {
    //TODO: I really don't want to accept the different font here of course, in any case these should be my own Font objects, but also looked up differently
    let mut layout_nodes: Vec<LayoutNode> = Vec::new();
    let mut next_position = Position {x: 10, y: 10};

    debug_print_html_node_tree(&html_nodes, "START_OF_BUILD_LAYOUT_LIST");

    layout_nodes.append(&mut build_header_nodes(&mut next_position));

    for html_node in html_nodes {
        let layout_state = LayoutState {bold : false, font_size: FONT_SIZE, visible: true};
        layout_nodes.append(&mut layout_children(html_node, &mut next_position, &layout_state, font_cache)); //TODO: understand the &mut in the argument better!
    }

    return layout_nodes;
}


pub fn compute_click_boxes(layout_nodes: &Vec<LayoutNode>) -> Vec<ClickBox> {
    let mut click_boxes: Vec<ClickBox> = Vec::new();

    for layout_node in layout_nodes {

        if layout_node.optional_link_url.is_some() {
            click_boxes.push(
                ClickBox {
                    x: layout_node.position.x,
                    y: layout_node.position.y,
                    width: 100,  //TODO: I'm hardcoding width and height here, because the layout node does not know how large it is. That should change somehow.
                    height: 20,
                    target_url: layout_node.optional_link_url.as_ref().unwrap().to_string()
                }
            )
        }
        //TODO: maybe I should just create the clickboxes when I also build the layout tree? -> yes

    }

    return click_boxes;
}


fn layout_children(main_node: &HtmlNode, next_position: &mut Position, layout_state: &LayoutState, font_cache: &mut FontCache) -> Vec<LayoutNode> {
    let mut layout_nodes: Vec<LayoutNode> = Vec::new();

    let new_layout_state : LayoutState;
    let mut move_to_next_line_after = false;


    if let HtmlNodeType::Text = main_node.node_type {

        if layout_state.visible {
            match &main_node.text_content { //TODO: I don't completely get why I need a & in the match here
                None => (),
                Some(text_vec) => {
                    
                    for text in text_vec {

                        //TODO: I need a font here, which is annoying.
                            //TODO: I now also need font sizes, making it even more annoying
            

                        let own_font = Font::new(layout_state.bold, layout_state.font_size); //TODO: the font should just live on the layout_node
                        let font = font_cache.get_font(&own_font);
                        let dimension = get_text_dimension(text, &font);
            
                        if next_position.x + dimension.width > SCREEN_WIDTH - LAYOUT_MARGIN_HORIZONTAL {
                            move_to_next_line(next_position);
                        }

                        //TODO: the code below is very broken, because the text node is a child of the "a" node, not the a node itself. We need to
                        //      be able to traverse the tree....
                        //let optional_link_url = if main_node.tag_name.is_some() && main_node.tag_name.as_ref().unwrap() == "a" {
                            //TODO: the direct unwrap after looking for href below is of course not good
                        //    Some(main_node.find_attribute_value("href").unwrap().concat())
                        //} else {
                        //    None
                        //};

                        let new_node = LayoutNode {
                            text: Option::Some(text.to_string()),
                            position: next_position.clone(),
                            bold: layout_state.bold,
                            font_size: layout_state.font_size,
                            optional_link_url: optional_link_url,
                        };
            
                        layout_nodes.push(new_node);
            
            
                        //TODO: this does not account for the height. We should track the max height, and add that when we move to the next line
                        move_next_position_by_x(next_position, dimension.width);
                    }
        
                }
            }
        }
        new_layout_state = LayoutState {..*layout_state}; //TODO: this is a lot of needless copying

    } else if let HtmlNodeType::Tag = main_node.node_type {

        if main_node.tag_name.is_some() {
            let tag_name = main_node.tag_name.as_ref().unwrap(); //TODO: I don't get why I need as_ref() here

            match &tag_name[..] {
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

        } else {
            new_layout_state = LayoutState {..*layout_state}; //TODO: this is a lot of needless copying

        }


    } else {
        new_layout_state = LayoutState {..*layout_state}; //TODO: this is a lot of needless copying

    }

    if let Some(ref children) = main_node.children {
        for child in children.iter() {
            layout_nodes.append(&mut layout_children(child, next_position, &new_layout_state, font_cache));
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
        optional_link_url: None
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
