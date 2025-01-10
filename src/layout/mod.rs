use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use image::DynamicImage;

use crate::color::Color;
use crate::debug::debug_log_warn;
use crate::dom::{
    Document,
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
use crate::SCREEN_HEIGHT;
use crate::style::{
    get_color_style_value,
    get_property_from_computed_styles,
    has_style_value,
    resolve_css_numeric_type_value,
    resolve_full_styles_for_layout_node,
    StyleContext,
};
use crate::ui::CONTENT_WIDTH;


#[cfg(test)] mod tests;


static NEXT_LAYOUT_NODE_INTERNAL: AtomicUsize = AtomicUsize::new(1);
pub fn get_next_layout_node_interal_id() -> usize { NEXT_LAYOUT_NODE_INTERNAL.fetch_add(1, Ordering::Relaxed) }


pub struct FullLayout {
    pub root_node: Rc<RefCell<LayoutNode>>,
    pub nodes_in_selection_order: Vec<Rc<RefCell<LayoutNode>>>,
}
impl FullLayout {
    pub fn page_height(&self) -> f32 {
        let node = RefCell::borrow(&self.root_node);
        match &node.content {
            LayoutNodeContent::BoxLayoutNode(box_node) => {
                return box_node.location.height;
            },
            _ => { panic!("Root node always should be a box layout node"); }
        }
    }
    pub fn new_empty() -> FullLayout {
        //Note that we we create a 1x1 rect even for an empty layout, since we need a rect to render it (for example when the first page is still loading)

        let box_node = BoxLayoutNode {
            location: Rect { x: 0.0, y: 0.0, width: 1.0, height: 1.0 },
            background_color: Color::BLACK,
        };

        let mut layout_node = LayoutNode::new_empty();
        layout_node.content = LayoutNodeContent::BoxLayoutNode(box_node);

        return FullLayout { root_node: Rc::from(RefCell::from(layout_node)), nodes_in_selection_order: Vec::new() };
    }
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct TextLayoutNode {
    pub line_break: bool,  //TODO: we should not need this. We just need an empty rect, or non layout node at all (as long as we generate the next text lower when layouting)
    pub rects: Vec<TextLayoutRect>,
    pub pre_wrap_rect_backup: Option<TextLayoutRect>,
    pub background_color: Color,
}

impl TextLayoutNode {
    pub fn undo_split_rects(&mut self) {
        //The main intention for this method is to be used before we start the process of computing line wrapping again (to undo the previous wrapping)

        if self.rects.len() > 1 {
            self.rects = vec![self.pre_wrap_rect_backup.as_ref().unwrap().clone()];
        }
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct ImageLayoutNode {
    pub image: DynamicImage,
    pub location: Rect,
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct ButtonLayoutNode {
    pub location: Rect,
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct TextInputLayoutNode {
    pub location: Rect,
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct BoxLayoutNode {
    pub location: Rect,
    #[allow(dead_code)] pub background_color: Color,  //TODO: use
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct TableLayoutNode {
    pub location: Rect,
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct TableCellLayoutNode {
    pub location: Rect,
    pub slot_x: usize,
    pub slot_y: usize,
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub enum LayoutNodeContent {
    TextLayoutNode(TextLayoutNode),
    ImageLayoutNode(ImageLayoutNode),
    ButtonLayoutNode(ButtonLayoutNode),
    TextInputLayoutNode(TextInputLayoutNode),
    BoxLayoutNode(BoxLayoutNode),
    TableLayoutNode(TableLayoutNode),
    TableCellLayoutNode(TableCellLayoutNode),
    NoContent,
}
impl LayoutNodeContent {
    pub fn is_inside(&self, x: f32, y: f32) -> bool {
        match self {
            LayoutNodeContent::TextLayoutNode(text_layout_node) => {
                for rect in text_layout_node.rects.iter() {
                    if rect.location.is_inside(x, y) { return true; }
                }
                return false;
            },
            LayoutNodeContent::ImageLayoutNode(image_node) => {
                return image_node.location.is_inside(x, y);
            }
            LayoutNodeContent::BoxLayoutNode(box_node) => {
                return box_node.location.is_inside(x, y);
            }
            LayoutNodeContent::ButtonLayoutNode(button_node) => {
                return button_node.location.is_inside(x, y);
            }
            LayoutNodeContent::TextInputLayoutNode(text_input_node) => {
                return text_input_node.location.is_inside(x, y);
            }
            LayoutNodeContent::TableLayoutNode(_) => {
                todo!(); //TODO: implement
            },
            LayoutNodeContent::TableCellLayoutNode(_) => {
                todo!(); //TODO: implement
            }
            LayoutNodeContent::NoContent => { return false; },
        }
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct LayoutNode {
    pub internal_id: usize,
    pub children: Option<Vec<Rc<RefCell<LayoutNode>>>>,

    pub from_dom_node: Option<Rc<RefCell<ElementDomNode>>>,

    pub display: Display,
    pub visible: bool,

    pub content: LayoutNodeContent,
}
impl LayoutNode {
    pub fn all_childnodes_have_given_display(&self, display: Display) -> bool {
        if self.children.is_none() {
            return true;
        }
        return self.children.as_ref().unwrap().iter().all(|node| RefCell::borrow(node).display == display);
    }

    pub fn update_single_rect_location(&mut self, new_location: Rect) {
        match &mut self.content {
            LayoutNodeContent::TextLayoutNode(node) => {
                debug_assert!(node.rects.len() == 1);
                node.rects[0].location = new_location;
            },
            LayoutNodeContent::ImageLayoutNode(node) => { node.location = new_location; },
            LayoutNodeContent::ButtonLayoutNode(node) => { node.location = new_location; },
            LayoutNodeContent::TextInputLayoutNode(node) => { node.location = new_location; },
            LayoutNodeContent::BoxLayoutNode(node) => { node.location = new_location; },
            LayoutNodeContent::TableLayoutNode(node) => { node.location = new_location; }
            LayoutNodeContent::TableCellLayoutNode(node) => { node.location = new_location; }
            LayoutNodeContent::NoContent => { }
        }
    }

    pub fn can_wrap(&self) -> bool {
        return if let LayoutNodeContent::TextLayoutNode(_) = self.content { true } else { false };
    }

    pub fn y_position(&self) -> f32 {
        return match &self.content {
            LayoutNodeContent::TextLayoutNode(text_layout_node) => { text_layout_node.rects.iter().next().unwrap().location.y },
            LayoutNodeContent::ImageLayoutNode(image_node) => { image_node.location.y }
            LayoutNodeContent::ButtonLayoutNode(button_node) => { button_node.location.y }
            LayoutNodeContent::TextInputLayoutNode(text_input_node) => { text_input_node.location.y }
            LayoutNodeContent::BoxLayoutNode(box_node) => { box_node.location.y }
            LayoutNodeContent::TableLayoutNode(table_node) => { table_node.location.y }
            LayoutNodeContent::TableCellLayoutNode(cell_node) => { cell_node.location.y }
            LayoutNodeContent::NoContent => { panic!("can't get a position of something without content") },
        }
    }

    pub fn get_size_of_bounding_box(&self) -> (f32, f32) {

        match &self.content {
            LayoutNodeContent::TextLayoutNode(text_node) => {
                let mut lowest_x = f32::MAX;
                let mut lowest_y = f32::MAX;
                let mut max_x: f32 = 0.0;
                let mut max_y: f32 = 0.0;

                for rect in text_node.rects.iter() {
                    lowest_x = lowest_x.min(rect.location.x);
                    lowest_y = lowest_y.min(rect.location.y);
                    max_x = max_x.max(rect.location.x + rect.location.width);
                    max_y = max_y.max(rect.location.y + rect.location.height);
                }

                let bounding_box_width = max_x - lowest_x;
                let bounding_box_height = max_y - lowest_y;
                return (bounding_box_width, bounding_box_height);
            },
            LayoutNodeContent::ImageLayoutNode(img_node) => { return (img_node.location.width, img_node.location.height); },
            LayoutNodeContent::ButtonLayoutNode(button_node)  => { return (button_node.location.width, button_node.location.height); },
            LayoutNodeContent::TextInputLayoutNode(input_node) => { return (input_node.location.width, input_node.location.height); },
            LayoutNodeContent::BoxLayoutNode(box_node) => { return (box_node.location.width, box_node.location.height); },
            LayoutNodeContent::TableLayoutNode(table_node) => { return (table_node.location.width, table_node.location.height); }
            LayoutNodeContent::TableCellLayoutNode(cell_node) => { return (cell_node.location.width, cell_node.location.height); }
            LayoutNodeContent::NoContent => { panic!("invalid state") },
        }
    }

    pub fn visible_on_y_location(&self, current_scroll_y: f32) -> bool {
        match &self.content {
            LayoutNodeContent::TextLayoutNode(text_node) => {
                return text_node.rects.iter().any(|rect| -> bool {rect.location.is_visible_on_y_location(current_scroll_y)});
            },
            LayoutNodeContent::ImageLayoutNode(image_node) => { return image_node.location.is_visible_on_y_location(current_scroll_y); },
            LayoutNodeContent::ButtonLayoutNode(button_node) => { return button_node.location.is_visible_on_y_location(current_scroll_y); }
            LayoutNodeContent::TextInputLayoutNode(text_input_node) => { return text_input_node.location.is_visible_on_y_location(current_scroll_y); }
            LayoutNodeContent::BoxLayoutNode(box_node) => { return box_node.location.is_visible_on_y_location(current_scroll_y); },
            LayoutNodeContent::TableLayoutNode(table_node) => { return table_node.location.is_visible_on_y_location(current_scroll_y); }
            LayoutNodeContent::TableCellLayoutNode(cell_node) => { return cell_node.location.is_visible_on_y_location(current_scroll_y); }
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
            display: Display::Block,
            visible: true,
            children: None,
            from_dom_node: None,
            content: LayoutNodeContent::NoContent,
        };
    }

    pub fn reset_selection(&mut self) {
        match self.content {
            LayoutNodeContent::TextLayoutNode(ref mut text_layout_node) => {
                for rect in text_layout_node.rects.iter_mut() {
                    rect.selection_rect = None;
                    rect.selection_char_range = None;
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
            LayoutNodeContent::BoxLayoutNode(_) => {
                //Note: this is a no-op for now, since there is nothing to select in a box node itself (just in its children)
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
                for rect in &text_layout_node.rects {
                    if rect.selection_char_range.is_some() {
                        let (start_idx, end_idx) = rect.selection_char_range.unwrap();
                        result.push_str(rect.text.chars().skip(start_idx).take(end_idx - start_idx + 1).collect::<String>().as_str());
                    }
                }
            },
            LayoutNodeContent::ImageLayoutNode(_) => todo!(),  //TODO: implement
            LayoutNodeContent::ButtonLayoutNode(_) => todo!(),  //TODO: implement
            LayoutNodeContent::TextInputLayoutNode(_) => todo!(),  //TODO: implement
            LayoutNodeContent::TableLayoutNode(_) => todo!(),  //TODO: implement
            LayoutNodeContent::TableCellLayoutNode(_) => todo!(),  //TODO: implement
            LayoutNodeContent::BoxLayoutNode(_) => {},
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
                for rect in text_layout_node.rects.iter_mut() {
                    rect.location.y += y_diff;
                }
            },
            LayoutNodeContent::ImageLayoutNode(image_node) => { image_node.location.y += y_diff; }
            LayoutNodeContent::ButtonLayoutNode(button_node) => { button_node.location.y += y_diff; }
            LayoutNodeContent::TextInputLayoutNode(text_input_node) => { text_input_node.location.y += y_diff; }
            LayoutNodeContent::BoxLayoutNode(box_node) => { box_node.location.y += y_diff; }
            LayoutNodeContent::TableLayoutNode(table_node) => { table_node.location.y += y_diff; }
            LayoutNodeContent::TableCellLayoutNode(table_cell_node) => { table_cell_node.location.y += y_diff; }
            LayoutNodeContent::NoContent => { panic!("Cant adjust position of something without content"); }
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
#[derive(Clone)]
pub struct TextLayoutRect {
    pub location: Rect,
    pub text: String,
    pub font: Font,
    pub font_color: Color,
    pub char_position_mapping: Vec<f32>,
    pub non_breaking_space_positions: Option<HashSet<usize>>,
    pub selection_rect: Option<Rect>,
    pub selection_char_range: Option<(usize, usize)>,
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(PartialEq)]
pub enum Display { //TODO: this is a CSS property, of which we will have many, we should probably define those somewhere else
    Block,
    Inline
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
impl Rect {
    pub fn is_inside(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width
        &&
        y >= self.y && y <= self.y + self.height
    }
    pub fn empty() -> Rect {
        return Rect { x: 0.0, y: 0.0, width: 0.0, height: 0.0 };
    }
    pub fn is_visible_on_y_location(&self, y: f32) -> bool {
        let top_of_node = self.y;
        let top_of_view = y;
        let bottom_of_node = top_of_node + self.height;
        let bottom_of_view = top_of_view + SCREEN_HEIGHT;

        return !(top_of_node > bottom_of_view || bottom_of_node < top_of_view);
    }
}


struct LayoutBuildState {
    last_char_was_space: bool,
}


pub fn build_full_layout(document: &Document, font_context: &FontContext) -> FullLayout {
    let mut top_level_layout_nodes: Vec<Rc<RefCell<LayoutNode>>> = Vec::new();

    let id_of_node_being_built = get_next_layout_node_interal_id();

    let mut state = LayoutBuildState { last_char_was_space: false };

    let layout_node = build_layout_tree(&document.document_node, document, font_context, &mut state, None);
    top_level_layout_nodes.push(layout_node);

    //Note: we need a node above the first node actually containing any content or styles, since for updates to content or styles we re-assign
    //      children to the parent, so we need all nodes that could update to have a valid parent. That is this root_node for the toplevel node(s).
    let root_node = LayoutNode {
        internal_id: id_of_node_being_built,
        display: Display::Block,
        visible: true,
        children: Some(top_level_layout_nodes),
        from_dom_node: None,
        content: LayoutNodeContent::BoxLayoutNode(BoxLayoutNode {
            location: Rect::empty(),
            background_color: Color::WHITE,
        }),
    };

    let rc_root_node = Rc::new(RefCell::from(root_node));

    let mut nodes_in_selection_order = Vec::new();
    collect_content_nodes_in_walk_order(&rc_root_node, &mut nodes_in_selection_order);

    return FullLayout { root_node: rc_root_node, nodes_in_selection_order };
}


pub fn collect_content_nodes_in_walk_order(node: &Rc<RefCell<LayoutNode>>, result: &mut Vec<Rc<RefCell<LayoutNode>>>) {
    //TODO: this is not correct, at least, not if we are using it for things like selection. Because absolutely positioned elements might have
    //      very different positions, regardless of their place in the tree. We need to base this on all (x, y) postions (and keep that updated)

    match RefCell::borrow(node).content {
        LayoutNodeContent::TextLayoutNode(_) => { result.push(Rc::clone(&node)); },
        LayoutNodeContent::ImageLayoutNode(_) => { result.push(Rc::clone(&node)); },
        LayoutNodeContent::ButtonLayoutNode(_) => { result.push(Rc::clone(&node)); },
        LayoutNodeContent::TextInputLayoutNode(_) => { result.push(Rc::clone(&node)); },
        LayoutNodeContent::BoxLayoutNode(_) => {},
        LayoutNodeContent::TableLayoutNode(_) => {},
        LayoutNodeContent::TableCellLayoutNode(_) => { result.push(Rc::clone(&node)); },
        LayoutNodeContent::NoContent => {},
    }

    if RefCell::borrow(node).children.as_ref().is_some() {
        for child in RefCell::borrow(node).children.as_ref().unwrap() {
            collect_content_nodes_in_walk_order(&child, result);
        }
    }
}


pub fn compute_layout(node: &Rc<RefCell<LayoutNode>>, style_context: &StyleContext, top_left_x: f32, top_left_y: f32, font_context: &FontContext,
                      current_scroll_y: f32, only_update_block_vertical_position: bool, force_full_layout: bool) {
    compute_layout_for_node(node, style_context, top_left_x, top_left_y, font_context, current_scroll_y, only_update_block_vertical_position, force_full_layout);

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

//This function is responsible for setting the location rects on the node, and all its children, and updating content if needed (sync with DOM)
//TODO: we now pass in top_left x and y, but I think we should compute the positions just for layout, and offset for UI in the render phase...
fn compute_layout_for_node(node: &Rc<RefCell<LayoutNode>>, style_context: &StyleContext, top_left_x: f32, top_left_y: f32, font_context: &FontContext,
                           current_scroll_y: f32, only_update_block_vertical_position: bool, force_full_layout: bool) {

    let mut mut_node = RefCell::borrow_mut(node);

    if only_update_block_vertical_position && !force_full_layout {
        let y_diff = top_left_y - mut_node.y_position();
        mut_node.move_node_vertically(y_diff);
        return;
    }

    if !mut_node.visible {
        mut_node.update_single_rect_location(Rect { x: top_left_x, y: top_left_y, width: 0.0, height: 0.0 });

    } else if mut_node.children.is_some() {

        if let LayoutNodeContent::TableLayoutNode(_) = mut_node.content {
            todo!();
            //TODO: we need to so something here that is neither block nor inline, and go in a seperate method, that method should call this method recursively
        }

        if mut_node.all_childnodes_have_given_display(Display::Block) {
            apply_block_layout(&mut mut_node, style_context, top_left_x, top_left_y, current_scroll_y, font_context, force_full_layout);
        } else if mut_node.all_childnodes_have_given_display(Display::Inline) {
            apply_inline_layout(&mut mut_node, style_context, top_left_x, top_left_y, CONTENT_WIDTH - top_left_x, current_scroll_y, font_context, force_full_layout);
        } else {
            panic!("Not all children are either inline or block, earlier in the process this should already have been fixed with anonymous blocks");
        }

    } else {

        let opt_dom_node = if mut_node.from_dom_node.is_some() {
            Some(Rc::clone(&mut_node.from_dom_node.as_ref().unwrap()))
        } else {
            None
        };

        match &mut mut_node.content {
            LayoutNodeContent::TextLayoutNode(ref mut text_layout_node) => {

                if opt_dom_node.is_some() && opt_dom_node.as_ref().unwrap().borrow().dirty {
                    text_layout_node.undo_split_rects();
                }

                for layout_rect in text_layout_node.rects.iter_mut() {
                    let (rect_width, rect_height) = font_context.get_text_dimension(&layout_rect.text, &layout_rect.font);
                    layout_rect.location = Rect { x: top_left_x, y: top_left_y, width: rect_width, height: rect_height };
                }
            },
            LayoutNodeContent::ImageLayoutNode(image_layout_node) => {
                image_layout_node.location =
                     Rect { x: top_left_x, y: top_left_y, width: image_layout_node.image.width() as f32, height: image_layout_node.image.height() as f32 };
            },
            LayoutNodeContent::ButtonLayoutNode(button_node) => {
                //TODO: for now we are setting a default size here, but that should actually retreived from the DOM
                let button_width = 100.0;  //TODO: this needs to be dependent on the text size. How do we do that? Compute it here?
                let button_height = 40.0;

                button_node.location = Rect { x: top_left_x, y: top_left_y, width: button_width, height: button_height };
                let mut_dom_node = mut_node.from_dom_node.as_ref().unwrap().borrow();
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

                text_input_node.location = Rect { x: top_left_x, y: top_left_y, width: field_width, height: field_height };
                let dom_node = mut_node.from_dom_node.as_ref().unwrap().borrow();
                let mut page_component = dom_node.page_component.as_ref().unwrap().borrow_mut();

                match page_component.deref_mut() {
                    PageComponent::Button(_) => { panic!("Invalid state"); },
                    PageComponent::TextField(text_field) => {
                        text_field.update_position(top_left_x, top_left_y - current_scroll_y, field_width, field_height);
                    }
                }
            },
            LayoutNodeContent::BoxLayoutNode(box_node) => {
                //Note: this is a boxlayoutnode, but without children (because that is a seperate case above), so no content.

                //TODO: for now generating 1 by 1 sized, this might not be correct given styling.
                box_node.location = Rect { x: top_left_x, y: top_left_y, width: 1.0, height: 1.0 };
            },
            LayoutNodeContent::NoContent => todo!(), //TODO: should we still compute a position? Maybe it is always 0 by 0 pixels?
            LayoutNodeContent::TableLayoutNode(table_layout_node) => {
                //This is a table without children, so it has no size (the case with children is handled above...)
                table_layout_node.location = Rect { x: top_left_x, y: top_left_y, width: 0.0, height: 0.0 };

            },
            LayoutNodeContent::TableCellLayoutNode(cell_layout_node) => {
                //This is the case where the cell has no children, which means no content, which means no size for rendering
                //(the position of other cells has already been computed when their parent was computed)
                cell_layout_node.location = Rect { x: top_left_x, y: top_left_y, width: 0.0, height: 0.0 };
            },
        }

    }
}


pub fn get_font_given_styles(styles: &HashMap<String, String>) -> (Font, Color) {
    let font_bold = has_style_value(&styles, "font-weight", &"bold".to_owned());
    let _font_underline = has_style_value(&styles, "text-decoration", &"underline".to_owned()); //TODO: we need to use this in a different way
    //TODO: we still need to parse italic (currently harcoded to false in the return below)
    let opt_font_size = get_property_from_computed_styles(&styles, "font-size");
    let font_size = resolve_css_numeric_type_value(&opt_font_size.unwrap()); //font-size has a default value, so this is a fatal error if not found

    let font_color_option = get_color_style_value(&styles, "color");
    let font_color = font_color_option.unwrap(); //color has a default value, so this is a fatal error if not found

    let default_font_face = FontFace::TimesNewRomanRegular;

    return (Font { face: default_font_face, bold: font_bold, italic: false, size: font_size as u16}, font_color);
}


fn apply_block_layout(node: &mut LayoutNode, style_context: &StyleContext, top_left_x: f32, top_left_y: f32,
                      current_scroll_y: f32, font_context: &FontContext, force_full_layout: bool) {
    let mut cursor_y = top_left_y;
    let mut max_width: f32 = 0.0;

    for child in node.children.as_ref().unwrap() {
        let only_update_block_vertical_position = !child.borrow().is_dirty_anywhere(); //Since the parent node is block layout, we can shift the while block up and down if its not dirty
        compute_layout_for_node(&child, style_context, top_left_x, cursor_y, font_context, current_scroll_y, only_update_block_vertical_position, force_full_layout);
        let (bounding_box_width, bounding_box_height) = RefCell::borrow(child).get_size_of_bounding_box();

        cursor_y += bounding_box_height;
        max_width = max_width.max(bounding_box_width);
    }

    let our_height = cursor_y - top_left_y;
    node.update_single_rect_location(Rect { x: top_left_x, y: top_left_y, width: max_width, height: our_height });
}


fn apply_inline_layout(node: &mut LayoutNode, style_context: &StyleContext, top_left_x: f32, top_left_y: f32, max_allowed_width: f32,
                       current_scroll_y: f32, font_context: &FontContext, force_full_layout: bool) {
    let mut cursor_x = top_left_x;
    let mut cursor_y = top_left_y;
    let mut max_width: f32 = 0.0;
    let mut max_height_of_line: f32 = 0.0;

    for child in node.children.as_ref().unwrap() {
        let only_update_block_vertical_position = false; //we can only do this if the parent is block layout, but in this case its inline. Inline might cause horizonal cascading changes.
        compute_layout_for_node(&child, style_context, cursor_x, cursor_y, font_context, current_scroll_y, only_update_block_vertical_position, force_full_layout);

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
            } else {
                //TODO: we need to make the height of the newline dependent on the font size, but
                //we don't have the styles anymore. Should we just make sure the font is set for
                //newline as well? It seems we don't have a rect in that case?

                let random_char_height = 16.0; //TODO: temporary hardcoded value

                cursor_x = top_left_x;
                cursor_y += random_char_height;
                child_height = random_char_height;
            }

            RefCell::borrow_mut(child).update_single_rect_location(Rect { x: top_left_x, y: top_left_y, width: max_width, height: child_height });

            continue;
        }

        if let LayoutNodeContent::TextLayoutNode(ref mut text_node) = RefCell::borrow_mut(child).content {
            text_node.undo_split_rects();
        }

        let child_borrow = RefCell::borrow(child);
        let (child_width, child_height) = child_borrow.get_size_of_bounding_box();

        if (cursor_x - top_left_x + child_width) > max_allowed_width {

            if child_borrow.children.is_none() && child_borrow.can_wrap() {
                // in this case, we might be able to split rects, and put part of the node on this line

                let mut rects_for_child;
                let rect_backup;

                match &child_borrow.content {
                    LayoutNodeContent::TextLayoutNode(text_layout_node) => {
                        let first_rect = text_layout_node.rects.iter().next().unwrap();
                        let font_color = first_rect.font_color;
                        let relative_cursor_x = cursor_x - top_left_x;
                        let amount_of_space_left_on_line = max_allowed_width - relative_cursor_x;
                        let wrapped_text = wrap_text(text_layout_node.rects.last().unwrap(), max_allowed_width, amount_of_space_left_on_line);

                        rects_for_child = Some(Vec::new());
                        for text in wrapped_text {

                            let mut new_rect = TextLayoutRect {
                                location: Rect::empty(),
                                selection_rect: None,
                                selection_char_range: None,
                                font: first_rect.font.clone(),
                                font_color: font_color,
                                char_position_mapping: font_context.compute_char_position_mapping(&first_rect.font, &text),
                                non_breaking_space_positions: None, //For now not computing these, although it would be more correct to update them after wrapping
                                text: text,
                            };

                            let (rect_width, rect_height) = font_context.get_text_dimension(&new_rect.text, &new_rect.font);

                            if cursor_x - top_left_x + rect_width > max_allowed_width {
                                if cursor_x != top_left_x {
                                    cursor_x = top_left_x;
                                    cursor_y += max_height_of_line;
                                    max_height_of_line = 0.0;
                                }
                            }

                            new_rect.location = Rect { x: cursor_x, y: cursor_y, width: rect_width, height: rect_height };
                            rects_for_child.as_mut().unwrap().push(new_rect);

                            cursor_x += rect_width;
                            max_width = max_width.max(cursor_x);
                            max_height_of_line = max_height_of_line.max(rect_height);

                        }

                        rect_backup = Some(text_layout_node.rects.iter().next().unwrap().clone());
                    },
                    _ => {
                        //We can only get here for nodes that can't wrap, but we checked that we can wrap already
                        panic!("Invalid state");
                    }
                }

                drop(child_borrow);

                match &mut RefCell::borrow_mut(child).content {
                    LayoutNodeContent::TextLayoutNode(ref mut text_layout_node) => {
                        text_layout_node.pre_wrap_rect_backup = rect_backup;
                        text_layout_node.rects = rects_for_child.unwrap();
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

                    let only_update_block_vertical_position = false; //we can only do this if the parent is block layout, but in this case its inline. Inline might cause horizonal cascading changes.
                    drop(child_borrow);
                    compute_layout_for_node(&child, style_context, cursor_x, cursor_y, font_context, current_scroll_y, only_update_block_vertical_position, force_full_layout);
                    let (child_width, child_height) = RefCell::borrow(child).get_size_of_bounding_box();

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
    node.update_single_rect_location(Rect { x: top_left_x, y: top_left_y, width: max_width, height: our_height });
}


fn wrap_text(text_layout_rect: &TextLayoutRect, max_width: f32, width_remaining_on_current_line: f32) -> Vec<String> {
    let no_wrap_positions = &text_layout_rect.non_breaking_space_positions;
    let char_positions = &text_layout_rect.char_position_mapping;

    let mut lines: Vec<String> = Vec::new();
    let mut current_line_buffer = String::new();
    let mut undecided_buffer = String::new();
    let mut consumed_size = 0.0;
    let mut last_decided_idx = 0;

    for (idx, character) in text_layout_rect.text.chars().enumerate() {
        let width_to_check = if lines.len() == 0 { width_remaining_on_current_line } else { max_width };

        undecided_buffer.push(character);

        let potential_line_length = char_positions[idx] - consumed_size;
        if potential_line_length >= width_to_check {
            lines.push(current_line_buffer);
            current_line_buffer = String::new();
            consumed_size = char_positions[last_decided_idx];
        }

        let wrapping_blocked = no_wrap_positions.is_some() && no_wrap_positions.as_ref().unwrap().contains(&idx);
        if !wrapping_blocked && character.is_whitespace() {
            current_line_buffer.push_str(undecided_buffer.as_str());
            undecided_buffer = String::new();
            last_decided_idx = idx;
        }
    }

    if !undecided_buffer.is_empty() {
        let potential_line_length = char_positions.last().unwrap() - consumed_size;
        let width_to_check = if lines.len() == 0 { width_remaining_on_current_line } else { max_width };
        if potential_line_length >= width_to_check {
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


fn get_display_type(node: &Rc<RefCell<ElementDomNode>>) -> Display {
    //TODO: eventually this needs the resolved styles as well, because that can influence the display type...

    let node = RefCell::borrow(node);

    if node.name.is_some() {
        let node_name = node.name.as_ref().unwrap();

        if node_name == "a" ||  //TODO: should we check a static array of str here?
           node_name == "b" ||
           node_name == "br" ||
           node_name == "img" ||
           node_name == "span" {
                return Display::Inline;
        }
        return Display::Block;

    }
    if node.text.is_some() {
        return Display::Inline;
    }
    if node.is_document_node {
        return Display::Block;
    }

    panic!("Node type not recognized");
}


fn build_layout_tree(main_node: &Rc<RefCell<ElementDomNode>>, document: &Document, font_context: &FontContext, state: &mut LayoutBuildState,
                     optional_new_text: Option<String>) -> Rc<RefCell<LayoutNode>> {
    let mut partial_node_visible = true;
    let mut partial_node_optional_img = None;
    let mut partial_node_line_break = false;
    let mut partial_node_styles = resolve_full_styles_for_layout_node(&Rc::clone(main_node), &document.all_nodes, &document.style_context);
    let mut partial_node_children = None;
    let mut partial_node_is_submit_button = false;
    let mut partial_node_is_text_input = false;
    let mut partial_node_text = None;
    let mut partial_node_font = None;
    let mut partial_node_font_color = None;
    let mut partial_node_non_breaking_space_positions = None;

    let mut prebuilt_node = None; //TODO: I think it is a good idea to transition all cases to pre built the node? needs checking

    let partial_node_background_color = get_color_style_value(&partial_node_styles, "background-color").unwrap_or(Color::WHITE);

    let mut childs_to_recurse_on: &Option<Vec<Rc<RefCell<ElementDomNode>>>> = &None;

    let main_node_refcell = main_node;
    let main_node = RefCell::borrow(main_node);

    if main_node.text.is_some() {
        partial_node_text = if optional_new_text.is_some() {
            Some(optional_new_text.unwrap())
        } else {
            Some(main_node.text.as_ref().unwrap().text_content.clone())
        };

        let font = get_font_given_styles(&partial_node_styles);
        partial_node_font = Some(font.0);
        partial_node_font_color = Some(font.1);
        partial_node_non_breaking_space_positions = main_node.text.as_ref().unwrap().non_breaking_space_positions.clone();

    } else if main_node.name.is_some() {
        debug_assert!(optional_new_text.is_none());

        childs_to_recurse_on = &main_node.children;

        match &main_node.name_for_layout {

            TagName::B => {
                //TODO: can this style not be in the general stylesheet?
                partial_node_styles.insert("font-weight".to_owned(), "bold".to_owned());
            }

            TagName::Br => {
                //A newline does not have text, but we still want to make a text node, since things like fontsize affect how it looks
                partial_node_text = Some(String::new());

                partial_node_line_break = true;

                let font = get_font_given_styles(&partial_node_styles);
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
                prebuilt_node = Some(build_layout_tree_for_table(main_node_refcell));
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
                match get_display_type(&child) {
                    Display::Block => {
                        if inline_seen {
                            has_mixed_inline_and_block = true;
                            break
                        }
                        block_seen = true;
                    },
                    Display::Inline => {
                        if block_seen {
                            has_mixed_inline_and_block = true;
                            break
                        }
                        inline_seen = true;
                    },
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

            for child in childs_to_recurse_on.as_ref().unwrap() {

                if get_display_type(&child) == Display::Block {
                    if !temp_inline_child_buffer.is_empty() {
                        let layout_childs = build_layout_for_inline_nodes(&temp_inline_child_buffer, document, font_context, state);

                        let anon_block = build_anonymous_block_layout_node(true, layout_childs, background_color);
                        partial_node_children.as_mut().unwrap().push(anon_block);

                        temp_inline_child_buffer = Vec::new();
                    }

                    state.last_char_was_space = false;
                    let layout_child = build_layout_tree(child, document, font_context, state, None);
                    partial_node_children.as_mut().unwrap().push(layout_child);

                } else {
                    temp_inline_child_buffer.push(child);
                }
            }

            if !temp_inline_child_buffer.is_empty() {
                let layout_childs = build_layout_for_inline_nodes(&temp_inline_child_buffer, document, font_context, state);

                let anon_block = build_anonymous_block_layout_node(true, layout_childs, background_color);
                partial_node_children.as_mut().unwrap().push(anon_block);
            }

        } else if get_display_type(&first_child) == Display::Inline {

            let mut inline_nodes_to_layout = Vec::new();
            for child in childs_to_recurse_on.as_ref().unwrap() {
                inline_nodes_to_layout.push(child);
            }
            let layout_childs = build_layout_for_inline_nodes(&inline_nodes_to_layout, document, font_context, state);

            for layout_child in layout_childs {
                partial_node_children.as_mut().unwrap().push(layout_child);
            }

        } else { //This means all childs are Display::Block

            for child in childs_to_recurse_on.as_ref().unwrap() {
                state.last_char_was_space = false;
                let layout_child = build_layout_tree(child, document, font_context, state, None);
                partial_node_children.as_mut().unwrap().push(layout_child);
            }
        }

    }

    if prebuilt_node.is_some() {
        //TODO: we could just return this prebuilt_node everywhere we build it, but I want to investigate what to do with the inline/block child logic in between
        return Rc::new(RefCell::from(prebuilt_node.unwrap()));
    }

    let content = if partial_node_text.is_some() {
        let rect = TextLayoutRect {
            char_position_mapping: font_context.compute_char_position_mapping(&partial_node_font.as_ref().unwrap(), &partial_node_text.as_ref().unwrap()),
            non_breaking_space_positions: partial_node_non_breaking_space_positions,
            location: Rect::empty(),
            selection_rect: None,
            selection_char_range: None,
            text: partial_node_text.unwrap(),
            font: partial_node_font.unwrap(),
            font_color: partial_node_font_color.unwrap(),
        };

        let text_node = TextLayoutNode {
            line_break: partial_node_line_break,
            rects: vec![rect],
            pre_wrap_rect_backup: None,
            background_color: partial_node_background_color,
        };
        LayoutNodeContent::TextLayoutNode(text_node)

    } else if partial_node_optional_img.is_some() {
        let img_node = ImageLayoutNode { image: partial_node_optional_img.unwrap(), location: Rect::empty() };
        LayoutNodeContent::ImageLayoutNode(img_node)

    } else if partial_node_is_submit_button {
        LayoutNodeContent::ButtonLayoutNode(ButtonLayoutNode { location: Rect::empty() })

    } else if partial_node_is_text_input {
        LayoutNodeContent::TextInputLayoutNode(TextInputLayoutNode { location: Rect::empty() })

    } else {
        LayoutNodeContent::BoxLayoutNode(BoxLayoutNode { location: Rect::empty(), background_color: partial_node_background_color })
    };

    let new_node = LayoutNode {
        internal_id: get_next_layout_node_interal_id(),
        display: get_display_type(main_node_refcell),
        visible: partial_node_visible,
        children: partial_node_children,
        from_dom_node: Some(Rc::clone(&main_node_refcell)),
        content: content,
    };

    return Rc::new(RefCell::from(new_node));
}


fn build_layout_tree_for_table(dom_node: &Rc<RefCell<ElementDomNode>>) -> LayoutNode {

    //TODO: process all the <tr> and <td> and recurse into general layout tree build function for cell content

    return LayoutNode {
        internal_id: get_next_layout_node_interal_id(),
        children: None,
        from_dom_node: Some(dom_node.clone()),
        display: Display::Block,
        visible: true,
        content: LayoutNodeContent::TableLayoutNode(TableLayoutNode {
            location: Rect::empty(),
        })
    }

}


pub fn rebuild_dirty_layout_childs(main_node: &Rc<RefCell<LayoutNode>>, document: &Document, font_context: &FontContext) {
    let mut main_node_mut = RefCell::borrow_mut(main_node);
    let main_node_children = &mut main_node_mut.children;

    if main_node_children.is_some() {
        for child_idx in 0..main_node_children.as_ref().unwrap().len() {
            let child = &main_node_children.as_ref().unwrap()[child_idx];

            if child.borrow().from_dom_node.is_some() && child.borrow().from_dom_node.as_ref().unwrap().borrow().dirty {
                let mut layout_build_state = LayoutBuildState { last_char_was_space: false }; //TODO: is there ever a case where this needs to be not false?
                                                                                              //      maybe when replacing in a series of inline nodes?
                let new_child = build_layout_tree(&child.borrow().from_dom_node.as_ref().unwrap(), document, font_context, &mut layout_build_state, None);
                main_node_children.as_mut().unwrap()[child_idx] = new_child;

            } else {
                rebuild_dirty_layout_childs(&child, document, font_context);
            }

        }
    }
}


fn build_layout_for_inline_nodes(inline_nodes: &Vec<&Rc<RefCell<ElementDomNode>>>, document: &Document, font_context: &FontContext,
                                 state: &mut LayoutBuildState) -> Vec<Rc<RefCell<LayoutNode>>> {

    let mut optional_new_text;
    let mut layout_nodes = Vec::new();
    let last_node_idx = inline_nodes.len();

    for (node_idx, node) in inline_nodes.iter().enumerate() {

        if RefCell::borrow(node).text.is_some() {
            let node = RefCell::borrow(node);

            let node_text = &node.text.as_ref().unwrap().text_content;
            let mut new_text = String::new();
            for (char_idx, c) in node_text.chars().enumerate() {
                if c == ' ' {

                    //TODO: is_on_edge_of_inline_context is not actually correct, I need to strip _all_ leading and trailing whitespace. I think it currently
                    //      works because of the way we build the DOM, but we should actually preserve all whitespace in there...
                    //      maybe I can just fix this by keeping on state whether I have seen a non-space already?, and the idx of the last non-space?
                    let is_on_edge_of_inline_context = (node_idx == 0 && char_idx == 0) || (node_idx == last_node_idx && char_idx == node_text.len());

                    if (!state.last_char_was_space) && (!is_on_edge_of_inline_context) {
                        new_text.push(c);
                    }
                    state.last_char_was_space = true;
                } else {
                    state.last_char_was_space = false;
                    new_text.push(c);
                }
            }

            optional_new_text = Some(new_text);
        } else {
            optional_new_text = None;
        }

        let layout_child = build_layout_tree(node, document, font_context, state, optional_new_text);
        layout_nodes.push(layout_child);
    }

    return layout_nodes;
}


fn build_anonymous_block_layout_node(visible: bool, inline_children: Vec<Rc<RefCell<LayoutNode>>>, background_color: Color) -> Rc<RefCell<LayoutNode>> {
    let id_of_node_being_built = get_next_layout_node_interal_id();

    let empty_box_layout_node = BoxLayoutNode {
        location: Rect::empty(),
        background_color,
    };

    let anonymous_node = LayoutNode {
        internal_id: id_of_node_being_built,
        display: Display::Block,
        visible: visible,
        children: Some(inline_children),
        from_dom_node: None,
        content: LayoutNodeContent::BoxLayoutNode(empty_box_layout_node),
    };

    return Rc::new(RefCell::from(anonymous_node));
}
