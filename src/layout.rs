use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use image::DynamicImage;

use crate::color::Color;
use crate::dom::{
    Document,
    ElementDomNode,
    TagName,
};
use crate::Font;
use crate::network::url::Url;
use crate::platform::Platform;
use crate::SCREEN_HEIGHT;
use crate::style::{
    get_color_style_value,
    get_property_from_computed_styles,
    has_style_value, resolve_css_numeric_type_value,
    resolve_full_styles_for_layout_node,
    StyleContext,
};
use crate::ui::CONTENT_WIDTH;
use crate::ui_components::compute_char_position_mapping;


static NEXT_LAYOUT_NODE_INTERNAL: AtomicUsize = AtomicUsize::new(1);
pub fn get_next_layout_node_interal_id() -> usize { NEXT_LAYOUT_NODE_INTERNAL.fetch_add(1, Ordering::Relaxed) }


pub struct FullLayout {
    pub root_node: Rc<RefCell<LayoutNode>>,
    pub all_nodes: HashMap<usize, Rc<RefCell<LayoutNode>>>,
    pub nodes_in_selection_order: Vec<Rc<RefCell<LayoutNode>>>,
}
impl FullLayout {
    pub fn page_height(&self) -> f32 {
        return RefCell::borrow(&self.root_node).rects.iter().next().unwrap().location.height;
    }
    pub fn new_empty(platform: &mut Platform) -> FullLayout {
        //Note that we we create a 1x1 rect even for an empty layout, since we need a rect to render it (for example when the first page is still loading)

        let mut layout_node = LayoutNode::new_empty();
        let text = String::new();
        let location = Rect { x: 0.0, y: 0.0, width: 1.0, height: 1.0 };
        let font = Font::new(false, false, 18);
        let char_position_mapping = compute_char_position_mapping(platform, &font, &text);
        let rect_text_data = Some(RectTextData { text, font, font_color: Color::BLACK, char_position_mapping, non_breaking_space_positions: None });
        layout_node.rects.push(LayoutRect { text_data: rect_text_data, image: None, location, selection_rect: None, selection_char_range: None });

        return FullLayout { root_node: Rc::from(RefCell::from(layout_node)), all_nodes: HashMap::new(), nodes_in_selection_order: Vec::new() };
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct LayoutNode {
    pub internal_id: usize,
    pub display: Display,
    pub visible: bool,
    pub line_break: bool,
    pub children: Option<Vec<Rc<RefCell<LayoutNode>>>>,
    pub parent_id: usize,
    pub optional_link_url: Option<Url>,
    pub rects: Vec<LayoutRect>,
    pub pre_wrap_rect_backup: Option<LayoutRect>,
    pub from_dom_node: Option<Rc<RefCell<ElementDomNode>>>,
    pub background_color: Color,
}
impl LayoutNode {
    pub fn all_childnodes_have_given_display(&self, display: Display) -> bool {
        if self.children.is_none() {
            return true;
        }
        return self.children.as_ref().unwrap().iter().all(|node| RefCell::borrow(node).display == display);
    }
    pub fn update_single_rect_location(&mut self, new_location: Rect) {
        debug_assert!(self.rects.len() == 1);
        self.rects[0].location = new_location;
    }
    pub fn can_wrap(&self) -> bool {
        return self.rects.iter().any(|rect| { rect.text_data.is_some()});
    }
    pub fn get_size_of_bounding_box(&self) -> (f32, f32) {
        let mut lowest_x = f32::MAX;
        let mut lowest_y = f32::MAX;
        let mut max_x: f32 = 0.0;
        let mut max_y: f32 = 0.0;

        for rect in self.rects.iter() {
            lowest_x = lowest_x.min(rect.location.x);
            lowest_y = lowest_y.min(rect.location.y);
            max_x = max_x.max(rect.location.x + rect.location.width);
            max_y = max_y.max(rect.location.y + rect.location.height);
        }

        let bounding_box_width = max_x - lowest_x;
        let bounding_box_height = max_y - lowest_y;
        return (bounding_box_width, bounding_box_height);
    }
    pub fn find_clickable(&self, x: f32, y: f32, current_scroll_y: f32) -> Option<Url> {
        let any_visible = self.rects.iter().any(|rect| -> bool {rect.location.is_visible_on_y_location(current_scroll_y)});
        if !any_visible {
            return None;
        }

        if self.optional_link_url.is_some() {
            for rect in self.rects.iter() {
                if x >= rect.location.x && x <= rect.location.x + rect.location.width &&
                   y >= rect.location.y && y <= rect.location.y + rect.location.height {
                    return self.optional_link_url.clone();
                }
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
            line_break: false,
            children: None,
            parent_id: 0,
            optional_link_url: None,
            rects: Vec::new(),
            pre_wrap_rect_backup: None,
            from_dom_node: None,
            background_color: Color::WHITE,
        };
    }
    pub fn reset_selection(&mut self) {
        for rect in self.rects.iter_mut() {
            rect.selection_rect = None;
            rect.selection_char_range = None;
        }
        if self.children.is_some() {
            for child in self.children.as_ref().unwrap() {
                RefCell::borrow_mut(child).reset_selection();
            }
        }
    }
    pub fn get_selected_text(&self, result: &mut String) {
        for rect in &self.rects {
            if rect.selection_char_range.is_some() {
                let (start_idx, end_idx) = rect.selection_char_range.unwrap();
                result.push_str(rect.text_data.as_ref().unwrap().text.chars().skip(start_idx).take(end_idx - start_idx + 1).collect::<String>().as_str());
            }
        }

        if self.children.is_some() {
            for child in self.children.as_ref().unwrap() {
                RefCell::borrow_mut(child).get_selected_text(result);
            }
        }
    }
    pub fn undo_split_rects(&mut self) {
        //The main intention for this method is to be used before we start the process of computing line wrapping again (to undo the previous wrapping)
        if self.rects.len() > 1 {
            debug_assert!(self.rects.iter().all(|rect| -> bool { rect.text_data.is_some()} ));
            debug_assert!(self.pre_wrap_rect_backup.is_some());
            self.rects = vec![self.pre_wrap_rect_backup.as_ref().unwrap().clone()];
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
        for rect in self.rects.iter_mut() {
            rect.location.y += y_diff;
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
pub struct LayoutRect {
    pub text_data: Option<RectTextData>,
    pub image: Option<DynamicImage>,
    pub location: Rect,
    pub selection_rect: Option<Rect>,
    pub selection_char_range: Option<(usize, usize)>,
}
impl LayoutRect {
    pub fn get_default_non_computed_rect() -> LayoutRect {
        return LayoutRect {
            text_data: None,
            image: None,
            location: Rect::empty(),
            selection_rect: None,
            selection_char_range: None,
        };
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
pub struct RectTextData {
    pub text: String,
    pub font: Font,
    pub font_color: Color,
    pub char_position_mapping: Vec<f32>,
    pub non_breaking_space_positions: Option<HashSet<usize>>,
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


pub fn build_full_layout(document: &Document, platform: &mut Platform, main_url: &Url) -> FullLayout {
    let mut top_level_layout_nodes: Vec<Rc<RefCell<LayoutNode>>> = Vec::new();
    let mut all_nodes: HashMap<usize, Rc<RefCell<LayoutNode>>> = HashMap::new();

    let id_of_node_being_built = get_next_layout_node_interal_id();

    let mut state = LayoutBuildState { last_char_was_space: false };

    let layout_node = build_layout_tree(&document.document_node, document, &mut all_nodes, id_of_node_being_built, platform, 
                                        main_url, &mut state, None);
    top_level_layout_nodes.push(layout_node);

    let root_node = LayoutNode {
        internal_id: id_of_node_being_built,
        display: Display::Block,
        visible: true,
        line_break: false,
        children: Some(top_level_layout_nodes),
        parent_id: id_of_node_being_built,  //this is the top node, so it does not really have a parent, we set it to ourselves,
        optional_link_url: None,
        rects: vec![LayoutRect::get_default_non_computed_rect()],
        pre_wrap_rect_backup: None,
        from_dom_node: None,
        background_color: Color::WHITE,
    };

    let rc_root_node = Rc::new(RefCell::from(root_node));
    all_nodes.insert(id_of_node_being_built, Rc::clone(&rc_root_node));

    let mut nodes_in_selection_order = Vec::new();
    collect_content_nodes_in_walk_order(&rc_root_node, &mut nodes_in_selection_order);

    return FullLayout { root_node: rc_root_node, all_nodes, nodes_in_selection_order };
}


fn collect_content_nodes_in_walk_order(node: &Rc<RefCell<LayoutNode>>, result: &mut Vec<Rc<RefCell<LayoutNode>>>) {

    let any_content = RefCell::borrow(node).rects.iter().any(|rect| -> bool { rect.image.is_some() || rect.text_data.is_some() } );
    if any_content {
        result.push(Rc::clone(&node));
    }

    if RefCell::borrow(node).children.as_ref().is_some() {
        for child in RefCell::borrow(node).children.as_ref().unwrap() {
            collect_content_nodes_in_walk_order(&child, result);
        }
    }
}


//This function is responsible for setting the location rects on the node, and all its children.
//TODO: need to find a way to make good tests for this (probably via exporting the layout in JSON)
//TODO: we now pass in top_left x and y, but I think we should compute the positions just for layout, and offset for UI in the render phase...
pub fn compute_layout(node: &Rc<RefCell<LayoutNode>>, all_nodes: &HashMap<usize, Rc<RefCell<LayoutNode>>>, style_context: &StyleContext,
                      top_left_x: f32, top_left_y: f32, platform: &mut Platform, only_update_block_vertical_position: bool, force_full_layout: bool) {

    let mut mut_node = RefCell::borrow_mut(node);

    if only_update_block_vertical_position && !force_full_layout {
        let y_diff = top_left_y - mut_node.rects.iter().next().unwrap().location.y;
        mut_node.move_node_vertically(y_diff);
        return;
    }

    if !mut_node.visible {
        mut_node.update_single_rect_location(Rect { x: top_left_x, y: top_left_y, width: 0.0, height: 0.0 });

    } else if mut_node.children.is_some() {
        if mut_node.all_childnodes_have_given_display(Display::Block) {
            apply_block_layout(&mut mut_node, all_nodes, style_context, top_left_x, top_left_y, platform, force_full_layout);
        } else if mut_node.all_childnodes_have_given_display(Display::Inline) {
            apply_inline_layout(&mut mut_node, all_nodes, style_context, top_left_x, top_left_y, CONTENT_WIDTH - top_left_x, platform, force_full_layout);
        } else {
            panic!("Not all children are either inline or block, earlier in the process this should already have been fixed with anonymous blocks");
        }

    } else {

        if mut_node.from_dom_node.is_some() {
            if mut_node.from_dom_node.as_ref().unwrap().borrow().dirty {

                //TODO: for now below we update the rect content. We should eventually also update styles etc.

                mut_node.undo_split_rects();

                let opt_image_clone = {
                    let dom_node = mut_node.from_dom_node.as_ref().unwrap().borrow();
                    let opt_image_clone = if dom_node.image.is_some() {
                        Some(dom_node.image.as_ref().unwrap().deref().clone())
                    } else {
                        None
                    };
                    opt_image_clone
                };

                debug_assert!(mut_node.rects.len() == 1);
                let main_rect = mut_node.rects.iter_mut().next().unwrap();
                main_rect.image = opt_image_clone;

                //TODO: also update text, and possible other content

                mut_node.from_dom_node.as_ref().unwrap().borrow_mut().dirty = false;
            }
        }

        for rect in mut_node.rects.iter_mut() {
            let (rect_width, rect_height) = compute_size_for_rect(rect, platform);
            rect.location = Rect { x: top_left_x, y: top_left_y, width: rect_width, height: rect_height };
        }
    }
}


pub fn get_font_given_styles(styles: &HashMap<String, String>) -> (Font, Color) {
    let font_bold = has_style_value(&styles, "font-weight", &"bold".to_owned());
    let font_underline = has_style_value(&styles, "text-decoration", &"underline".to_owned());
    let opt_font_size = get_property_from_computed_styles(&styles, "font-size");
    let font_size = resolve_css_numeric_type_value(&opt_font_size.unwrap()); //font-size has a default value, so this is a fatal error if not found

    let font_color_option = get_color_style_value(&styles, "color");
    let font_color = font_color_option.unwrap(); //color has a default value, so this is a fatal error if not found

    return (Font::new(font_bold, font_underline, font_size as u16), font_color);
}


fn apply_block_layout(node: &mut LayoutNode, all_nodes: &HashMap<usize, Rc<RefCell<LayoutNode>>>, style_context: &StyleContext,
                      top_left_x: f32, top_left_y: f32, platform: &mut Platform, force_full_layout: bool) {
    let mut cursor_y = top_left_y;
    let mut max_width: f32 = 0.0;

    for child in node.children.as_ref().unwrap() {
        let only_update_block_vertical_position = !child.borrow().is_dirty_anywhere(); //Since the parent node is block layout, we can shift the while block up and down if its not dirty
        compute_layout(&child, all_nodes, style_context, top_left_x, cursor_y, platform, only_update_block_vertical_position, force_full_layout);
        let (bounding_box_width, bounding_box_height) = RefCell::borrow(child).get_size_of_bounding_box();

        cursor_y += bounding_box_height;
        max_width = max_width.max(bounding_box_width);
    }

    let our_height = cursor_y - top_left_y;
    node.update_single_rect_location(Rect { x: top_left_x, y: top_left_y, width: max_width, height: our_height });
}


fn apply_inline_layout(node: &mut LayoutNode, all_nodes: &HashMap<usize, Rc<RefCell<LayoutNode>>>, style_context: &StyleContext, top_left_x: f32,
                       top_left_y: f32, max_allowed_width: f32, platform: &mut Platform, force_full_layout: bool) {
    let mut cursor_x = top_left_x;
    let mut cursor_y = top_left_y;
    let mut max_width: f32 = 0.0;
    let mut max_height_of_line: f32 = 0.0;

    for child in node.children.as_ref().unwrap() {
        let only_update_block_vertical_position = false; //we can only do this if the parent is block layout, but in this case its inline. Inline might cause horizonal cascading changes.
        compute_layout(&child, all_nodes, style_context, cursor_x, cursor_y, platform, only_update_block_vertical_position, force_full_layout);

        if RefCell::borrow(child).line_break {
            let child_height;
            if cursor_x != top_left_x {
                cursor_x = top_left_x;
                cursor_y += max_height_of_line;
                child_height = max_height_of_line;
            } else {
                //TODO: we need to make the height of the newline dependent on the font size, but
                //we don't have the styles anymore. Should we just make sure the font is set for
                //newline as well? It seems we don't have a rect in that case?

                //let (font, _) = get_font_given_styles(&RefCell::borrow(child).styles);
                ////we get the hight of a random character in the font for the height of the newline:
                //let (_, random_char_height) = platform.get_text_dimension(&String::from("x"), &font);
                let random_char_height = 16.0; //TODO: temporary hardcoded value

                cursor_x = top_left_x;
                cursor_y += random_char_height;
                child_height = random_char_height;
            }

            RefCell::borrow_mut(child).update_single_rect_location(Rect { x: top_left_x, y: top_left_y, width: max_width, height: child_height });

            continue;
        }

        RefCell::borrow_mut(child).undo_split_rects();

        let child_borrow = RefCell::borrow(child);
        let (child_width, child_height) = child_borrow.get_size_of_bounding_box();

        if (cursor_x - top_left_x + child_width) > max_allowed_width {

            if child_borrow.children.is_none() && child_borrow.can_wrap() && child_borrow.rects.iter().all(|rect| -> bool { rect.text_data.is_some()} ) {
                // in this case, we might be able to split rects, and put part of the node on this line

                let first_rect = child_borrow.rects.iter().next().unwrap();
                let text_data = first_rect.text_data.as_ref().unwrap();
                let font_color = text_data.font_color;
                let relative_cursor_x = cursor_x - top_left_x;
                let amount_of_space_left_on_line = max_allowed_width - relative_cursor_x;
                let wrapped_text = wrap_text(child_borrow.rects.last().unwrap(), max_allowed_width, amount_of_space_left_on_line, &text_data.font, platform);

                let mut rects_for_child = Vec::new();
                for text in wrapped_text {

                    let new_text_data = RectTextData {
                        font: text_data.font.clone(),
                        font_color: font_color,
                        char_position_mapping: compute_char_position_mapping(platform, &text_data.font, &text),
                        non_breaking_space_positions: None, //For now not computing these, although it would be more correct to update them after wrapping
                        text: text,
                    };

                    let mut new_rect = LayoutRect {
                        text_data: Some(new_text_data),
                        image: None,
                        location: Rect::empty(),
                        selection_rect: None,
                        selection_char_range: None,
                    };

                    let (rect_width, rect_height) = compute_size_for_rect(&new_rect, platform);

                    if cursor_x - top_left_x + rect_width > max_allowed_width {
                        if cursor_x != top_left_x {
                            cursor_x = top_left_x;
                            cursor_y += max_height_of_line;
                            max_height_of_line = 0.0;
                        }
                    }

                    new_rect.location = Rect { x: cursor_x, y: cursor_y, width: rect_width, height: rect_height };
                    rects_for_child.push(new_rect);

                    cursor_x += rect_width;
                    max_width = max_width.max(cursor_x);
                    max_height_of_line = max_height_of_line.max(rect_height);

                }
                let rect_backup = child_borrow.rects.iter().next().unwrap().clone();
                drop(child_borrow);

                RefCell::borrow_mut(child).pre_wrap_rect_backup = Some(rect_backup);
                RefCell::borrow_mut(child).rects = rects_for_child;

            } else {
                if cursor_x != top_left_x {
                    //we can move to a new line, it might fit there

                    cursor_x = top_left_x;
                    cursor_y += max_height_of_line;

                    let only_update_block_vertical_position = false; //we can only do this if the parent is block layout, but in this case its inline. Inline might cause horizonal cascading changes.
                    drop(child_borrow);
                    compute_layout(&child, all_nodes, style_context, cursor_x, cursor_y, platform, only_update_block_vertical_position, force_full_layout);
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


//Note that this function returns the size, but does not update the rect with that size (because we also need to position for the computed location object)
fn compute_size_for_rect(layout_rect: &LayoutRect, platform: &mut Platform) -> (f32, f32) {

    if layout_rect.text_data.is_some() {
        let text_data = layout_rect.text_data.as_ref().unwrap();
        return platform.get_text_dimension(&text_data.text, &text_data.font);
    }

    if layout_rect.image.is_some() {
        let img = layout_rect.image.as_ref().unwrap();
        return (img.width() as f32, img.height() as f32);
    }

    return (0.0, 0.0);
}


fn wrap_text(layout_rect: &LayoutRect, max_width: f32, width_remaining_on_current_line: f32, font: &Font, platform: &mut Platform) -> Vec<String> {
    let text = &layout_rect.text_data.as_ref().unwrap().text;
    let no_wrap_positions = &layout_rect.text_data.as_ref().unwrap().non_breaking_space_positions;
    let mut str_buffers = Vec::new();
    let mut str_buffer_undecided = String::new();
    let mut pos = 0;
    let mut current_line = 0;

    str_buffers.push(String::new());

    let mut char_iter = text.chars();
    loop {
        let possible_c = char_iter.next();

        if possible_c.is_none() ||
                (possible_c.unwrap() == ' ' && !(no_wrap_positions.is_some() && no_wrap_positions.as_ref().unwrap().contains(&pos))) {
            let mut combined = String::new();
            combined.push_str(&str_buffers[current_line]);
            combined.push_str(&str_buffer_undecided);

            let width_to_check = if str_buffers.len() == 1 { width_remaining_on_current_line } else { max_width };

            if platform.get_text_dimension(&combined, font).0 < width_to_check {
                str_buffers[current_line] = combined;
            } else {
                current_line += 1;
                str_buffers.push(String::new());

                //TODO: this is ugly and slow, but for now we need to not have all new lines start with a space:
                if str_buffer_undecided.chars().next().is_some() && str_buffer_undecided.chars().next().unwrap() == ' ' {
                    str_buffer_undecided.remove(0);
                }

                str_buffers[current_line] = str_buffer_undecided;
            }
            str_buffer_undecided = String::new();
        }

        if possible_c.is_none() {
            break;
        }

        str_buffer_undecided.push(possible_c.unwrap());
        pos += 1;
    }

    return str_buffers;
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
                     parent_id: usize, platform: &mut Platform, main_url: &Url, state: &mut LayoutBuildState, optional_new_text: Option<String>) -> Rc<RefCell<LayoutNode>> {
    let mut partial_node_text_data = None;
    let mut partial_node_visible = true;
    let mut partial_node_optional_link_url = None;
    let mut partial_node_optional_img = None;
    let mut partial_node_line_break = false;
    let mut partial_node_styles = resolve_full_styles_for_layout_node(&Rc::clone(main_node), &document.all_nodes, &document.style_context);
    let mut partial_node_children = None;
    let partial_node_background_color = get_color_style_value(&partial_node_styles, "background-color").unwrap_or(Color::WHITE);

    let mut childs_to_recurse_on: &Option<Vec<Rc<RefCell<ElementDomNode>>>> = &None;

    let main_node_refcell = main_node;
    let main_node = RefCell::borrow(main_node);

    if main_node.text.is_some() {
        let partial_node_text = if optional_new_text.is_some() {
            optional_new_text.unwrap()
        } else {
            main_node.text.as_ref().unwrap().text_content.clone()
        };

        let partial_node_non_breaking_space_positions = main_node.text.as_ref().unwrap().non_breaking_space_positions.clone();
        let font = get_font_given_styles(&partial_node_styles);
        let partial_char_position_mapping = compute_char_position_mapping(platform, &font.0, &partial_node_text);

        partial_node_text_data = Some(RectTextData {
            text: partial_node_text,
            font: font.0,
            font_color: font.1,
            char_position_mapping: partial_char_position_mapping,
            non_breaking_space_positions: partial_node_non_breaking_space_positions,
        });

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
                partial_node_line_break = true;
            }

            TagName::Img => {
                if main_node.image.is_some() {
                    //TODO: eventually it would be nice to point in some cache of resources somewhere (possibly indirectly via an id if
                    //      ownership causes issues). For now we just clone every time we built the layout node.
                    partial_node_optional_img = Some(main_node.image.as_ref().unwrap().deref().clone());
                }
                childs_to_recurse_on = &None; //images should not have children (its a tag that does not have a close tag, formally)
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
                                                                          platform, main_url, state);

                        let anon_block = build_anonymous_block_layout_node(true, id_of_node_being_built, layout_childs, all_nodes, background_color);
                        partial_node_children.as_mut().unwrap().push(anon_block);

                        temp_inline_child_buffer = Vec::new();
                    }

                    state.last_char_was_space = false;
                    let layout_child = build_layout_tree(child, document, all_nodes, id_of_node_being_built, platform, main_url, state, None);
                    partial_node_children.as_mut().unwrap().push(layout_child);

                } else {
                    temp_inline_child_buffer.push(child);
                }

            }

            if !temp_inline_child_buffer.is_empty() {
                let layout_childs = build_layout_for_inline_nodes(&temp_inline_child_buffer, document, all_nodes, id_of_node_being_built,
                                                                  platform, main_url, state);

                let anon_block = build_anonymous_block_layout_node(true, id_of_node_being_built, layout_childs, all_nodes, background_color);
                partial_node_children.as_mut().unwrap().push(anon_block);
            }

        } else if get_display_type(&first_child) == Display::Inline {

            let mut inline_nodes_to_layout = Vec::new();
            for child in childs_to_recurse_on.as_ref().unwrap() {
                inline_nodes_to_layout.push(child);
            }
            let layout_childs = build_layout_for_inline_nodes(&inline_nodes_to_layout, document, all_nodes, id_of_node_being_built,
                                                              platform, main_url, state);

            for layout_child in layout_childs {
                partial_node_children.as_mut().unwrap().push(layout_child);
            }

        } else { //This means all childs are Display::Block

            for child in childs_to_recurse_on.as_ref().unwrap() {
                state.last_char_was_space = false;
                let layout_child = build_layout_tree(child, document, all_nodes, id_of_node_being_built, platform, main_url, state, None);
                partial_node_children.as_mut().unwrap().push(layout_child);
            }
        }

    }

    let layout_rect = LayoutRect {
        text_data: partial_node_text_data,
        image: partial_node_optional_img,
        location: Rect::empty(),
        selection_rect: None,
        selection_char_range: None,
    };

    let new_node = LayoutNode {
        internal_id: id_of_node_being_built,
        display: get_display_type(main_node_refcell),
        visible: partial_node_visible,
        line_break: partial_node_line_break,
        children: partial_node_children,
        parent_id: parent_id,
        optional_link_url: partial_node_optional_link_url,
        rects: vec![layout_rect],
        pre_wrap_rect_backup: None,
        from_dom_node: Some(Rc::clone(&main_node_refcell)),
        background_color: partial_node_background_color,
    };

    let rc_new_node = Rc::new(RefCell::from(new_node));
    all_nodes.insert(id_of_node_being_built, Rc::clone(&rc_new_node));

    return rc_new_node;
}


fn build_layout_for_inline_nodes(inline_nodes: &Vec<&Rc<RefCell<ElementDomNode>>>, document: &Document, all_nodes: &mut HashMap<usize, Rc<RefCell<LayoutNode>>>,
                                 parent_id: usize, platform: &mut Platform, main_url: &Url, state: &mut LayoutBuildState) -> Vec<Rc<RefCell<LayoutNode>>> {

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

        let layout_child = build_layout_tree(node, document, all_nodes, parent_id, platform, main_url, state, optional_new_text);
        layout_nodes.push(layout_child);
    }

    return layout_nodes;
}


fn build_anonymous_block_layout_node(visible: bool, parent_id: usize, inline_children: Vec<Rc<RefCell<LayoutNode>>>,
                                     all_nodes: &mut HashMap<usize, Rc<RefCell<LayoutNode>>>, background_color: Color) -> Rc<RefCell<LayoutNode>> {
    let id_of_node_being_built = get_next_layout_node_interal_id();

    let anonymous_node = LayoutNode {
        internal_id: id_of_node_being_built,
        display: Display::Block,
        visible: visible,
        line_break: false,
        children: Some(inline_children),
        parent_id: parent_id,
        optional_link_url: None,
        rects: vec![LayoutRect::get_default_non_computed_rect()],
        pre_wrap_rect_backup: None,
        from_dom_node: None,
        background_color: background_color,
    };

    let internal_id = anonymous_node.internal_id;
    let anon_rc = Rc::new(RefCell::from(anonymous_node));
    all_nodes.insert(internal_id, Rc::clone(&anon_rc));
    return anon_rc;
}
