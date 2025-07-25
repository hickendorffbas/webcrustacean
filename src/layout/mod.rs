use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use image::RgbaImage;

use crate::color::Color;
use crate::debug::debug_log_warn;
use crate::dom::{
    Document,
    DomPropertyDisplay,
    ElementDomNode,
    NavigationAction,
    TagName,
};
use crate::platform::fonts::{
    Font,
    FontContext,
    FontFace,
};
use crate::ui_components::PageComponent;
use crate::SelectionRect;
use crate::style::{
    get_color_style_value,
    get_property_from_computed_styles,
    has_style_value,
    resolve_css_numeric_type_value,
    resolve_full_styles_for_layout_node,
    StyleContext,
};


mod tables;
#[cfg(test)] mod tests;


static NEXT_LAYOUT_NODE_INTERNAL: AtomicUsize = AtomicUsize::new(1);
pub fn get_next_layout_node_interal_id() -> usize { NEXT_LAYOUT_NODE_INTERNAL.fetch_add(1, Ordering::Relaxed) }


pub struct FullLayout {
    pub root_node: Rc<RefCell<LayoutNode>>,
    pub content_nodes_in_selection_order: Vec<Rc<RefCell<LayoutNode>>>, //this list only contains nodes without children
}
impl FullLayout {
    pub fn page_height(&self) -> f32 {
        let node = RefCell::borrow(&self.root_node);
        match &node.content {
            LayoutNodeContent::AreaLayoutNode(area_node) => {
                return area_node.css_box.height;
            },
            _ => { panic!("Root node always should be a box layout node"); }
        }
    }
    pub fn new_empty() -> FullLayout {
        //Note that we we create a 1x1 box even for an empty layout, since we need a box to render it (for example when the first page is still loading)

        let area_node = AreaLayoutNode {
            background_color: Color::WHITE,
            css_box: CssBox { x: 0.0, y: 0.0, width: 1.0, height: 1.0 },
        };

        let mut layout_node = LayoutNode::new_empty();
        layout_node.content = LayoutNodeContent::AreaLayoutNode(area_node);

        return FullLayout { root_node: Rc::from(RefCell::from(layout_node)), content_nodes_in_selection_order: Vec::new() };
    }
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct TextLayoutNode {
    pub line_break: bool,  //TODO: we should not need this. We just need an empty rect, or non layout node at all (as long as we generate the next text lower when layouting)
    pub css_text_boxes: Vec<CssTextBox>,
    pub pre_wrap_box_backup: Option<CssTextBox>,
    pub background_color: Color,
    pub font: Font,
    pub font_color: Color,
    pub non_breaking_space_positions: Option<HashSet<usize>>, //these are the positions in the unsplit boxes
}
impl TextLayoutNode {
    pub fn undo_split_boxes(&mut self) {
        //The main intention for this method is to be used before we start the process of computing line wrapping again (to undo the previous wrapping)

        if self.css_text_boxes.len() > 1 {
            self.css_text_boxes = vec![self.pre_wrap_box_backup.as_ref().unwrap().clone()];
        }
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct ImageLayoutNode {
    pub image: RgbaImage,
    pub css_box: CssBox,
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct ButtonLayoutNode {
    pub css_box: CssBox,
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct TextInputLayoutNode {
    pub css_box: CssBox,
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct AreaLayoutNode {
    pub css_box: CssBox,
    #[allow(dead_code)] pub background_color: Color,  //TODO: use
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub enum LayoutNodeContent {
    TextLayoutNode(TextLayoutNode),
    ImageLayoutNode(ImageLayoutNode),
    ButtonLayoutNode(ButtonLayoutNode),
    TextInputLayoutNode(TextInputLayoutNode),
    AreaLayoutNode(AreaLayoutNode),
    TableLayoutNode(tables::TableLayoutNode),
    TableCellLayoutNode(tables::TableCellLayoutNode),
    NoContent,
}
impl LayoutNodeContent {
    pub fn is_inside(&self, x: f32, y: f32) -> bool {
        match self {
            LayoutNodeContent::TextLayoutNode(text_layout_node) => {
                for css_text_box in text_layout_node.css_text_boxes.iter() {
                    if css_text_box.css_box.is_inside(x, y) { return true; }
                }
                return false;
            },
            LayoutNodeContent::ImageLayoutNode(image_node) => { return image_node.css_box.is_inside(x, y); }
            LayoutNodeContent::AreaLayoutNode(area_node) => { return area_node.css_box.is_inside(x, y); }
            LayoutNodeContent::ButtonLayoutNode(button_node) => { return button_node.css_box.is_inside(x, y); }
            LayoutNodeContent::TextInputLayoutNode(text_input_node) => { return text_input_node.css_box.is_inside(x, y); }
            LayoutNodeContent::TableLayoutNode(table_layout_node) => { return table_layout_node.css_box.is_inside(x, y); },
            LayoutNodeContent::TableCellLayoutNode(table_cell_layout_node) => { return table_cell_layout_node.css_box.is_inside(x, y); }
            LayoutNodeContent::NoContent => { return false; },
        }
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub enum PositioningScheme {
    Static,
    #[allow(dead_code)] Relative, //TODO: use
    #[allow(dead_code)] Absolute, //TODO: use
    #[allow(dead_code)] Fixed, //TODO: use
    #[allow(dead_code)] Sticky, //TODO: use
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct LayoutNode {
    pub internal_id: usize,
    pub children: Option<Vec<Rc<RefCell<LayoutNode>>>>,

    pub from_dom_node: Option<Rc<RefCell<ElementDomNode>>>,

    pub visible: bool,

    pub formatting_context: FormattingContext, //The context for laying out the children of this node
    pub positioning_scheme: PositioningScheme, //The positioning scheme is used for the node itself

    pub content: LayoutNodeContent,
}
impl LayoutNode {
    pub fn update_css_box(&mut self, new_css_box: CssBox) {
        match &mut self.content {
            LayoutNodeContent::TextLayoutNode(node) => {
                debug_assert!(node.css_text_boxes.len() == 1);
                node.css_text_boxes[0].css_box = new_css_box;
            },
            LayoutNodeContent::ImageLayoutNode(node) => { node.css_box = new_css_box; },
            LayoutNodeContent::ButtonLayoutNode(node) => { node.css_box = new_css_box; },
            LayoutNodeContent::TextInputLayoutNode(node) => { node.css_box = new_css_box; },
            LayoutNodeContent::AreaLayoutNode(node) => { node.css_box = new_css_box; },
            LayoutNodeContent::TableLayoutNode(node) => { node.css_box = new_css_box; }
            LayoutNodeContent::TableCellLayoutNode(node) => { node.css_box = new_css_box; }
            LayoutNodeContent::NoContent => { }
        }
    }

    pub fn can_wrap(&self) -> bool {
        //wrapping here means being able to split its css box into mutliple css boxes
        return if let LayoutNodeContent::TextLayoutNode(_) = self.content { true } else { false };
    }

    pub fn y_position(&self) -> f32 {
        return match &self.content {
            LayoutNodeContent::TextLayoutNode(text_layout_node) => { text_layout_node.css_text_boxes.iter().next().unwrap().css_box.y },
            LayoutNodeContent::ImageLayoutNode(image_node) => { image_node.css_box.y }
            LayoutNodeContent::ButtonLayoutNode(button_node) => { button_node.css_box.y }
            LayoutNodeContent::TextInputLayoutNode(text_input_node) => { text_input_node.css_box.y }
            LayoutNodeContent::AreaLayoutNode(box_node) => { box_node.css_box.y }
            LayoutNodeContent::TableLayoutNode(table_node) => { table_node.css_box.y }
            LayoutNodeContent::TableCellLayoutNode(cell_node) => { cell_node.css_box.y }
            LayoutNodeContent::NoContent => { panic!("can't get a position of something without content") },
        }
    }

    pub fn get_bounding_box(&self) -> (f32, f32, f32, f32) { //this returns (top_left_x, top_left_y, width, height)
        match &self.content {
            LayoutNodeContent::TextLayoutNode(text_node) => {
                let mut lowest_x = f32::MAX;
                let mut lowest_y = f32::MAX;
                let mut max_x: f32 = 0.0;
                let mut max_y: f32 = 0.0;

                for css_text_box in text_node.css_text_boxes.iter() {
                    lowest_x = lowest_x.min(css_text_box.css_box.x);
                    lowest_y = lowest_y.min(css_text_box.css_box.y);
                    max_x = max_x.max(css_text_box.css_box.x + css_text_box.css_box.width);
                    max_y = max_y.max(css_text_box.css_box.y + css_text_box.css_box.height);
                }

                let bounding_box_width = max_x - lowest_x;
                let bounding_box_height = max_y - lowest_y;
                return (lowest_x, lowest_y, bounding_box_width, bounding_box_height);
            },
            LayoutNodeContent::ImageLayoutNode(img_node) => { return (img_node.css_box.x, img_node.css_box.y, img_node.css_box.width, img_node.css_box.height); },
            LayoutNodeContent::ButtonLayoutNode(button_node)  => { return (button_node.css_box.x, button_node.css_box.y, button_node.css_box.width, button_node.css_box.height); },
            LayoutNodeContent::TextInputLayoutNode(input_node) => { return (input_node.css_box.x, input_node.css_box.y, input_node.css_box.width, input_node.css_box.height); },
            LayoutNodeContent::AreaLayoutNode(box_node) => { return (box_node.css_box.x, box_node.css_box.y, box_node.css_box.width, box_node.css_box.height); },
            LayoutNodeContent::TableLayoutNode(table_node) => { return (table_node.css_box.x, table_node.css_box.y, table_node.css_box.width, table_node.css_box.height); }
            LayoutNodeContent::TableCellLayoutNode(cell_node) => { return (cell_node.css_box.x, cell_node.css_box.y, cell_node.css_box.width, cell_node.css_box.height); }
            LayoutNodeContent::NoContent => { panic!("invalid state") },
        }
    }

    pub fn visible_on_y_location(&self, y_location: f32, screen_height: f32) -> bool {
        if !self.visible { return false; }

        match &self.content {
            LayoutNodeContent::TextLayoutNode(text_node) => {
                return text_node.css_text_boxes.iter().any(|text_box| -> bool {text_box.css_box.is_visible_on_y_location(y_location, screen_height)});
            },
            LayoutNodeContent::ImageLayoutNode(image_node) => { return image_node.css_box.is_visible_on_y_location(y_location, screen_height); },
            LayoutNodeContent::ButtonLayoutNode(button_node) => { return button_node.css_box.is_visible_on_y_location(y_location, screen_height); }
            LayoutNodeContent::TextInputLayoutNode(text_input_node) => { return text_input_node.css_box.is_visible_on_y_location(y_location, screen_height); }
            LayoutNodeContent::AreaLayoutNode(box_node) => { return box_node.css_box.is_visible_on_y_location(y_location, screen_height); },
            LayoutNodeContent::TableLayoutNode(table_node) => { return table_node.css_box.is_visible_on_y_location(y_location, screen_height); }
            LayoutNodeContent::TableCellLayoutNode(cell_node) => { return cell_node.css_box.is_visible_on_y_location(y_location, screen_height); }
            LayoutNodeContent::NoContent => { return false; }
        }
    }

    pub fn find_dom_node_at_position(&self, x: f32, y: f32) -> Option<Rc<RefCell<ElementDomNode>>> {
        if self.content.is_inside(x, y) {
            if self.children.is_some() {
                for child in self.children.as_ref().unwrap() {
                    if RefCell::borrow(child).visible {
                        let possible_node = child.borrow().find_dom_node_at_position(x, y);
                        if possible_node.is_some() {
                            return possible_node;
                        }
                    }
                }
            }

            if self.from_dom_node.is_some() {
                return Some(self.from_dom_node.as_ref().unwrap().clone());
            }
        }

        return None;
    }

    pub fn find_content_child_at_position(&self, x: f32, y: f32) -> Option<Rc<RefCell<LayoutNode>>> {
        if self.children.is_some() {
            for child in self.children.as_ref().unwrap() {
                if RefCell::borrow(child).visible && child.borrow().content.is_inside(x, y) {
                    let possible_node = child.borrow().find_content_child_at_position(x, y);
                    if possible_node.is_some() {
                        return possible_node;
                    }
                    return Some(child.clone());
                }
            }
        }

        return None;
    }

    pub fn click(&self, x: f32, y: f32, document: &Document) -> NavigationAction {
        let possible_dom_node = self.find_dom_node_at_position(x, y);

        if possible_dom_node.is_some() {
            return possible_dom_node.unwrap().borrow().click(document);
        }
        return NavigationAction::None;
    }

    pub fn new_empty() -> LayoutNode {
        return LayoutNode {
            internal_id: 0,
            visible: true,
            children: None,
            from_dom_node: None,
            content: LayoutNodeContent::NoContent,
            formatting_context: FormattingContext::Block,
            positioning_scheme: PositioningScheme::Static,
        };
    }

    pub fn reset_selection(&mut self) {
        match self.content {
            LayoutNodeContent::TextLayoutNode(ref mut text_layout_node) => {
                for text_box in text_layout_node.css_text_boxes.iter_mut() {
                    text_box.selection_rect = None;
                    text_box.selection_char_range = None;
                }
            },
            LayoutNodeContent::ImageLayoutNode(_) => {
                //For now you can't select images
            },
            LayoutNodeContent::ButtonLayoutNode(_) => {}
            LayoutNodeContent::TextInputLayoutNode(_) => {
                //It seems in other browers, when you select content with a text input in it, the content of the text box is not included
                //   so for now we are not doing anything here...

                //TODO: unsure if I also need to reset the selection _inside_ the text input here.
            },
            LayoutNodeContent::AreaLayoutNode(_) => {
                //Note: this is a no-op for now, since there is nothing to select in a area node itself (just in its children)
            },
            LayoutNodeContent::TableLayoutNode(_) | LayoutNodeContent::TableCellLayoutNode(_) => {
                //Note: for now this is a no-op. There is a usecase of selecing and copying tables, but we don't support it for now
            },
            LayoutNodeContent::NoContent => {},
        }

        if self.children.is_some() {
            for child in self.children.as_ref().unwrap() {
                RefCell::borrow_mut(child).reset_selection();
            }
        }
    }

    pub fn get_selected_text(&self, result: &mut String) {
        match &self.content {
            LayoutNodeContent::TextLayoutNode(text_layout_node) => {
                for css_text_box in &text_layout_node.css_text_boxes {
                    if css_text_box.selection_char_range.is_some() {
                        let (start_idx, end_idx) = css_text_box.selection_char_range.unwrap();
                        result.push_str(css_text_box.text.chars().skip(start_idx).take(end_idx - start_idx + 1).collect::<String>().as_str());
                    }
                }
            },
            LayoutNodeContent::TextInputLayoutNode(_) | LayoutNodeContent::ButtonLayoutNode(_) => {
                //for components, we find it via the focusTarget, not via the tree of nodes, so we do nothing here
            },
            LayoutNodeContent::ImageLayoutNode(_) => {
                //images don't select as text
            }
            LayoutNodeContent::TableLayoutNode(_) => todo!(),  //TODO: implement
            LayoutNodeContent::TableCellLayoutNode(_) => todo!(),  //TODO: implement
            LayoutNodeContent::AreaLayoutNode(_) => {},
            LayoutNodeContent::NoContent => {},
        }

        if self.children.is_some() {
            for child in self.children.as_ref().unwrap() {
                RefCell::borrow_mut(child).get_selected_text(result);
            }
        }
    }

    pub fn is_dirty_anywhere(&self) -> bool {
        if self.from_dom_node.is_some() && self.from_dom_node.as_ref().unwrap().borrow().dirty {
            return true;
        }

        if self.children.is_some() {
            for child in self.children.as_ref().unwrap() {
                if child.borrow().is_dirty_anywhere() {
                    return true;
                }
            }
        }
        return false;
    }

    pub fn move_node_vertically(&mut self, y_diff: f32) {
        match &mut self.content {
            LayoutNodeContent::TextLayoutNode(ref mut text_layout_node) => {
                for text_box in text_layout_node.css_text_boxes.iter_mut() {
                    text_box.css_box.y += y_diff;
                }
            },
            LayoutNodeContent::ImageLayoutNode(image_node) => { image_node.css_box.y += y_diff; }
            LayoutNodeContent::ButtonLayoutNode(button_node) => { button_node.css_box.y += y_diff; }
            LayoutNodeContent::TextInputLayoutNode(text_input_node) => { text_input_node.css_box.y += y_diff; }
            LayoutNodeContent::AreaLayoutNode(box_node) => { box_node.css_box.y += y_diff; }
            LayoutNodeContent::TableLayoutNode(table_node) => { table_node.css_box.y += y_diff; }
            LayoutNodeContent::TableCellLayoutNode(table_cell_node) => { table_cell_node.css_box.y += y_diff; }
            LayoutNodeContent::NoContent => { panic!("Cant adjust position of a node without content"); }
        }

        if self.children.is_some() {
            for child in self.children.as_ref().unwrap() {
                let mut mut_child = RefCell::borrow_mut(child);
                mut_child.move_node_vertically(y_diff);
            }
        }
    }

}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone, Copy)]
#[derive(PartialEq)]
pub enum FormattingContext {
    Block,
    Inline,
    Table,
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub struct CssBox {
    //TODO: eventually things like borders and margins should be included here
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
impl CssBox {
    pub fn is_inside(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width
        &&
        y >= self.y && y <= self.y + self.height
    }
    pub fn empty() -> CssBox {
        return CssBox { x: 0.0, y: 0.0, width: 0.0, height: 0.0 };
    }
    pub fn is_visible_on_y_location(&self, y: f32, screen_height: f32) -> bool {
        //TODO: we are asking for "screen_height" here. Should that not be "content_height" ?
        let top_of_node = self.y;
        let top_of_view = y;
        let bottom_of_node = top_of_node + self.height;
        let bottom_of_view = top_of_view + screen_height;

        return !(top_of_node > bottom_of_view || bottom_of_node < top_of_view);
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub struct CssTextBox {
    pub css_box: CssBox,
    pub text: String,
    pub char_position_mapping: Vec<f32>,
    pub selection_rect: Option<SelectionRect>,
    pub selection_char_range: Option<(usize, usize)>,
}


pub fn build_full_layout(document: &Document, font_context: &FontContext) -> FullLayout {
    let mut top_level_layout_nodes: Vec<Rc<RefCell<LayoutNode>>> = Vec::new();

    let layout_node = build_layout_tree(&document.document_node, document, font_context, FormattingContext::Block);
    top_level_layout_nodes.push(layout_node);

    //Note: we need a node above the first node actually containing any content or styles, since for updates to content or styles we re-assign
    //      children to the parent, so we need all nodes that could update to have a valid parent. That is this root_node for the toplevel node(s).
    let root_node = LayoutNode {
        internal_id: get_next_layout_node_interal_id(),
        visible: true,
        children: Some(top_level_layout_nodes),
        from_dom_node: None,
        content: LayoutNodeContent::AreaLayoutNode(AreaLayoutNode {
            css_box: CssBox::empty(),
            background_color: Color::WHITE,
        }),
        positioning_scheme: PositioningScheme::Static,
        formatting_context: FormattingContext::Block,
    };

    let rc_root_node = Rc::new(RefCell::from(root_node));

    let mut content_nodes_in_selection_order = Vec::new();
    collect_content_nodes_in_walk_order_for_normal_flow(&rc_root_node, &mut content_nodes_in_selection_order);

    return FullLayout { root_node: rc_root_node, content_nodes_in_selection_order };
}


pub fn collect_content_nodes_in_walk_order_for_normal_flow(node: &Rc<RefCell<LayoutNode>>, result: &mut Vec<Rc<RefCell<LayoutNode>>>) {
    //TODO: we need to check if an item is not in normal flow, and not include it in that case (such as position: absolute items etc.)

    match RefCell::borrow(node).content {
        LayoutNodeContent::TextLayoutNode(_) => { result.push(Rc::clone(&node)); },
        LayoutNodeContent::ImageLayoutNode(_) => { result.push(Rc::clone(&node)); },
        LayoutNodeContent::ButtonLayoutNode(_) => { result.push(Rc::clone(&node)); },
        LayoutNodeContent::TextInputLayoutNode(_) => { result.push(Rc::clone(&node)); },

        //nodes that have children, but no selectable content of their own
        LayoutNodeContent::AreaLayoutNode(_) => {},
        LayoutNodeContent::TableLayoutNode(_) => {},
        LayoutNodeContent::TableCellLayoutNode(_) => {},
        LayoutNodeContent::NoContent => {},
    }

    if RefCell::borrow(node).children.as_ref().is_some() {
        for child in RefCell::borrow(node).children.as_ref().unwrap() {
            collect_content_nodes_in_walk_order_for_normal_flow(&child, result);
        }
    }
}


pub fn compute_layout(node: &Rc<RefCell<LayoutNode>>, style_context: &StyleContext, top_left_x: f32, top_left_y: f32, font_context: &FontContext,
                      current_scroll_y: f32, only_update_block_vertical_position: bool, force_full_layout: bool, available_width: f32) {
    compute_layout_for_node(node, style_context, top_left_x, top_left_y, font_context, current_scroll_y,
                            only_update_block_vertical_position, force_full_layout, available_width, true);

    reset_dirtyness(node);
}

fn reset_dirtyness(node: &Rc<RefCell<LayoutNode>>) {
    let node = node.borrow();

    if node.from_dom_node.is_some() {
        let dom_node = node.from_dom_node.as_ref().unwrap();
        RefCell::borrow_mut(dom_node).dirty = false;
    }

    if node.children.is_some() {
        for child in node.children.as_ref().unwrap() {
            reset_dirtyness(child);
        }
    }
}

//This function is responsible for setting the correct css boxes on the node, and all its children, and updating content if needed (sync with DOM)
//TODO: we now pass in top_left x and y, but I think we should compute the positions just for layout, and offset for UI in the render phase...
fn compute_layout_for_node(node: &Rc<RefCell<LayoutNode>>, style_context: &StyleContext, top_left_x: f32, top_left_y: f32, font_context: &FontContext,
                           current_scroll_y: f32, only_update_block_vertical_position: bool, force_full_layout: bool, available_width: f32, allow_single_node_wrap: bool) {

    let mut mut_node = RefCell::borrow_mut(node);

    if only_update_block_vertical_position && !force_full_layout {
        let y_diff = top_left_y - mut_node.y_position();
        mut_node.move_node_vertically(y_diff);
        return;
    }

    if !mut_node.visible {
        mut_node.update_css_box(CssBox { x: top_left_x, y: top_left_y, width: 0.0, height: 0.0 });
        return;
    }

    match mut_node.positioning_scheme {
        PositioningScheme::Static => {

            match mut_node.children {
                Some(_) => {
                    match mut_node.formatting_context {
                        FormattingContext::Block => {
                            apply_block_layout(&mut mut_node, style_context, top_left_x, top_left_y, current_scroll_y, font_context, force_full_layout, available_width);
                        },
                        FormattingContext::Inline => {
                            apply_inline_layout(&mut mut_node, style_context, top_left_x, top_left_y, available_width, current_scroll_y, font_context, force_full_layout);
                        },
                        FormattingContext::Table => {
                            match &mut_node.content {
                                LayoutNodeContent::TableLayoutNode(_) => {
                                    drop(mut_node);
                                    tables::compute_layout_for_table(node, style_context, top_left_x, top_left_y, font_context, current_scroll_y,
                                                                     only_update_block_vertical_position, force_full_layout, available_width);
                                },
                                _ => panic!("Table formatting context on non-table layout node")
                            }
                        },
                    }
                },
                None => {
                    set_css_boxes_for_node_without_children(&mut mut_node, top_left_x, top_left_y, font_context, current_scroll_y, available_width, allow_single_node_wrap);
                },
            }
        },
        _ => todo!("Positioning scheme not yet implemented"),
    }

}


fn set_css_boxes_for_node_without_children(node: &mut LayoutNode, top_left_x: f32, top_left_y: f32, font_context: &FontContext,
                                           current_scroll_y: f32, available_width: f32, allow_single_node_wrap: bool) {

    match &mut node.content {
        LayoutNodeContent::TextLayoutNode(ref mut text_layout_node) => {
            text_layout_node.undo_split_boxes();

            if allow_single_node_wrap && text_layout_node.css_text_boxes.len() != 0 {
                let first_and_only_box = &text_layout_node.css_text_boxes[0];

                text_layout_node.pre_wrap_box_backup = Some(first_and_only_box.clone());
                let wrapped_lines = wrap_text(first_and_only_box, &text_layout_node.non_breaking_space_positions, available_width, available_width);

                let mut css_text_boxes = Vec::new();
                let mut cursor_y = top_left_y;
                for line in wrapped_lines {
                    let (box_width, box_height) = font_context.get_text_dimension(&line, &text_layout_node.font);
                    let text_box = CssTextBox {
                        css_box: CssBox { x: top_left_x, y: cursor_y, width: box_width, height: box_height },
                        char_position_mapping: font_context.compute_char_position_mapping(&text_layout_node.font, &line),
                        text: line,
                        selection_rect: None,
                        selection_char_range: None,
                    };
                    cursor_y += box_height;

                    css_text_boxes.push(text_box);
                }

                text_layout_node.css_text_boxes = css_text_boxes;

            } else {

                for css_text_box in text_layout_node.css_text_boxes.iter_mut() {
                    let (box_width, box_height) = font_context.get_text_dimension(&css_text_box.text, &text_layout_node.font);
                    css_text_box.css_box = CssBox { x: top_left_x, y: top_left_y, width: box_width, height: box_height };
                }
            }
        },
        LayoutNodeContent::ImageLayoutNode(image_layout_node) => {
            image_layout_node.css_box =
                CssBox { x: top_left_x, y: top_left_y, width: image_layout_node.image.width() as f32, height: image_layout_node.image.height() as f32 };
        },
        LayoutNodeContent::ButtonLayoutNode(button_node) => {
            //TODO: for now we are setting a default size here, but that should actually retreived from the DOM
            let button_width = 100.0;  //TODO: this needs to be dependent on the text size. How do we do that? Compute it here?
            let button_height = 40.0;

            button_node.css_box = CssBox { x: top_left_x, y: top_left_y, width: button_width, height: button_height };
            let mut_dom_node = node.from_dom_node.as_ref().unwrap().borrow();
            let mut page_component = mut_dom_node.page_component.as_ref().unwrap().borrow_mut();

            match page_component.deref_mut() {
                PageComponent::Button(button) => {
                    //TODO: here we get the text size, and then add margins, but that is knowledge that should be inside the ui component...
                    //      (for example the exact size of the margins)
                    let text_dimension = font_context.get_text_dimension(&button.text, &button.font);
                    button.update_position(top_left_x, top_left_y - current_scroll_y, text_dimension.0 + 10.0, text_dimension.1 + 10.0);
                }
                PageComponent::TextField(_) => { panic!("Invalid state"); },
            }

        }
        LayoutNodeContent::TextInputLayoutNode(text_input_node) => {
            //TODO: for now we are setting a default size here, but that should actually retreived from the DOM
            let field_width = 500.0;
            let field_height = 40.0;

            text_input_node.css_box = CssBox { x: top_left_x, y: top_left_y, width: field_width, height: field_height };
            let dom_node = node.from_dom_node.as_ref().unwrap().borrow();
            let mut page_component = dom_node.page_component.as_ref().unwrap().borrow_mut();

            match page_component.deref_mut() {
                PageComponent::Button(_) => { panic!("Invalid state"); },
                PageComponent::TextField(text_field) => {
                    text_field.update_position(top_left_x, top_left_y - current_scroll_y, field_width, field_height);
                }
            }
        },
        LayoutNodeContent::AreaLayoutNode(area_node) => {
            //Note: this is a boxlayoutnode, but without children (because that is a seperate case above), so no content.

            //TODO: for now generating 1 by 1 sized, this might not be correct given styling.
            area_node.css_box = CssBox { x: top_left_x, y: top_left_y, width: 1.0, height: 1.0 };
        },
        LayoutNodeContent::NoContent => todo!(), //TODO: should we still compute a position? Maybe it is always 0 by 0 pixels?
        LayoutNodeContent::TableLayoutNode(table_layout_node) => {
            //This is a table without children, so it has no size (the case with children is handled above...)
            table_layout_node.css_box = CssBox { x: top_left_x, y: top_left_y, width: 0.0, height: 0.0 };

        },
        LayoutNodeContent::TableCellLayoutNode(cell_layout_node) => {
            //This is the case where the cell has no children, which means no content, which means no size for rendering
            //(the position of other cells has already been computed when their parent was computed)
            cell_layout_node.css_box = CssBox { x: top_left_x, y: top_left_y, width: 0.0, height: 0.0 };
        },
    }
}


//This returns (the minimal width needed for the element, the potential width the element can take up)
fn compute_potential_widths(node: &Rc<RefCell<LayoutNode>>, font_context: &FontContext, style_context: &StyleContext) -> (f32, f32) {

    compute_layout_for_node(node, style_context, 0.0, 0.0, font_context, 0.0, false, true, 1.0, true);
    let minimal_width = node.borrow().get_bounding_box().2;

    compute_layout_for_node(node, style_context, 0.0, 0.0, font_context, 0.0, false, true, 1000000000.0, true);
    let potential_width = node.borrow().get_bounding_box().2;

    return (minimal_width, potential_width);
}


pub fn get_font_given_styles(styles: &HashMap<String, String>) -> (Font, Color) {
    let font_bold = has_style_value(&styles, "font-weight", &"bold".to_owned());
    let font_italic = has_style_value(styles, "font-style", &"italic".to_owned());
    let _font_underline = has_style_value(&styles, "text-decoration", &"underline".to_owned()); //TODO: we need to use this in a different way

    let opt_font_size = get_property_from_computed_styles(&styles, "font-size");
    let font_size = resolve_css_numeric_type_value(&opt_font_size.unwrap()); //font-size has a default value, so this is a fatal error if not found

    let font_color_option = get_color_style_value(&styles, "color");
    let font_color = font_color_option.unwrap(); //color has a default value, so this is a fatal error if not found

    let default_font_face = FontFace::TimesNewRomanRegular;

    return (Font { face: default_font_face, bold: font_bold, italic: font_italic, size: font_size as u16}, font_color);
}


fn apply_block_layout(node: &mut LayoutNode, style_context: &StyleContext, top_left_x: f32, top_left_y: f32,
                      current_scroll_y: f32, font_context: &FontContext, force_full_layout: bool, available_width: f32) {
    let mut cursor_y = top_left_y;
    let mut max_width: f32 = 0.0;

    for child in node.children.as_ref().unwrap() {
        let only_update_block_vertical_position = !child.borrow().is_dirty_anywhere(); //Since the parent node is block layout, we can shift the while block up and down if its not dirty
        compute_layout_for_node(&child, style_context, top_left_x, cursor_y, font_context, current_scroll_y,
                                only_update_block_vertical_position, force_full_layout, available_width, true);
        let (_, _, bounding_box_width, bounding_box_height) = RefCell::borrow(child).get_bounding_box();

        cursor_y += bounding_box_height;
        max_width = max_width.max(bounding_box_width);
    }

    let our_height = cursor_y - top_left_y;
    node.update_css_box(CssBox { x: top_left_x, y: top_left_y, width: max_width, height: our_height });
}


fn fix_inline_whitespace(node: &mut LayoutNode) {
    //We remove whitespace at the beginning or end of the sequence of inline nodes because it should not show,
    //      and we can't really handle this earlier in the process (like other whitespace correctness) since we don't know what the inline nodes will be then.

    if !node.children.as_ref().unwrap().is_empty() {
        {
            let mut first_child = node.children.as_ref().unwrap().first().unwrap().borrow_mut();

            if let LayoutNodeContent::TextLayoutNode(text_node) = &mut first_child.content {
                let first_box = text_node.css_text_boxes.first_mut().unwrap();
                if first_box.text.starts_with(" ") {
                    first_box.text = first_box.text.trim_start_matches(' ').to_string();
                }
            }
        }

        {
            let mut last_child = node.children.as_ref().unwrap().last().unwrap().borrow_mut();

            if let LayoutNodeContent::TextLayoutNode(text_node) = &mut last_child.content {
                let last_box = text_node.css_text_boxes.last_mut().unwrap();
                if last_box.text.ends_with(" ") {
                    last_box.text = last_box.text.trim_end_matches(' ').to_string();
                }
            }
        }
    }
}


fn apply_inline_layout(node: &mut LayoutNode, style_context: &StyleContext, top_left_x: f32, top_left_y: f32, available_width: f32,
                       current_scroll_y: f32, font_context: &FontContext, force_full_layout: bool) {
    let mut cursor_x = top_left_x;
    let mut cursor_y = top_left_y;
    let mut max_width: f32 = 0.0;
    let mut max_height_of_line: f32 = 0.0;


    fix_inline_whitespace(node);

    for child in node.children.as_ref().unwrap() {

        if child.borrow().from_dom_node.is_some() {
            let child_borr = child.borrow();
            let dom_node = child_borr.from_dom_node.as_ref().unwrap().borrow();
            if dom_node.text.is_some() {
                let text = dom_node.text.as_ref().unwrap();
                if text.text_content.trim().is_empty() {
                    //we are ignoring whitespace only text elements, to implement the rule that whitespace outside elements is ignored
                    continue;
                }
            }
        }


        let only_update_block_vertical_position = false; //we can only do this if the parent is block layout, but in this case its inline. Inline might cause horizontally cascading changes.
        let space_left = available_width - (cursor_x - top_left_x);
        compute_layout_for_node(&child, style_context, cursor_x, cursor_y, font_context, current_scroll_y, only_update_block_vertical_position,
                                force_full_layout, space_left,
                                false //Note: we don't allow wrapping here, because we want to do the wrapping later here combined with other inline elements
        );

        let is_line_break = if let LayoutNodeContent::TextLayoutNode(text_node) = &RefCell::borrow(child).content {
            text_node.line_break
        } else {
            false
        };

        if is_line_break {
            let child_height;
            if cursor_x != top_left_x {
                cursor_x = top_left_x;
                cursor_y += max_height_of_line;
                child_height = max_height_of_line;
                max_height_of_line = 0.0;
            } else {

                let char_height = if let LayoutNodeContent::TextLayoutNode(text_node) = &RefCell::borrow(child).content {
                    //we take the height of an arbitrary character here, because whitespace may have no height in some fonts
                    let (_, char_height) = font_context.get_text_dimension(&String::from("X"), &text_node.font);
                    char_height
                } else {
                    panic!("Linebreak should always be a text node");
                };

                cursor_x = top_left_x;
                cursor_y += char_height;
                child_height = char_height;
                max_height_of_line = 0.0;
            }

            RefCell::borrow_mut(child).update_css_box(CssBox { x: cursor_x, y: cursor_y, width: 0.0, height: child_height });
            continue;
        }

        if let LayoutNodeContent::TextLayoutNode(ref mut text_node) = RefCell::borrow_mut(child).content {
            //TODO: are there every splits to undo here? I think we just made a single box above (in the function setting the size of the element)
            text_node.undo_split_boxes();
        }

        let child_borrow = RefCell::borrow(child);
        let (_, _, child_width, child_height) = child_borrow.get_bounding_box();

        if (cursor_x - top_left_x + child_width) > available_width {

            if child_borrow.children.is_none() && child_borrow.can_wrap() {
                // in this case, we might be able to split the css boxes, and put part of the node on this line

                let mut new_css_text_boxes;
                let css_text_box_backup;

                match &child_borrow.content {
                    LayoutNodeContent::TextLayoutNode(text_layout_node) => {
                        let relative_cursor_x = cursor_x - top_left_x;
                        let amount_of_space_left_on_line = available_width - relative_cursor_x;
                        let wrapped_text = wrap_text(text_layout_node.css_text_boxes.last().unwrap(), &text_layout_node.non_breaking_space_positions,
                                                     available_width, amount_of_space_left_on_line);

                        new_css_text_boxes = Vec::new();
                        for text in wrapped_text {

                            let mut new_css_text_box = CssTextBox {
                                css_box: CssBox::empty(),
                                selection_rect: None,
                                selection_char_range: None,
                                char_position_mapping: font_context.compute_char_position_mapping(&text_layout_node.font, &text),
                                text: text,
                            };

                            let (rect_width, rect_height) = font_context.get_text_dimension(&new_css_text_box.text, &text_layout_node.font);

                            if cursor_x - top_left_x + rect_width > available_width {
                                if cursor_x != top_left_x {
                                    cursor_x = top_left_x;
                                    cursor_y += max_height_of_line;
                                    max_height_of_line = 0.0;
                                }
                            }

                            new_css_text_box.css_box = CssBox { x: cursor_x, y: cursor_y, width: rect_width, height: rect_height };
                            new_css_text_boxes.push(new_css_text_box);

                            cursor_x += rect_width;
                            max_width = max_width.max(cursor_x);
                            max_height_of_line = max_height_of_line.max(rect_height);

                        }

                        css_text_box_backup = Some(text_layout_node.css_text_boxes.iter().next().unwrap().clone());
                    },
                    _ => {
                        //We can only get here for nodes that can't wrap, but we checked that we can wrap already
                        panic!("Invalid state");
                    }
                }

                drop(child_borrow);

                match &mut RefCell::borrow_mut(child).content {
                    LayoutNodeContent::TextLayoutNode(ref mut text_layout_node) => {
                        text_layout_node.pre_wrap_box_backup = css_text_box_backup;
                        text_layout_node.css_text_boxes = new_css_text_boxes;
                    },
                    _ => {
                        //We can only get here for nodes that can't wrap, but we checked that we can wrap already
                        panic!("Invalid state");
                    }
                }

            } else {
                if cursor_x != top_left_x {
                    //we can move to a new line, it might fit there

                    cursor_x = top_left_x;
                    cursor_y += max_height_of_line;

                    let only_update_block_vertical_position = false; // We can only do this if the parent is block layout, but in this case its inline.
                                                                     // Inline might cause horizonal cascading changes.
                    drop(child_borrow);
                    compute_layout_for_node(&child, style_context, cursor_x, cursor_y, font_context, current_scroll_y,
                                            only_update_block_vertical_position, force_full_layout, available_width, false);
                    let (_, _, child_width, child_height) = RefCell::borrow(child).get_bounding_box();

                    cursor_x += child_width;
                    max_width = max_width.max(cursor_x);
                    max_height_of_line = child_height;

                } else {
                    //we already are on a new line, we just put it here
                    cursor_x += child_width;
                    max_width = max_width.max(cursor_x);
                    max_height_of_line = max_height_of_line.max(child_height);
                }

            }

        } else {
            // we append the items to the current line because it fits

            cursor_x += child_width;
            max_width = max_width.max(cursor_x);
            max_height_of_line = max_height_of_line.max(child_height);
        }

    }
    let our_height = (cursor_y - top_left_y) + max_height_of_line;
    node.update_css_box(CssBox { x: top_left_x, y: top_left_y, width: max_width, height: our_height });
}


fn wrap_text(css_text_box: &CssTextBox, non_breaking_space_positions: &Option<HashSet<usize>>, max_width: f32, width_remaining_on_current_line: f32) -> Vec<String> {
    let char_positions = &css_text_box.char_position_mapping;

    let mut lines: Vec<String> = Vec::new();
    let mut current_line_buffer = String::new();
    let mut undecided_buffer = String::new();
    let mut consumed_size = 0.0;
    let mut last_decided_idx = 0;

    for (idx, character) in css_text_box.text.chars().enumerate() {
        let width_to_check = if lines.len() == 0 { width_remaining_on_current_line } else { max_width };

        undecided_buffer.push(character);

        let potential_line_length = char_positions[idx] - consumed_size;
        if potential_line_length > width_to_check {
            lines.push(current_line_buffer);
            current_line_buffer = String::new();
            consumed_size = char_positions[last_decided_idx];
        }

        let wrapping_blocked = non_breaking_space_positions.is_some() && non_breaking_space_positions.as_ref().unwrap().contains(&idx);
        if !wrapping_blocked && character.is_whitespace() {
            current_line_buffer.push_str(undecided_buffer.as_str());
            undecided_buffer = String::new();
            last_decided_idx = idx;
        }
    }

    if !undecided_buffer.is_empty() {
        let potential_line_length = char_positions.last().unwrap() - consumed_size;
        let width_to_check = if lines.len() == 0 { width_remaining_on_current_line } else { max_width };
        if potential_line_length > width_to_check {
            lines.push(current_line_buffer);
            current_line_buffer = String::new();
        }
        current_line_buffer.push_str(undecided_buffer.as_str());
    }

    if !current_line_buffer.is_empty() {
        lines.push(current_line_buffer);
    }

    return lines;
}


fn build_layout_tree(main_node: &Rc<RefCell<ElementDomNode>>, document: &Document, font_context: &FontContext,
                     formatting_context: FormattingContext) -> Rc<RefCell<LayoutNode>> {
    let mut partial_node_visible = true;
    let mut partial_node_optional_img = None;
    let mut partial_node_line_break = false;
    let mut partial_node_children = None;
    let mut partial_node_is_submit_button = false;
    let mut partial_node_is_text_input = false;
    let mut partial_node_text = None;
    let mut partial_node_font = None;
    let mut partial_node_font_color = None;
    let mut partial_node_non_breaking_space_positions = None;
    let mut partial_formatting_context = formatting_context;

    let mut prebuilt_node = None; //TODO: I think it is a good idea to transition all cases to pre built the node? needs checking

    let node_styles = resolve_full_styles_for_layout_node(&Rc::clone(main_node), &document.all_nodes, &document.style_context);
    let partial_node_background_color = get_color_style_value(&node_styles, "background-color").unwrap_or(Color::WHITE);

    let positioning_scheme = PositioningScheme::Static; //TODO: this should be derived from the position attribute on the DOM node

    let mut childs_to_recurse_on: &Option<Vec<Rc<RefCell<ElementDomNode>>>> = &None;

    let main_node_refcell = main_node;
    let main_node = RefCell::borrow(main_node);

    if main_node.text.is_some() {
        partial_node_text = Some(main_node.text.as_ref().unwrap().text_content.clone());
        let font = get_font_given_styles(&node_styles);
        partial_node_font = Some(font.0);
        partial_node_font_color = Some(font.1);
        partial_node_non_breaking_space_positions = main_node.text.as_ref().unwrap().non_breaking_space_positions.clone();

    } else if main_node.name.is_some() {
        childs_to_recurse_on = &main_node.children;

        match &main_node.name_for_layout {
            TagName::Br => {
                //A newline does not have text, but we still want to make a text node, since things like fontsize affect how it looks
                partial_node_text = Some(String::new());

                partial_node_line_break = true;

                let font = get_font_given_styles(&node_styles);
                partial_node_font = Some(font.0);
                partial_node_font_color = Some(font.1);
            }

            TagName::Img => {
                if main_node.image.is_some() {
                    //TODO: eventually it would be nice to point in some cache of resources somewhere (possibly indirectly via an id if
                    //      ownership causes issues). For now we just clone every time we built the layout node.
                    partial_node_optional_img = Some(main_node.image.as_ref().unwrap().deref().clone());
                }
                childs_to_recurse_on = &None; //images should not have children (its a tag that does not have a close tag, formally)
            }

            TagName::Input => {
                let input_type = main_node.get_attribute_value("type");

                //TODO: we should not check type attribute here, the dom node already has either a textfield or a button on it

                if input_type.is_none() || input_type.as_ref().unwrap() == "text" {
                    partial_node_is_text_input = true;
                } else if input_type.is_some() && input_type.as_ref().unwrap() == "submit" {
                    partial_node_is_submit_button = true;
                } else {
                    debug_log_warn(format!("Unknown type of input element: {}", input_type.unwrap()));
                }
            }

            //TODO: this one might not be neccesary any more after we fix our html parser to not try to parse the javascript
            TagName::Script => { partial_node_visible = false; }

            //TODO: same as for "script", do these need nodes in the DOM? probably not
            TagName::Style => { partial_node_visible = false; }

            //TODO: eventually we want to do something else with the title (update the window title or so)
            TagName::Title => { partial_node_visible = false; }

            TagName::Table => {
                childs_to_recurse_on = &None; // we handle the children in our own method //TODO: it would still be nice to re-use the block/inline logic below
                drop(main_node);
                prebuilt_node = Some(tables::build_layout_tree_for_table(main_node_refcell, document, font_context));
            }

            TagName::Other => {}
        }
    } else if main_node.is_document_node {
        childs_to_recurse_on = &main_node.children;
    }

    let has_mixed_inline_and_block = {
        let mut has_mixed_inline_and_block = false;

        if childs_to_recurse_on.is_some() {
            let mut block_seen = false;
            let mut inline_seen = false;

            for child in childs_to_recurse_on.as_ref().unwrap() {
                match &child.borrow().dom_property_display() {
                    DomPropertyDisplay::Block => {
                        if inline_seen {
                            has_mixed_inline_and_block = true;
                            break
                        }
                        block_seen = true;
                    },
                    DomPropertyDisplay::Inline => {
                        if block_seen {
                            has_mixed_inline_and_block = true;
                            break
                        }
                        inline_seen = true;
                    },
                    DomPropertyDisplay::None => {},
                }
            }
        }

        has_mixed_inline_and_block
    };

    if childs_to_recurse_on.is_some() && childs_to_recurse_on.as_ref().unwrap().len() > 0 {
        partial_node_children = Some(Vec::new());
        let first_child = childs_to_recurse_on.as_ref().unwrap().iter().next().unwrap();

        if has_mixed_inline_and_block {
            let mut temp_inline_child_buffer = Vec::new();
            let background_color = partial_node_background_color;
            partial_formatting_context = FormattingContext::Block;

            for child in childs_to_recurse_on.as_ref().unwrap() {

                if child.borrow().dom_property_display() == DomPropertyDisplay::Block {
                    if !temp_inline_child_buffer.is_empty() {

                        let mut layout_childs = Vec::new();
                        for &node in temp_inline_child_buffer.iter() {
                            let layout_child = build_layout_tree(node, document, font_context, FormattingContext::Inline);
                            layout_childs.push(layout_child);
                        }

                        let anon_block = build_anonymous_layout_node(true, layout_childs, background_color, FormattingContext::Inline);
                        partial_node_children.as_mut().unwrap().push(anon_block);

                        temp_inline_child_buffer = Vec::new();
                    }

                    let layout_child = build_layout_tree(child, document, font_context, partial_formatting_context);
                    partial_node_children.as_mut().unwrap().push(layout_child);

                } else {
                    temp_inline_child_buffer.push(child);
                }
            }

            if !temp_inline_child_buffer.is_empty() {
                let mut layout_childs = Vec::new();
                for node in temp_inline_child_buffer.iter() {
                    let layout_child = build_layout_tree(node, document, font_context, FormattingContext::Inline);
                    layout_childs.push(layout_child);
                }

                let anon_block = build_anonymous_layout_node(true, layout_childs, background_color, FormattingContext::Inline);
                partial_node_children.as_mut().unwrap().push(anon_block);
            }

        } else if first_child.borrow().dom_property_display() == DomPropertyDisplay::Inline {

            partial_formatting_context = FormattingContext::Inline;

            for child in childs_to_recurse_on.as_ref().unwrap() {
                let layout_child = build_layout_tree(child, document, font_context, partial_formatting_context);
                partial_node_children.as_mut().unwrap().push(layout_child);
            }

        } else { //This means the children have all display = block   //TODO: this is not true, there might be Disply: None in here as well, which also goes
                                                                      //      for the above cases, structure needs to be a bit different probably

            partial_formatting_context = FormattingContext::Block;

            for child in childs_to_recurse_on.as_ref().unwrap() {
                let layout_child = build_layout_tree(child, document, font_context, partial_formatting_context);
                partial_node_children.as_mut().unwrap().push(layout_child);
            }
        }

    }

    if prebuilt_node.is_some() {
        //TODO: we could just return this prebuilt_node everywhere we build it, but I want to investigate what to do with the inline/block child logic in between
        return Rc::new(RefCell::from(prebuilt_node.unwrap()));
    }

    let content = if partial_node_text.is_some() {
        let css_text_box = CssTextBox {
            css_box:  CssBox::empty(),
            char_position_mapping: font_context.compute_char_position_mapping(&partial_node_font.as_ref().unwrap(), &partial_node_text.as_ref().unwrap()),
            text: partial_node_text.unwrap(),
            selection_rect: None,
            selection_char_range: None,
        };

        let text_node = TextLayoutNode {
            line_break: partial_node_line_break,
            background_color: partial_node_background_color,
            css_text_boxes: vec![css_text_box],
            pre_wrap_box_backup: None,
            font: partial_node_font.unwrap(),
            font_color: partial_node_font_color.unwrap(),
            non_breaking_space_positions: partial_node_non_breaking_space_positions,
        };
        LayoutNodeContent::TextLayoutNode(text_node)

    } else if partial_node_optional_img.is_some() {
        let img_node = ImageLayoutNode { image: partial_node_optional_img.unwrap(), css_box: CssBox::empty() };
        LayoutNodeContent::ImageLayoutNode(img_node)

    } else if partial_node_is_submit_button {
        LayoutNodeContent::ButtonLayoutNode(ButtonLayoutNode { css_box: CssBox::empty() })

    } else if partial_node_is_text_input {
        LayoutNodeContent::TextInputLayoutNode(TextInputLayoutNode { css_box: CssBox::empty() })

    } else {
        LayoutNodeContent::AreaLayoutNode(AreaLayoutNode { css_box: CssBox::empty(), background_color: partial_node_background_color })
    };

    let new_node = LayoutNode {
        internal_id: get_next_layout_node_interal_id(),
        formatting_context: partial_formatting_context,
        visible: partial_node_visible,
        children: partial_node_children,
        from_dom_node: Some(Rc::clone(&main_node_refcell)),
        content: content,
        positioning_scheme,
    };

    return Rc::new(RefCell::from(new_node));
}


pub fn rebuild_dirty_layout_childs(main_node: &Rc<RefCell<LayoutNode>>, document: &Document, font_context: &FontContext) {
    let mut main_node_mut = RefCell::borrow_mut(main_node);
    let main_node_formatting_context = main_node_mut.formatting_context;
    let main_node_children = &mut main_node_mut.children;

    if main_node_children.is_some() {
        for child_idx in 0..main_node_children.as_ref().unwrap().len() {
            let child = &main_node_children.as_ref().unwrap()[child_idx];

            if child.borrow().from_dom_node.is_some() && child.borrow().from_dom_node.as_ref().unwrap().borrow().dirty {
                let new_child = build_layout_tree(&child.borrow().from_dom_node.as_ref().unwrap(), document, font_context, main_node_formatting_context);
                main_node_children.as_mut().unwrap()[child_idx] = new_child;

            } else {
                rebuild_dirty_layout_childs(&child, document, font_context);
            }

        }
    }
}


fn build_anonymous_layout_node(visible: bool, inline_children: Vec<Rc<RefCell<LayoutNode>>>, background_color: Color, formatting_context: FormattingContext) -> Rc<RefCell<LayoutNode>> {
    let id_of_node_being_built = get_next_layout_node_interal_id();

    let empty_box_layout_node = AreaLayoutNode {
        css_box: CssBox::empty(),
        background_color,
    };

    let anonymous_node = LayoutNode {
        internal_id: id_of_node_being_built,
        formatting_context: formatting_context,
        visible: visible,
        children: Some(inline_children),
        from_dom_node: None,
        content: LayoutNodeContent::AreaLayoutNode(empty_box_layout_node),
        positioning_scheme: PositioningScheme::Static,
    };

    return Rc::new(RefCell::from(anonymous_node));
}
