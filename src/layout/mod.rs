use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use image::DynamicImage;

use crate::color::Color;
use crate::debug::debug_log_warn;
use crate::dom::{
    Document,
    ElementDomNode,
    TagName,
};
use crate::network::url::Url;
use crate::platform::fonts::{
    Font,
    FontContext,
    FontFace,
};
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
    pub all_nodes: HashMap<usize, Rc<RefCell<LayoutNode>>>,
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

        return FullLayout { root_node: Rc::from(RefCell::from(layout_node)), all_nodes: HashMap::new(), nodes_in_selection_order: Vec::new() };
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
pub enum LayoutNodeContent {
    TextLayoutNode(TextLayoutNode),
    ImageLayoutNode(ImageLayoutNode),
    ButtonLayoutNode(ButtonLayoutNode),
    TextInputLayoutNode(TextInputLayoutNode),
    BoxLayoutNode(BoxLayoutNode),
    NoContent,
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct LayoutNode {
    pub internal_id: usize,
    pub parent_id: usize,
    pub children: Option<Vec<Rc<RefCell<LayoutNode>>>>,

    pub from_dom_node: Option<Rc<RefCell<ElementDomNode>>>,

    pub display: Display,
    pub visible: bool,

    pub content: LayoutNodeContent,

    pub optional_link_url: Option<Url>, //TODO: This is not really nice, there are other things that could happen on click as well. Refactor to a more
                                        //      general setup of what should happen on click (or, better defer to the DOM for that, should not be on layout nodes)
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
            LayoutNodeContent::ImageLayoutNode(img_node) => { return (img_node.location.width, img_node.location.height) },
            LayoutNodeContent::ButtonLayoutNode(button_node)  => { return (button_node.location.width, button_node.location.height) },
            LayoutNodeContent::TextInputLayoutNode(input_node) => { return (input_node.location.width, input_node.location.height) },
            LayoutNodeContent::BoxLayoutNode(box_node) => { return (box_node.location.width, box_node.location.height) },
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
            LayoutNodeContent::NoContent => { return false; }
        }
    }

    pub fn find_clickable(&self, x: f32, y: f32, current_scroll_y: f32) -> Option<Url> {
        if !self.visible_on_y_location(current_scroll_y) {
            return None;
        }

        if self.optional_link_url.is_some() {
            let mut in_rect = false;

            match &self.content {
                LayoutNodeContent::TextLayoutNode(text_layout_node) => {
                    for rect in text_layout_node.rects.iter() {
                        if rect.location.is_inside(x, y) {
                            in_rect = true;
                            break;
                        }
                    }
                },
                LayoutNodeContent::ImageLayoutNode(image_node) => {
                    if image_node.location.is_inside(x, y) { in_rect = true; }
                }
                LayoutNodeContent::BoxLayoutNode(box_node) => {
                    if box_node.location.is_inside(x, y) { in_rect = true; }
                }
                LayoutNodeContent::ButtonLayoutNode(_) => todo!(), //TODO: implement (note: first refactor the way we click thinks. Use click handlers always
                                                                   //      and have those on the dom instead of the layout tree)
                _ => {},
            }

            if in_rect {
                return self.optional_link_url.clone();
            }
        }


        if self.children.is_some() {
            for child in self.children.as_ref().unwrap() {
                if RefCell::borrow(child).visible {
                    let opt_url = RefCell::borrow(child).find_clickable(x, y, current_scroll_y);
                    if opt_url.is_some() {
                        return opt_url;
                    }
                }
            }
        }

        return None;
    }

    pub fn new_empty() -> LayoutNode {
        return LayoutNode {
            internal_id: 0,
            display: Display::Block,
            visible: true,
            children: None,
            parent_id: 0,
            from_dom_node: None,
            content: LayoutNodeContent::NoContent,
            optional_link_url: None,
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
            LayoutNodeContent::ImageLayoutNode(_) => todo!(),  //TODO: implement
            LayoutNodeContent::ButtonLayoutNode(_) => todo!(),  //TODO: implement
            LayoutNodeContent::TextInputLayoutNode(_) => todo!(),  //TODO: implement
            LayoutNodeContent::BoxLayoutNode(_) => {
                //Note: this is a no-op for now, since there is nothing to select in a box node itself (just in its children)
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


pub fn build_full_layout(document: &Document, font_context: &FontContext, main_url: &Url) -> FullLayout {
    let mut top_level_layout_nodes: Vec<Rc<RefCell<LayoutNode>>> = Vec::new();
    let mut all_nodes: HashMap<usize, Rc<RefCell<LayoutNode>>> = HashMap::new();

    let id_of_node_being_built = get_next_layout_node_interal_id();

    let mut state = LayoutBuildState { last_char_was_space: false };

    let layout_node = build_layout_tree(&document.document_node, document, &mut all_nodes, id_of_node_being_built, font_context, 
                                        main_url, &mut state, None);
    top_level_layout_nodes.push(layout_node);

    let root_node = LayoutNode {
        internal_id: id_of_node_being_built,
        display: Display::Block,
        visible: true,
        children: Some(top_level_layout_nodes),
        parent_id: id_of_node_being_built,  //this is the top node, so it does not really have a parent, we set it to ourselves,
        from_dom_node: None,
        content: LayoutNodeContent::BoxLayoutNode(BoxLayoutNode {
            location: Rect::empty(),
            background_color: Color::WHITE,
        }),
        optional_link_url: None,
    };

    let rc_root_node = Rc::new(RefCell::from(root_node));
    all_nodes.insert(id_of_node_being_built, Rc::clone(&rc_root_node));

    let mut nodes_in_selection_order = Vec::new();
    collect_content_nodes_in_walk_order(&rc_root_node, &mut nodes_in_selection_order);

    return FullLayout { root_node: rc_root_node, all_nodes, nodes_in_selection_order };
}


fn collect_content_nodes_in_walk_order(node: &Rc<RefCell<LayoutNode>>, result: &mut Vec<Rc<RefCell<LayoutNode>>>) {
    match RefCell::borrow(node).content {
        LayoutNodeContent::TextLayoutNode(_) => { result.push(Rc::clone(&node)); },
        LayoutNodeContent::ImageLayoutNode(_) => { result.push(Rc::clone(&node)); },
        LayoutNodeContent::ButtonLayoutNode(_) => { result.push(Rc::clone(&node)); },
        LayoutNodeContent::TextInputLayoutNode(_) => { result.push(Rc::clone(&node)); },
        LayoutNodeContent::BoxLayoutNode(_) => {},
        LayoutNodeContent::NoContent => {},
    }

    if RefCell::borrow(node).children.as_ref().is_some() {
        for child in RefCell::borrow(node).children.as_ref().unwrap() {
            collect_content_nodes_in_walk_order(&child, result);
        }
    }
}


//This function is responsible for setting the location rects on the node, and all its children.
//TODO: we now pass in top_left x and y, but I think we should compute the positions just for layout, and offset for UI in the render phase...
pub fn compute_layout(node: &Rc<RefCell<LayoutNode>>, all_nodes: &HashMap<usize, Rc<RefCell<LayoutNode>>>, style_context: &StyleContext,
                      top_left_x: f32, top_left_y: f32, font_context: &FontContext, only_update_block_vertical_position: bool, force_full_layout: bool) {

    let mut mut_node = RefCell::borrow_mut(node);

    if only_update_block_vertical_position && !force_full_layout {
        let y_diff = top_left_y - mut_node.y_position();
        mut_node.move_node_vertically(y_diff);
        return;
    }

    if !mut_node.visible {
        mut_node.update_single_rect_location(Rect { x: top_left_x, y: top_left_y, width: 0.0, height: 0.0 });

    } else if mut_node.children.is_some() {
        if mut_node.all_childnodes_have_given_display(Display::Block) {
            apply_block_layout(&mut mut_node, all_nodes, style_context, top_left_x, top_left_y, font_context, force_full_layout);
        } else if mut_node.all_childnodes_have_given_display(Display::Inline) {
            apply_inline_layout(&mut mut_node, all_nodes, style_context, top_left_x, top_left_y, CONTENT_WIDTH - top_left_x, font_context, force_full_layout);
        } else {
            panic!("Not all children are either inline or block, earlier in the process this should already have been fixed with anonymous blocks");
        }

    } else {

        let node_was_dirty = if mut_node.from_dom_node.is_some() {
            let node_was_dirty = mut_node.from_dom_node.as_ref().unwrap().borrow().dirty;
            mut_node.from_dom_node.as_ref().unwrap().borrow_mut().dirty = false;
            node_was_dirty
        } else {
            false
        };

        let opt_dom_node = if mut_node.from_dom_node.is_some() {
            Some(Rc::clone(&mut_node.from_dom_node.as_ref().unwrap()))
        } else {
            None
        };

        match &mut mut_node.content {
            LayoutNodeContent::TextLayoutNode(ref mut text_layout_node) => {

                if node_was_dirty {
                    text_layout_node.undo_split_rects();
                }

                for rect in text_layout_node.rects.iter_mut() {
                    let (rect_width, rect_height) = font_context.get_text_dimension(&rect.text, &rect.font);
                    rect.location = Rect { x: top_left_x, y: top_left_y, width: rect_width, height: rect_height };
                }
            },
            LayoutNodeContent::ImageLayoutNode(ref mut image_layout_node) => {

                if node_was_dirty {
                    //TODO: why are we reloading the image here? In the build tree we also set it. If we don't rebuild if the image changes, we should set
                    //      it here, but then we should not set it in the build step. Or, if we _do_ rebuild, we should not update it here...
                    //         -> I think we only rebuild if we get a new document, so then setting it here makes sense, but then build should not.
                    //            probably the same for other content.....

                    let dom_node = opt_dom_node.as_ref().unwrap().borrow();
                    let opt_image_clone = if dom_node.image.is_some() {
                        dom_node.image.as_ref().unwrap().deref().clone()
                    } else {
                        panic!("invalid state"); // we have built an image node based on the DOM, so there should be an image on the DOM
                    };

                    image_layout_node.image = opt_image_clone;

                    opt_dom_node.as_ref().unwrap().borrow_mut().dirty = false;
                }

                image_layout_node.location =
                     Rect { x: top_left_x, y: top_left_y, width: image_layout_node.image.width() as f32, height: image_layout_node.image.height() as f32 };
            },
            LayoutNodeContent::ButtonLayoutNode(_) => todo!(),  //TODO: implement
            LayoutNodeContent::TextInputLayoutNode(_) => todo!(),  //TODO: implement
            LayoutNodeContent::BoxLayoutNode(box_node) => {
                //Note: this is a boxlayoutnode, but without children (because that is a seperate case above), so no content.

                //TODO: for now generating 1 by 1 sized, this might not be correct given styling.
                box_node.location = Rect { x: top_left_x, y: top_left_y, width: 1.0, height: 1.0 };
            },
            LayoutNodeContent::NoContent => todo!(),  //TODO: implement
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


fn apply_block_layout(node: &mut LayoutNode, all_nodes: &HashMap<usize, Rc<RefCell<LayoutNode>>>, style_context: &StyleContext,
                      top_left_x: f32, top_left_y: f32, font_context: &FontContext, force_full_layout: bool) {
    let mut cursor_y = top_left_y;
    let mut max_width: f32 = 0.0;

    for child in node.children.as_ref().unwrap() {
        let only_update_block_vertical_position = !child.borrow().is_dirty_anywhere(); //Since the parent node is block layout, we can shift the while block up and down if its not dirty
        compute_layout(&child, all_nodes, style_context, top_left_x, cursor_y, font_context, only_update_block_vertical_position, force_full_layout);
        let (bounding_box_width, bounding_box_height) = RefCell::borrow(child).get_size_of_bounding_box();

        cursor_y += bounding_box_height;
        max_width = max_width.max(bounding_box_width);
    }

    let our_height = cursor_y - top_left_y;
    node.update_single_rect_location(Rect { x: top_left_x, y: top_left_y, width: max_width, height: our_height });
}


fn apply_inline_layout(node: &mut LayoutNode, all_nodes: &HashMap<usize, Rc<RefCell<LayoutNode>>>, style_context: &StyleContext, top_left_x: f32,
                       top_left_y: f32, max_allowed_width: f32, font_context: &FontContext, force_full_layout: bool) {
    let mut cursor_x = top_left_x;
    let mut cursor_y = top_left_y;
    let mut max_width: f32 = 0.0;
    let mut max_height_of_line: f32 = 0.0;

    for child in node.children.as_ref().unwrap() {
        let only_update_block_vertical_position = false; //we can only do this if the parent is block layout, but in this case its inline. Inline might cause horizonal cascading changes.
        compute_layout(&child, all_nodes, style_context, cursor_x, cursor_y, font_context, only_update_block_vertical_position, force_full_layout);

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
                    compute_layout(&child, all_nodes, style_context, cursor_x, cursor_y, font_context, only_update_block_vertical_position, force_full_layout);
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

    //Note: we don't re-compute the char positions after wrapping, since we always break on spaces, which don't affect kerning of the next glyph
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


fn build_layout_tree(main_node: &Rc<RefCell<ElementDomNode>>, document: &Document, all_nodes: &mut HashMap<usize, Rc<RefCell<LayoutNode>>>,
                     parent_id: usize, font_context: &FontContext, main_url: &Url, state: &mut LayoutBuildState,
                     optional_new_text: Option<String>) -> Rc<RefCell<LayoutNode>> {
    let mut partial_node_visible = true;
    let mut partial_node_optional_link_url = None;
    let mut partial_node_optional_img = None;
    let mut partial_node_line_break = false;
    let mut partial_node_styles = resolve_full_styles_for_layout_node(&Rc::clone(main_node), &document.all_nodes, &document.style_context);
    let mut partial_node_children = None;
    let mut partial_node_is_submit_button = false;
    let mut partial_node_is_text_input = false;
    let mut partial_text = None;
    let mut partial_font = None;
    let mut partial_font_color = None;
    let mut partial_node_non_breaking_space_positions = None;

    let partial_node_background_color = get_color_style_value(&partial_node_styles, "background-color").unwrap_or(Color::WHITE);

    let mut childs_to_recurse_on: &Option<Vec<Rc<RefCell<ElementDomNode>>>> = &None;

    let main_node_refcell = main_node;
    let main_node = RefCell::borrow(main_node);

    if main_node.text.is_some() {
        partial_text = if optional_new_text.is_some() {
            Some(optional_new_text.unwrap())
        } else {
            Some(main_node.text.as_ref().unwrap().text_content.clone())
        };

        let font = get_font_given_styles(&partial_node_styles);
        partial_font = Some(font.0);
        partial_font_color = Some(font.1);
        partial_node_non_breaking_space_positions = main_node.text.as_ref().unwrap().non_breaking_space_positions.clone();

    } else if main_node.name.is_some() {
        debug_assert!(optional_new_text.is_none());

        childs_to_recurse_on = &main_node.children;

        match &main_node.name_for_layout {
            TagName::A => {
                let opt_href = main_node.get_attribute_value("href");
                if opt_href.is_some() {
                    partial_node_optional_link_url = Some(Url::from_base_url(&opt_href.unwrap(), Some(main_url)));
                } else {
                    partial_node_optional_link_url = None;
                }
            }

            TagName::B => {
                //TODO: can this style not be in the general stylesheet?
                partial_node_styles.insert("font-weight".to_owned(), "bold".to_owned());
            }

            TagName::Br => {
                //A newline does not have text, but we still want to make a text node, since things like fontsize affect how it looks
                partial_text = Some(String::new());

                partial_node_line_break = true;

                let font = get_font_given_styles(&partial_node_styles);
                partial_font = Some(font.0);
                partial_font_color = Some(font.1);
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

            TagName::Other => {}
        }
    } else if main_node.is_document_node {
        childs_to_recurse_on = &main_node.children;
    }


    let id_of_node_being_built = get_next_layout_node_interal_id();

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
                        let layout_childs = build_layout_for_inline_nodes(&temp_inline_child_buffer, document, all_nodes, id_of_node_being_built,
                                                                          font_context, main_url, state);

                        let anon_block = build_anonymous_block_layout_node(true, id_of_node_being_built, layout_childs, all_nodes, background_color);
                        partial_node_children.as_mut().unwrap().push(anon_block);

                        temp_inline_child_buffer = Vec::new();
                    }

                    state.last_char_was_space = false;
                    let layout_child = build_layout_tree(child, document, all_nodes, id_of_node_being_built, font_context, main_url, state, None);
                    partial_node_children.as_mut().unwrap().push(layout_child);

                } else {
                    temp_inline_child_buffer.push(child);
                }

            }

            if !temp_inline_child_buffer.is_empty() {
                let layout_childs = build_layout_for_inline_nodes(&temp_inline_child_buffer, document, all_nodes, id_of_node_being_built,
                                                                  font_context, main_url, state);

                let anon_block = build_anonymous_block_layout_node(true, id_of_node_being_built, layout_childs, all_nodes, background_color);
                partial_node_children.as_mut().unwrap().push(anon_block);
            }

        } else if get_display_type(&first_child) == Display::Inline {

            let mut inline_nodes_to_layout = Vec::new();
            for child in childs_to_recurse_on.as_ref().unwrap() {
                inline_nodes_to_layout.push(child);
            }
            let layout_childs = build_layout_for_inline_nodes(&inline_nodes_to_layout, document, all_nodes, id_of_node_being_built,
                                                              font_context, main_url, state);

            for layout_child in layout_childs {
                partial_node_children.as_mut().unwrap().push(layout_child);
            }

        } else { //This means all childs are Display::Block

            for child in childs_to_recurse_on.as_ref().unwrap() {
                state.last_char_was_space = false;
                let layout_child = build_layout_tree(child, document, all_nodes, id_of_node_being_built, font_context, main_url, state, None);
                partial_node_children.as_mut().unwrap().push(layout_child);
            }
        }

    }

    let content = if partial_text.is_some() {
        let rect = TextLayoutRect {
            char_position_mapping: font_context.compute_char_position_mapping(&partial_font.as_ref().unwrap(), &partial_text.as_ref().unwrap()),
            non_breaking_space_positions: partial_node_non_breaking_space_positions,
            location: Rect::empty(),
            selection_rect: None,
            selection_char_range: None,
            text: partial_text.unwrap(),
            font: partial_font.unwrap(),
            font_color: partial_font_color.unwrap(),
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
        internal_id: id_of_node_being_built,
        display: get_display_type(main_node_refcell),
        visible: partial_node_visible,
        children: partial_node_children,
        parent_id: parent_id,
        from_dom_node: Some(Rc::clone(&main_node_refcell)),
        content: content,
        optional_link_url: partial_node_optional_link_url,
    };

    let rc_new_node = Rc::new(RefCell::from(new_node));
    all_nodes.insert(id_of_node_being_built, Rc::clone(&rc_new_node));

    return rc_new_node;
}


fn build_layout_for_inline_nodes(inline_nodes: &Vec<&Rc<RefCell<ElementDomNode>>>, document: &Document, all_nodes: &mut HashMap<usize, Rc<RefCell<LayoutNode>>>,
                                 parent_id: usize, font_context: &FontContext, main_url: &Url, state: &mut LayoutBuildState) -> Vec<Rc<RefCell<LayoutNode>>> {

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

        let layout_child = build_layout_tree(node, document, all_nodes, parent_id, font_context, main_url, state, optional_new_text);
        layout_nodes.push(layout_child);
    }

    return layout_nodes;
}


fn build_anonymous_block_layout_node(visible: bool, parent_id: usize, inline_children: Vec<Rc<RefCell<LayoutNode>>>,
                                     all_nodes: &mut HashMap<usize, Rc<RefCell<LayoutNode>>>, background_color: Color) -> Rc<RefCell<LayoutNode>> {
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
        parent_id: parent_id,
        from_dom_node: None,
        content: LayoutNodeContent::BoxLayoutNode(empty_box_layout_node),
        optional_link_url: None,
    };

    let internal_id = anonymous_node.internal_id;
    let anon_rc = Rc::new(RefCell::from(anonymous_node));
    all_nodes.insert(internal_id, Rc::clone(&anon_rc));
    return anon_rc;
}
