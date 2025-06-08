use std::{cell::RefCell, rc::Rc};

use crate::layout::{
    CssTextBox, FullLayout, LayoutNode, LayoutNodeContent
};


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub struct Selection {
    pub point1_x: f32,
    pub point1_y: f32,
    pub point2_x: f32,
    pub point2_y: f32,
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub struct SelectionRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}


pub fn set_selection_regions(full_layout: &FullLayout, selection: &Selection) {

    let node1 = full_layout.root_node.borrow().find_content_child_at_position(selection.point1_x, selection.point1_y);
    let node2 = full_layout.root_node.borrow().find_content_child_at_position(selection.point2_x, selection.point2_y);

    let node1 = if node1.is_none() {
        full_layout.root_node.clone()  // take the root node if the click was outside any known node
    } else {
        node1.unwrap()
    };
    let node2 = if node2.is_none() {
        full_layout.root_node.clone()  // take the root node if the click was outside any known node
    } else {
        node2.unwrap()
    };

    let mut content_node1 = node1.clone();
    while content_node1.borrow().children.is_some() && content_node1.borrow().children.as_ref().unwrap().len() > 0 {
        let child = content_node1.borrow().children.as_ref().unwrap()[0].clone();
        content_node1 = child;
    }
    let mut content_node2 = node2.clone();
    while content_node2.borrow().children.is_some() && content_node2.borrow().children.as_ref().unwrap().len() > 0 {
        let child = content_node2.borrow().children.as_ref().unwrap()[0].clone();
        content_node2 = child;
    }


    let possible_content_node = go_down_to_content_node(node1, selection);
    if possible_content_node.is_none() {
        return; //There is no content node we can select, so we don't select anything
    }
    let node1 = possible_content_node.unwrap();

    let possible_content_node = go_down_to_content_node(node2, selection);
    if possible_content_node.is_none() {
        return; //There is no content node we can select, so we don't select anything
    }
    let node2 = possible_content_node.unwrap();


    let mut start_point = (selection.point1_x, selection.point1_y);
    let mut end_point = (selection.point2_x, selection.point2_y);
    let mut start_node = node1.clone();
    let mut end_node = node2.clone();

    let mut invert = false;


    if start_node.borrow().internal_id == end_node.borrow().internal_id {
        //TODO: what should we do here? We need to compare which line (if it even has lines) it has

        match &start_node.borrow().content {
            LayoutNodeContent::TextLayoutNode(text_layout_node) => {

                for css_text_box in &text_layout_node.css_text_boxes {

                    if css_text_box.css_box.is_inside(start_point.0, start_point.1) {
                        if css_text_box.css_box.is_inside(end_point.0, end_point.1) {
                            if start_point.0 > end_point.0 {
                                invert = true;
                            }
                        }
                        break;
                    }
        
                    if css_text_box.css_box.is_inside(end_point.0, end_point.1) {
                        invert = true;
                        break;
                    }

                }

            },
            _ => todo!(), //TODO: I think for most other cases we just compare the actual positions... (maybe just for all, in this default case?)
        }


    } else {

        for node_to_check in &full_layout.content_nodes_in_selection_order {
            if node_to_check.borrow().internal_id == content_node1.borrow().internal_id {
                break;
            }
            if node_to_check.borrow().internal_id == content_node2.borrow().internal_id {
                invert = true;
                break;
            }
        }
    }


    if invert {
        start_node = node2.clone();
        end_node = node1.clone();
        start_point = (selection.point2_x, selection.point2_y);
        end_point = (selection.point1_x, selection.point1_y);
    }


    let mut found_start = false;
    for content_node in full_layout.content_nodes_in_selection_order.iter() {

        if content_node.borrow().internal_id == end_node.borrow().internal_id {
            break;
        }

        if found_start {
            set_content_fully_selected(&mut content_node.borrow_mut().content);
        }

        if content_node.borrow().internal_id == start_node.borrow().internal_id {
            found_start = true;
        }
    }


    //handle the selection within the start node
    let mut start_node_borr = start_node.borrow_mut();
    match &mut start_node_borr.content {
        LayoutNodeContent::TextLayoutNode(text_layout_node) => {

            let mut first_selected_box_found = false;
            for css_text_box in text_layout_node.css_text_boxes.iter_mut() {

                if first_selected_box_found {
                    set_css_text_box_fully_selected(css_text_box);
                    continue;
                }

                let css_box = &css_text_box.css_box;
                let css_end_y = css_box.y + css_box.height;

                if start_point.1 <= css_end_y && start_point.1 >= css_box.y {

                    let selection_overlap_start = f32::max(start_point.0, css_box.x);
                    let mut selection_overlap_end = f32::min(end_point.0, css_box.x + css_box.width);

                    if end_point.1 > css_box.y + css_box.height {
                        //select the whole rest of the rect, since the selection is going further down
                        selection_overlap_end = css_box.x + css_box.width;
                    }

                    set_css_text_box_partially_selected(css_text_box, selection_overlap_start, selection_overlap_end);
                    first_selected_box_found = true;
                }
            }

            if !first_selected_box_found {
                //Nothing was matched based on position, but we did find this node as start node, so we set all as selected
                set_content_fully_selected(&mut start_node_borr.content);
            }

        },
        LayoutNodeContent::ImageLayoutNode(_) => {
            //TODO: select the image if enough of it is selected
            todo!();
        },
        _ => {},
    }
    drop(start_node_borr);


    //handle the selection within the end node, if it is another node than the start node
    if start_node.borrow().internal_id != end_node.borrow().internal_id {

        let mut end_node_borr = end_node.borrow_mut();
        match &mut end_node_borr.content {
            LayoutNodeContent::TextLayoutNode(text_layout_node) => {

                let mut last_selected_box_found = false;
                for css_text_box in text_layout_node.css_text_boxes.iter_mut() {

                    let css_box = &css_text_box.css_box;
                    let css_end_y = css_box.y + css_box.height;

                    if end_point.1 <= css_end_y && end_point.1 >= css_box.y {

                        let selection_overlap_start = css_box.x;
                        let selection_overlap_end = f32::min(end_point.0, css_box.x + css_box.width);
                        println!("check {} {} {}", end_point.0, css_box.x + css_box.width, selection_overlap_end);

                        set_css_text_box_partially_selected(css_text_box, selection_overlap_start, selection_overlap_end);
                        last_selected_box_found = true;
                    }

                    if !last_selected_box_found {
                        set_css_text_box_fully_selected(css_text_box);
                        continue;
                    }

                }

            },
            LayoutNodeContent::ImageLayoutNode(_) => {
                //TODO: select the image if enough of it is selected
                todo!();
            },
            _ => {},
        }
        drop(end_node_borr);
    }
}



