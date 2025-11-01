use std::{
    cell::RefCell,
    rc::Rc
};

use crate::dom::{Document, ElementDomNode};
use crate::layout::{
    build_layout_tree,
    compute_layout_for_node,
    compute_potential_widths,
    get_next_layout_node_interal_id,
    CssBox,
    FormattingContext,
    LayoutNode,
    LayoutNodeContent,
    PositioningScheme,
};
use crate::platform::fonts::FontContext;


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct TableLayoutNode {
    pub width_in_slots: usize,
    pub height_in_slots: usize,
    pub css_box: CssBox,
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct TableCellLayoutNode {
    pub css_box: CssBox,
    pub slot_x_idx: usize,
    pub slot_y_idx: usize,
    pub cell_width: usize,
    pub cell_height: usize,
}


pub fn compute_layout_for_table(table_layout_node: &Rc<RefCell<LayoutNode>>, top_left_x: f32, top_left_y: f32, font_context: &FontContext,
                                current_scroll_y: f32, only_update_block_vertical_position: bool, force_full_layout: bool, available_width: f32) {

    debug_assert!(matches!(table_layout_node.borrow().content, LayoutNodeContent::TableLayoutNode { .. }));

    let (table_width_in_slots, table_height_in_slots) = if let LayoutNodeContent::TableLayoutNode(node) = &table_layout_node.borrow().content {
        (node.width_in_slots, node.height_in_slots)
    } else {
        (0, 0)
    };

    let mut element_minimum_widths = Vec::with_capacity(table_width_in_slots);
    let mut element_potential_widths = Vec::with_capacity(table_width_in_slots);
    for _idx in 0..table_width_in_slots {
        element_minimum_widths.push(0.0);
        element_potential_widths.push(0.0);
    }

    let layout_node_borrow = table_layout_node.borrow();
    let cell_nodes = layout_node_borrow.children.as_ref().unwrap();

    for y_pos in 0..table_height_in_slots {
        for x_pos in 0..table_width_in_slots {

            let cell = find_node_at_table_slot(cell_nodes, x_pos, y_pos, true);
            if cell.is_some() {
                let (minimum_element_width, potentential_element_width) = compute_potential_widths(cell.as_ref().unwrap(), font_context);

                let nr_slots_wide = match &cell.as_ref().unwrap().borrow().content {
                    LayoutNodeContent::TableCellLayoutNode(table_cell_layout_node) => table_cell_layout_node.cell_width,
                    _ => panic!("expected a TableCellLayoutNode"),
                };

                let prev_minimum_widths: f32 = element_minimum_widths[x_pos..x_pos+nr_slots_wide].iter().sum();
                if minimum_element_width > prev_minimum_widths {
                    if nr_slots_wide == 1 {
                        element_minimum_widths[x_pos] = minimum_element_width;
                    } else {
                        let missing = minimum_element_width - prev_minimum_widths;
                        let missing_per_col = missing / nr_slots_wide as f32;
                        for col_idx in 0..nr_slots_wide {
                            element_minimum_widths[col_idx] += missing_per_col;
                        }
                    }
                }

                let prev_potential_widths: f32 = element_potential_widths[x_pos..x_pos+nr_slots_wide].iter().sum();
                if potentential_element_width > prev_potential_widths {
                    if nr_slots_wide == 1 {
                        element_potential_widths[x_pos] = potentential_element_width;
                    } else {
                        let missing = potentential_element_width - prev_potential_widths;
                        let missing_per_col = missing / nr_slots_wide as f32;
                        for col_idx in 0..nr_slots_wide {
                            element_potential_widths[col_idx] += missing_per_col;
                        }
                    }
                }
            }
        }
    }

    let mut total_min_width = 0.0;
    let mut total_potential_width = 0.0;
    for idx in 0..table_width_in_slots {
        total_min_width += element_minimum_widths[idx];
        total_potential_width += element_potential_widths[idx];
    }

    let mut column_widths = Vec::with_capacity(table_width_in_slots);

    if total_potential_width <= available_width {
        for idx in 0..table_width_in_slots {
            column_widths.push(element_potential_widths[idx]);
        }
    } else {
        let remaining_space = f32::max(0.0, available_width - total_min_width);
        let needed_extra_space = total_potential_width - total_min_width;
        let shrink_factor = remaining_space / needed_extra_space;

        for idx in 0..table_width_in_slots {
            let min_for_col = element_minimum_widths[idx];
            let potential_for_col = element_potential_widths[idx];
            let potential_added_per_col = potential_for_col - min_for_col;
            column_widths.push(min_for_col + (potential_added_per_col * shrink_factor));
        }
    }

    let mut cursor_x = top_left_x;
    let mut cursor_y = top_left_y;

    let mut max_cursor_x_seen = 0.0;
    let mut max_cursor_y_seen = 0.0;

    let mut minimal_starting_point_per_row = Vec::with_capacity(table_height_in_slots);
    for _idx in 0..table_height_in_slots {
        minimal_starting_point_per_row.push(0.0);
    }

    for y_pos in 0..table_height_in_slots {
        let mut max_height_of_row = 0.0;

        let minimal_starting_point = minimal_starting_point_per_row[y_pos];
        if cursor_y < minimal_starting_point {
            cursor_y = minimal_starting_point;
        }

        for x_pos in 0..table_width_in_slots {
            let cell = find_node_at_table_slot(cell_nodes, x_pos, y_pos, true);
            if cell.is_some() {
                let cell = cell.unwrap();

                let (nr_slots_wide, nr_slots_high) = match &cell.borrow().content {
                    LayoutNodeContent::TableCellLayoutNode(table_cell_layout_node) => {
                        (table_cell_layout_node.cell_width, table_cell_layout_node.cell_height)
                    },
                    _ => panic!("expected a TableCellLayoutNode"),
                };
                let available_cell_width: f32 = column_widths[x_pos..x_pos+nr_slots_wide].iter().sum();

                compute_layout_for_node(&cell, cursor_x, cursor_y, font_context, current_scroll_y,
                                        only_update_block_vertical_position, force_full_layout, available_cell_width, true);

                let element_height = cell.borrow().get_bounding_box().3;

                if nr_slots_high == 1 {
                    if element_height > max_height_of_row {
                        max_height_of_row = element_height;
                    }
                } else {
                    let first_row_below = y_pos + nr_slots_high;
                    let minimal_starting_point = cursor_y + element_height;

                    if minimal_starting_point > minimal_starting_point_per_row[first_row_below] {
                        minimal_starting_point_per_row[first_row_below] = minimal_starting_point;
                    }
                }

            }

            cursor_x += column_widths[x_pos];
            if max_cursor_x_seen < cursor_x {
                max_cursor_x_seen = cursor_x;
            }
        }

        cursor_x = top_left_x;
        cursor_y += max_height_of_row;
        if max_cursor_y_seen < cursor_y {
            max_cursor_y_seen = cursor_y;
        }
    }

    drop(layout_node_borrow);
    table_layout_node.borrow_mut().update_css_box(CssBox { x: top_left_x, y: top_left_y, width: max_cursor_x_seen, height: max_cursor_y_seen });
}


pub fn build_layout_tree_for_table(table_dom_node: &Rc<RefCell<ElementDomNode>>, document: &Document, font_context: &FontContext) -> LayoutNode {
    let mut layout_children = Vec::new();
    let mut slot_y_idx = 0;
    let mut highest_slot_x_idx = 0;
    let mut highest_slot_y_idx = 0;

    if table_dom_node.borrow().children.is_some() {
        for dom_table_child in table_dom_node.borrow().children.as_ref().unwrap() {

            let dom_table_child = dom_table_child.borrow();
            if dom_table_child.name.is_some() && dom_table_child.name.as_ref().unwrap() == &String::from("tr") {

                let mut slot_x_idx = 0;

                if dom_table_child.children.is_some() {
                    for dom_row_child in dom_table_child.children.as_ref().unwrap() {

                        let borr_dom_row_child = dom_row_child.borrow();
                        if borr_dom_row_child.name.is_some() && (borr_dom_row_child.name.as_ref().unwrap() == &String::from("td") ||
                                                                 borr_dom_row_child.name.as_ref().unwrap() == &String::from("th")) {

                            let colspan_str = borr_dom_row_child.get_attribute_value("colspan");
                            let colspan = if colspan_str.is_some() {
                                //TODO: this parsing to numeric should probably live on the dom node somewhere
                                colspan_str.unwrap().parse::<usize>().unwrap_or(1)
                            } else {
                                1
                            };

                            let rowspan_str = borr_dom_row_child.get_attribute_value("rowspan");
                            let rowspan = if rowspan_str.is_some() {
                                //TODO: this parsing to numeric should probably live on the dom node somewhere
                                rowspan_str.unwrap().parse::<usize>().unwrap_or(1)
                            } else {
                                1
                            };

                            loop {
                                let mut all_are_free = true;
                                'offset_loops: for x_offset in 0..colspan {
                                    for y_offset in 0..rowspan {
                                        if find_node_at_table_slot(&layout_children, slot_x_idx + x_offset, slot_y_idx + y_offset, false).is_some() {
                                            all_are_free = false;
                                            break 'offset_loops;
                                        }
                                    }
                                }
                                if all_are_free {
                                    break;
                                } else {
                                    slot_x_idx += 1;
                                }
                            }

                            let mut cell_children = Vec::new();

                            if borr_dom_row_child.children.is_some() {
                                for dom_cell_child in borr_dom_row_child.children.as_ref().unwrap() {
                                    let layout_child = build_layout_tree(dom_cell_child, document, font_context);
                                    cell_children.push(layout_child);
                                }
                            }

                            let cell_layout_node = LayoutNode {
                                internal_id: get_next_layout_node_interal_id(),
                                children: Some(cell_children),
                                from_dom_node: Some(dom_row_child.clone()),
                                formatting_context: FormattingContext::Inline, //TODO: this should be based on the css properties
                                                                               //      its now inline to make text work, but this is not always correct
                                visible: true,
                                content: LayoutNodeContent::TableCellLayoutNode(TableCellLayoutNode {
                                    css_box: CssBox::empty(),
                                    slot_x_idx,
                                    slot_y_idx,
                                    cell_width: colspan,
                                    cell_height: rowspan,
                                }),
                                positioning_scheme: PositioningScheme::Static,
                            };
                            if slot_x_idx + colspan - 1 > highest_slot_x_idx { highest_slot_x_idx = slot_x_idx + colspan - 1; }
                            if slot_y_idx + rowspan - 1 > highest_slot_y_idx { highest_slot_y_idx = slot_y_idx + rowspan - 1; }

                            layout_children.push(Rc::from(RefCell::from(cell_layout_node)));

                            slot_x_idx += colspan;
                        }

                        //TODO: handle other cases, we at least also have table body, in which case we need to recurse somehow
                        //      there might also be text (at the very least whitespace that we should ignore) in between rows and cells
                    }
                }

                slot_y_idx += 1;
            }

            //TODO: handle other cases, we at least also have table body, in which case we need to recurse somehow
            //      there might also be text (at the very least whitespace that we should ignore) in between rows and cells
        }
    }

    return LayoutNode {
        internal_id: get_next_layout_node_interal_id(),
        children: Some(layout_children),
        from_dom_node: Some(table_dom_node.clone()),
        formatting_context: FormattingContext::Table,
        visible: true,
        content: LayoutNodeContent::TableLayoutNode(TableLayoutNode {
            css_box: CssBox::empty(),
            width_in_slots: highest_slot_x_idx + 1,
            height_in_slots: highest_slot_y_idx + 1,
        }),
        positioning_scheme: PositioningScheme::Static,
    }
}


pub fn find_node_at_table_slot(cells: &Vec<Rc<RefCell<LayoutNode>>>, slot_x_idx: usize, slot_y_idx: usize, only_match_anchor: bool) -> Option<Rc<RefCell<LayoutNode>>> {
    for cell in cells {

        match &cell.borrow().content {
            LayoutNodeContent::TableCellLayoutNode(table_cell_layout_node) => {
                let x = table_cell_layout_node.slot_x_idx;
                let y = table_cell_layout_node.slot_y_idx;

                if only_match_anchor {
                    if x == slot_x_idx && y == slot_y_idx {
                        return Some(cell.clone());
                    }
                } else {
                    let width = table_cell_layout_node.cell_width;
                    let height = table_cell_layout_node.cell_height;
                    if x <= slot_x_idx && slot_x_idx <= (x + width - 1) && y <= slot_y_idx && slot_y_idx <= (y + height - 1) {
                        return Some(cell.clone());
                    }
                }
            },
            _ => {
                panic!("Only expecting table cell nodes here");
            }
        }
    }

    return None;
}
