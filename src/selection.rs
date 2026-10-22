use crate::layout::{
    CssBox,
    CssTextBox,
    FullLayout,
    LayoutNodeContent
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


//we use PartialOrd here to be able to compare on selection order in the document
#[derive(Clone, Copy, PartialEq, PartialOrd)]
struct SelectionPosition {
    node_idx: usize, //this is the index in the list of nodes in selection order
    box_idx: usize,
    char_idx: usize,
}


pub fn set_selection_regions(full_layout: &FullLayout, selection: &Selection) {
    full_layout.root_node.borrow_mut().reset_selection();

    let position1 = find_selection_position(full_layout, selection.point1_x, selection.point1_y);
    let position2 = find_selection_position(full_layout, selection.point2_x, selection.point2_y);
    if position1.is_none() || position2.is_none() {
        return; //There is no content we can select
    }

    let (start, end) = if position1.unwrap() <= position2.unwrap() {
        (position1.unwrap(), position2.unwrap())
    } else {
        (position2.unwrap(), position1.unwrap())
    };

    for node_idx in start.node_idx..(end.node_idx + 1) {
        let mut node = full_layout.content_nodes_in_selection_order[node_idx].borrow_mut();
        match &mut node.content {
            LayoutNodeContent::TextLayoutNode(text_layout_node) => {
                for (box_idx, css_text_box) in text_layout_node.css_text_boxes.iter_mut().enumerate() {
                    let char_count = css_text_box.char_position_mapping.len();
                    let (from_char_idx, to_char_idx) = selected_char_range_in_box(start, end, node_idx, box_idx, char_count);
                    if from_char_idx < to_char_idx {
                        set_css_text_box_selected(css_text_box, from_char_idx, to_char_idx);
                    }
                }
            },
            LayoutNodeContent::ImageLayoutNode(image_node) => {
                let (from_char_idx, to_char_idx) = selected_char_range_in_box(start, end, node_idx, 0, 1);
                image_node.selected = from_char_idx < to_char_idx;
            },
            _ => {},
        }
    }
}


fn find_selection_position(full_layout: &FullLayout, x: f32, y: f32) -> Option<SelectionPosition> {
    let mut closest_on_same_line: Option<(SelectionPosition, f32)> = None;
    let mut first_below: Option<(SelectionPosition, f32)> = None;
    let mut end_of_last_box: Option<SelectionPosition> = None;

    for (node_idx, node) in full_layout.content_nodes_in_selection_order.iter().enumerate() {
        let node = node.borrow();

        for (box_idx, (css_box, char_end_positions)) in selectable_boxes(&node.content).iter().enumerate() {

            if y >= css_box.y && y <= css_box.y + css_box.height {
                let horizontal_distance = if x < css_box.x {
                    css_box.x - x
                } else {
                    f32::max(0.0, x - (css_box.x + css_box.width))
                };

                if closest_on_same_line.is_none() || horizontal_distance < closest_on_same_line.unwrap().1 {
                    let char_idx = find_char_idx_at(char_end_positions, x - css_box.x);
                    closest_on_same_line = Some((SelectionPosition { node_idx, box_idx, char_idx }, horizontal_distance));
                }
            } else if css_box.y > y {
                if first_below.is_none() || css_box.y < first_below.unwrap().1 {
                    first_below = Some((SelectionPosition { node_idx, box_idx, char_idx: 0 }, css_box.y));
                }
            }

            end_of_last_box = Some(SelectionPosition { node_idx, box_idx, char_idx: char_end_positions.len() });
        }
    }

    if closest_on_same_line.is_some() {
        return Some(closest_on_same_line.unwrap().0);
    }
    if first_below.is_some() {
        return Some(first_below.unwrap().0);
    }
    return end_of_last_box;
}


fn selectable_boxes(content: &LayoutNodeContent) -> Vec<(&CssBox, &[f32])> {
    return match content {
        LayoutNodeContent::TextLayoutNode(text_layout_node) => {
            text_layout_node.css_text_boxes.iter()
                .map(|css_text_box| (&css_text_box.css_box, css_text_box.char_position_mapping.as_slice()))
                .collect()
        },
        LayoutNodeContent::ImageLayoutNode(image_node) => {
            vec![(&image_node.css_box, std::slice::from_ref(&image_node.css_box.width))]
        },
        _ => { Vec::new() },
    };
}


fn find_char_idx_at(char_end_positions: &[f32], relative_x: f32) -> usize {
    for (idx, &char_end) in char_end_positions.iter().enumerate() {
        let char_start = char_start_position(char_end_positions, idx);
        let char_middle = (char_start + char_end) / 2.0;
        if relative_x < char_middle {
            return idx;
        }
    }
    return char_end_positions.len();
}


fn char_start_position(char_end_positions: &[f32], char_idx: usize) -> f32 {
    if char_idx == 0 {
        return 0.0;
    }
    return char_end_positions[char_idx - 1];
}


fn selected_char_range_in_box(start: SelectionPosition, end: SelectionPosition, node_idx: usize, box_idx: usize, char_count: usize) -> (usize, usize) {
    let box_start = SelectionPosition { node_idx, box_idx, char_idx: 0 };
    let box_end = SelectionPosition { node_idx, box_idx, char_idx: char_count };

    if end <= box_start || start >= box_end {
        return (0, 0);
    }

    let from_char_idx = if start > box_start {
        start.char_idx
    } else {
        0
    };
    let to_char_idx = if end < box_end {
        end.char_idx
    } else {
        char_count
    };

    return (from_char_idx, to_char_idx);
}


fn set_css_text_box_selected(css_text_box: &mut CssTextBox, from_char_idx: usize, to_char_idx: usize) {
    let start_x = char_start_position(&css_text_box.char_position_mapping, from_char_idx);
    let end_x = char_start_position(&css_text_box.char_position_mapping, to_char_idx);

    css_text_box.selection_char_range = Some( (from_char_idx, to_char_idx) );
    css_text_box.selection_rect = Some(SelectionRect { x: css_text_box.css_box.x + start_x, y: css_text_box.css_box.y,
                                                       width: end_x - start_x, height: css_text_box.css_box.height });
}