fn go_down_to_content_node(node: Rc<RefCell<LayoutNode>>, selection: &Selection) -> Option<Rc<RefCell<LayoutNode>>> {

    let mut current_node = node.clone();

    //If we hit a non-content node with children for the start node, we now need to find the first content node in the selection
    loop {
        let current_node_borr = current_node.borrow();
        if current_node_borr.children.is_none() || current_node_borr.children.as_ref().unwrap().len() == 0 {
            drop(current_node_borr);
            return Some(current_node);
        }

        let mut found = false;
        for child in current_node_borr.children.as_ref().unwrap() {

            let (x, y, width, height) = child.borrow().get_bounding_box();
            let bottom_right_x = x + width;
            let bottom_right_y = y + height;

            if bottom_right_x >= selection.point1_x && bottom_right_y >= selection.point1_y {
                let child_rc_clone = child.clone();
                found = true;
                drop(current_node_borr);
                current_node = child_rc_clone;
                break;
            }
        }

        if !found {
            return None; //There is no content node we can select, so we don't select anything
        }
    }

}


fn set_content_fully_selected(content: &mut LayoutNodeContent) {

    match content {
        LayoutNodeContent::TextLayoutNode(ref mut text_layout_node) => {
            for text_box in text_layout_node.css_text_boxes.iter_mut() {
                set_css_text_box_fully_selected(text_box);
            }
        },
        LayoutNodeContent::ImageLayoutNode(_) => todo!(),
        LayoutNodeContent::ButtonLayoutNode(_) => todo!(),
        LayoutNodeContent::TextInputLayoutNode(_) => todo!(),
        LayoutNodeContent::AreaLayoutNode(_) => todo!(),
        LayoutNodeContent::TableLayoutNode(_) => todo!(),
        LayoutNodeContent::TableCellLayoutNode(_) => todo!(),
        LayoutNodeContent::NoContent => todo!(),
    }
}


fn set_css_text_box_fully_selected(css_text_box: &mut CssTextBox) {
    let selection_rect_for_css_text_box = SelectionRect { x: css_text_box.css_box.x, y: css_text_box.css_box.y,
                                                            width: css_text_box.css_box.width, height: css_text_box.css_box.height};
    css_text_box.selection_rect = Some(selection_rect_for_css_text_box);
    css_text_box.selection_char_range = Some( (0, css_text_box.text.len()) );

}


fn set_css_text_box_partially_selected(css_text_box: &mut CssTextBox, selection_start: f32, selection_end: f32) {
    let css_box = &css_text_box.css_box;

    let mut start_selection_idx = 0;
    for (idx, &x_position) in css_text_box.char_position_mapping.iter().enumerate() {
        let abs_pos = x_position + css_box.x;

        if abs_pos >= selection_start {
            start_selection_idx = idx;
            break;
        }
    }

    let mut end_selection_idx = css_text_box.char_position_mapping.len() - 1;
    for (idx, &x_position) in css_text_box.char_position_mapping.iter().enumerate().rev() {
        let abs_pos = x_position + css_box.x;

        if abs_pos <= selection_end {
            if idx + 1 < css_text_box.char_position_mapping.len() {
                end_selection_idx = idx + 1;
            } else {
                end_selection_idx = idx;
            }
            
            break;
        }
    }

    let start_char_boundary = if start_selection_idx == 0 {
        css_text_box.char_position_mapping[start_selection_idx]
    } else {
        css_text_box.char_position_mapping[start_selection_idx - 1]
    };
    let end_char_boundary = css_text_box.char_position_mapping[end_selection_idx];
    let selection_width = end_char_boundary - start_char_boundary;

    css_text_box.selection_char_range = Some( (start_selection_idx, end_selection_idx) );
    css_text_box.selection_rect = Some(SelectionRect { x: start_char_boundary, y: css_box.y,
                                                        width: selection_width, height: css_box.height });

}

