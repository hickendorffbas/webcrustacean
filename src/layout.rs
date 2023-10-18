use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use image::DynamicImage;

use crate::{
    Font,
    resource_loader,
    SCREEN_HEIGHT
};
use crate::color::Color;
use crate::debug::debug_log_warn;
use crate::dom::{Document, DomNode};
use crate::network::url::Url;
use crate::platform::Platform;
use crate::style::{
    StyleContext,
    get_color_style_value,
    get_numeric_style_value,
    has_style_value,
    resolve_full_styles_for_layout_node,
};
use crate::ui::{
    CONTENT_TOP_LEFT_X,
    CONTENT_TOP_LEFT_Y,
    CONTENT_WIDTH
};


static NEXT_LAYOUT_NODE_INTERNAL: AtomicUsize = AtomicUsize::new(1);
pub fn get_next_layout_node_interal_id() -> usize { NEXT_LAYOUT_NODE_INTERNAL.fetch_add(1, Ordering::Relaxed) }


pub struct FullLayout {
    pub root_node: Rc<LayoutNode>,
    pub all_nodes: HashMap<usize, Rc<LayoutNode>>,
}
impl FullLayout {
    pub fn page_height(&self) -> f32 {
        return self.root_node.rects.borrow().iter().next().unwrap().location.borrow().height();
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct LayoutNode {
    pub internal_id: usize,
    pub display: Display, //TODO: eventually we don't want every css construct as a member on this struct ofcourse (TODO: we have the styles member now, use that)
    pub visible: bool,
    pub line_break: bool,
    pub children: Option<Vec<Rc<LayoutNode>>>,
    pub parent_id: usize,
    pub styles: HashMap<String, String>,  //TODO: it would eventually be nice to have something stronger typed here
    pub optional_link_url: Option<Url>,
    pub rects: RefCell<Vec<LayoutRect>>,
    pub from_dom_node: Option<Rc<DomNode>>,
}
impl LayoutNode {
    pub fn all_childnodes_have_given_display(&self, display: Display) -> bool {
        if self.children.is_none() {
            return true;
        }
        return self.children.as_ref().unwrap().iter().all(|node| node.display == display);
    }
    pub fn update_single_rect_location(&self, new_location: ComputedLocation) {
        debug_assert!(self.rects.borrow().len() == 1);
        self.rects.borrow()[0].location.replace(new_location);
    }
    pub fn can_wrap(&self) -> bool {
        return self.rects.borrow().iter().any(|rect| { rect.text.is_some()});
    }
    pub fn get_size_of_bounding_box(&self) -> (f32, f32) {
        let mut lowest_x = f32::MAX;
        let mut lowest_y = f32::MAX;
        let mut max_x: f32 = 0.0;
        let mut max_y: f32 = 0.0;

        for rect in self.rects.borrow().iter() {
            let rect_loc = rect.location.borrow();
            lowest_x = lowest_x.min(rect_loc.x());
            lowest_y = lowest_y.min(rect_loc.y());
            max_x = max_x.max(rect_loc.x() + rect_loc.width());
            max_y = max_y.max(rect_loc.y() + rect_loc.height());
        }
        let bounding_box_width = max_x - lowest_x;
        let bounding_box_height = max_y - lowest_y;
        return (bounding_box_width, bounding_box_height);
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct LayoutRect {
    pub text: Option<String>,
    pub non_breaking_space_positions: Option<HashSet<usize>>, //TODO: might be nice to combine this with text_content in a text struct
    pub image: Option<DynamicImage>,
    pub location: RefCell<ComputedLocation>,
}
impl LayoutRect {
    pub fn get_default_non_computed_rect() -> LayoutRect {
        return LayoutRect {
            text: None,
            non_breaking_space_positions: None,
            image: None,
            location: RefCell::new(ComputedLocation::NotYetComputed),
        };
    }
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub enum ComputedLocation {
    NotYetComputed,
    Computed(Rect)
}
impl ComputedLocation {
    pub fn x_y(&self) -> (f32, f32) {
        return match self {
            ComputedLocation::NotYetComputed => panic!("Node has not yet been computed"),
            ComputedLocation::Computed(loc) => { (loc.x, loc.y) },
        }
    }
    pub fn x(&self) -> f32 {
        return match self {
            ComputedLocation::NotYetComputed => panic!("Node has not yet been computed"),
            ComputedLocation::Computed(loc) => { loc.x },
        }
    }
    pub fn y(&self) -> f32 {
        return match self {
            ComputedLocation::NotYetComputed => panic!("Node has not yet been computed"),
            ComputedLocation::Computed(loc) => { loc.y },
        }
    }
    pub fn width(&self) -> f32 {
        return match self {
            ComputedLocation::NotYetComputed => panic!("Node has not yet been computed"),
            ComputedLocation::Computed(loc) => { loc.width },
        }
    }
    pub fn height(&self) -> f32 {
        return match self {
            ComputedLocation::NotYetComputed => panic!("Node has not yet been computed"),
            ComputedLocation::Computed(loc) => { loc.height },
        }
    }
    pub fn is_inside(&self, x: f32, y: f32) -> bool {
        //TODO: for now we use this to check pixel values, but we actually should convert units properly somewhere (before the renderer, I guess)
        //      in general we need to do a pass on using correct units everywhere
        return match self {
            ComputedLocation::NotYetComputed => panic!("Node has not yet been computed"),
            ComputedLocation::Computed(loc) => {
                x >= loc.x && x <= loc.x + loc.width
                &&
                y >= loc.y && y <= loc.y + loc.height
            },
        }
    }
    pub fn is_visible_on_y_location(&self, y: f32) -> bool {
        return match self {
            ComputedLocation::NotYetComputed => panic!("Node has not yet been computed"),
            ComputedLocation::Computed(loc) => {
                let top_of_node = loc.y;
                let top_of_view = y;
                let bottom_of_node = top_of_node + loc.height;
                let bottom_of_view = top_of_view + SCREEN_HEIGHT;

                !(top_of_node > bottom_of_view || bottom_of_node < top_of_view)
            }
        }
    }
}


#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(PartialEq)]
pub enum Display { //TODO: this is a CSS property, of which we will have many, we should probably define those somewhere else
    Block,
    Inline
}


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}


pub fn build_full_layout(document: &Document, platform: &mut Platform, main_url: &Url) -> FullLayout {
    let mut top_level_layout_nodes: Vec<Rc<LayoutNode>> = Vec::new();
    let mut all_nodes: HashMap<usize, Rc<LayoutNode>> = HashMap::new();

    let id_of_node_being_built = get_next_layout_node_interal_id();

    let document_layout_node = build_layout_tree(&document.document_node, document, &mut all_nodes, id_of_node_being_built, platform, main_url);
    top_level_layout_nodes.push(document_layout_node);

    let root_node = LayoutNode {
        internal_id: id_of_node_being_built,
        display: Display::Block,
        visible: true,
        line_break: false,
        children: Some(top_level_layout_nodes),
        parent_id: id_of_node_being_built,  //this is the top node, so it does not really have a parent, we set it to ourselves,
        styles: resolve_full_styles_for_layout_node(&Rc::clone(&document.document_node), &document.all_nodes, &document.style_context),
        optional_link_url: None,
        rects: RefCell::new(vec![LayoutRect::get_default_non_computed_rect()]),
        from_dom_node: Some(Rc::clone(&document.document_node)),
    };

    let rc_root_node = Rc::new(root_node);
    all_nodes.insert(id_of_node_being_built, Rc::clone(&rc_root_node));

    compute_layout(&rc_root_node, &all_nodes, &document.style_context, CONTENT_TOP_LEFT_X, CONTENT_TOP_LEFT_Y, platform);
    let (root_width, root_height) = rc_root_node.get_size_of_bounding_box();
    let root_location = ComputedLocation::Computed(
        Rect { x: CONTENT_TOP_LEFT_X, y: CONTENT_TOP_LEFT_Y, width: root_width, height: root_height }
    );
    rc_root_node.update_single_rect_location(root_location);

    return FullLayout { root_node: rc_root_node, all_nodes }
}


//This function is responsible for setting the location rects on the node, and all its children.
//TODO: need to find a way to make good tests for this (maybe just reftests?)
fn compute_layout(node: &LayoutNode, all_nodes: &HashMap<usize, Rc<LayoutNode>>, style_context: &StyleContext,
                  top_left_x: f32, top_left_y: f32, platform: &mut Platform) {

    if !node.visible {
        let node_location = ComputedLocation::Computed(
            Rect { x: top_left_x, y: top_left_y, width: 0.0, height: 0.0 }
        );
        node.update_single_rect_location(node_location);
    }

    if node.children.is_some() {
        let all_block = node.all_childnodes_have_given_display(Display::Block);
        let all_inline = node.all_childnodes_have_given_display(Display::Inline);

        if all_block {
            return apply_block_layout(node, all_nodes, style_context, top_left_x, top_left_y, platform);
        }
        if all_inline {
            return apply_inline_layout(node, all_nodes, style_context, top_left_x, top_left_y, CONTENT_WIDTH - top_left_x, platform);
        }

        panic!("Not all children are either inline or block, earlier in the process this should already have been fixed with anonymous blocks");
    }

    for rect in node.rects.borrow().iter() {
        let (rect_width, rect_height) = compute_size_for_rect(rect, &node.styles, platform);
        let rect_location = ComputedLocation::Computed(
            Rect { x: top_left_x, y: top_left_y, width: rect_width, height: rect_height }
        );
        rect.location.replace(rect_location);
    }
}


pub fn get_font_given_styles(styles: &HashMap<String, String>) -> (Font, Color) {
    let font_bold = has_style_value(&styles, "font-weight", &"bold".to_owned());
    let font_underline = has_style_value(&styles, "text-decoration", &"underline".to_owned());
    let font_size = get_numeric_style_value(&styles, "font-size")
                        .expect("No font-size found"); //font-size should be in the default styles, so this is a fatal error if not found
    let font_color = get_color_style_value(&styles, "color")
                        .expect(format!("Unkown color").as_str()); //TODO: we need to handle this in a graceful way, instead of crashing

    return (Font::new(font_bold, font_underline, font_size), font_color);
}


fn apply_block_layout(node: &LayoutNode, all_nodes: &HashMap<usize, Rc<LayoutNode>>, style_context: &StyleContext,
                      top_left_x: f32, top_left_y: f32, platform: &mut Platform) {
    let mut cursor_y = top_left_y;
    let mut max_width: f32 = 0.0;

    for child in node.children.as_ref().unwrap() {
        compute_layout(child, all_nodes, style_context, top_left_x, cursor_y, platform);
        let (bounding_box_width, bounding_box_height) = child.get_size_of_bounding_box();

        cursor_y += bounding_box_height;
        max_width = max_width.max(bounding_box_width);
    }

    let our_height = cursor_y - top_left_y;
    let node_location = ComputedLocation::Computed(
        Rect { x: top_left_x, y: top_left_y, width: max_width, height: our_height }
    );
    node.update_single_rect_location(node_location);
}


fn apply_inline_layout(node: &LayoutNode, all_nodes: &HashMap<usize, Rc<LayoutNode>>, style_context: &StyleContext, top_left_x: f32, top_left_y: f32,
                       max_allowed_width: f32, platform: &mut Platform) {
    let mut cursor_x = top_left_x;
    let mut cursor_y = top_left_y;
    let mut max_width: f32 = 0.0;
    let mut max_height_of_line: f32 = 0.0;

    for child in node.children.as_ref().unwrap() {

        compute_layout(child, all_nodes, style_context, cursor_x, cursor_y, platform);

        if child.line_break {
            let child_height;
            if cursor_x != top_left_x {
                cursor_x = top_left_x;
                cursor_y += max_height_of_line;
                child_height = max_height_of_line;
            } else {
                let (font, _) = get_font_given_styles(&child.styles);
                //we get the hight of a random character in the font for the height of the newline:
                let (_, random_char_height) = platform.get_text_dimension(&String::from("x"), &font);

                cursor_x = top_left_x;
                cursor_y += random_char_height;
                child_height = random_char_height;
            }

            let child_location = ComputedLocation::Computed(
                Rect { x: top_left_x, y: top_left_y, width: max_width, height: child_height }
            );
            child.update_single_rect_location(child_location);

            continue;
        }

        let (child_width, child_height) = child.get_size_of_bounding_box();

        if (cursor_x - top_left_x + child_width) > max_allowed_width {

            if child.children.is_none() && child.can_wrap() && child.rects.borrow().iter().all(|rect| -> bool { rect.text.is_some()} ) {
                // in this case, we might be able to split rects, and put part of the node on this line

                let font = get_font_given_styles(&child.styles);
                let relative_cursor_x = cursor_x - top_left_x;
                let amount_of_space_left_on_line = max_allowed_width - relative_cursor_x;
                let wrapped_text = wrap_text(child.rects.borrow().last().unwrap(), max_allowed_width, 
                                             amount_of_space_left_on_line, &font.0, platform);

                let mut rects_for_child = Vec::new();
                for text in wrapped_text {

                    let new_rect = LayoutRect {
                        text: Some(text),
                        non_breaking_space_positions: None, //For now not computing these, although it would be more correct to update them after wrapping
                        image: None,
                        location: RefCell::new(ComputedLocation::NotYetComputed),
                    };

                    let (rect_width, rect_height) = compute_size_for_rect(&new_rect, &child.styles, platform);

                    if cursor_x - top_left_x + rect_width > max_allowed_width {
                        if cursor_x != top_left_x {
                            cursor_x = top_left_x;
                            cursor_y += max_height_of_line;
                            max_height_of_line = 0.0;
                        }
                    }

                    new_rect.location.replace(ComputedLocation::Computed(
                        Rect { x: cursor_x, y: cursor_y, width: rect_width, height: rect_height }
                    ));
                    rects_for_child.push(new_rect);

                    cursor_x += rect_width;
                    max_width = max_width.max(cursor_x);
                    max_height_of_line = max_height_of_line.max(rect_height);

                }
                child.rects.replace(rects_for_child);

            } else {
                if cursor_x != top_left_x {
                    //we can move to a new line, it might fit there

                    cursor_x = top_left_x;
                    cursor_y += max_height_of_line;

                    compute_layout(child, all_nodes, style_context, cursor_x, cursor_y, platform);
                    let (child_width, child_height) = child.get_size_of_bounding_box();

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

    let node_location = ComputedLocation::Computed(
        Rect { x: top_left_x, y: top_left_y, width: max_width, height: our_height }
    );
    node.update_single_rect_location(node_location);
}


//Note that this function returns the size, but does not update the rect with that size (because we also need to position for the computed location object)
fn compute_size_for_rect(layout_rect: &LayoutRect, styles: &HashMap<String, String>, platform: &mut Platform) -> (f32, f32) {

    if layout_rect.text.is_some() {
        let font = get_font_given_styles(styles);
        return platform.get_text_dimension(layout_rect.text.as_ref().unwrap(), &font.0);
    }

    if layout_rect.image.is_some() {
        let img = layout_rect.image.as_ref().unwrap();
        return (img.width() as f32, img.height() as f32);
    }

    //we panic here, because we only expect to be this this method for rects of layoutnodes that don't have children, otherwise we should compute via the sizes of the children:
    panic!("We don't know how to compute the size of rects without content, they should be computed via their children")
}


fn wrap_text(layout_rect: &LayoutRect, max_width: f32, width_remaining_on_current_line: f32, font: &Font, platform: &mut Platform) -> Vec<String> {
    let text = layout_rect.text.as_ref();
    let no_wrap_positions = &layout_rect.non_breaking_space_positions;
    let mut str_buffers = Vec::new();
    let mut str_buffer_undecided = String::new();
    let mut pos = 0;
    let mut current_line = 0;

    str_buffers.push(String::new());

    let mut char_iter = text.unwrap().chars();
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
                if str_buffer_undecided.chars().next().unwrap() == ' ' {
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


fn build_layout_tree(main_node: &Rc<DomNode>, document: &Document, all_nodes: &mut HashMap<usize, Rc<LayoutNode>>,
                     parent_id: usize, platform: &Platform, main_url: &Url) -> Rc<LayoutNode> {
    let mut partial_node_text = None;
    let mut partial_node_non_breaking_space_positions = None;
    let mut partial_node_visible = true;
    let mut partial_node_optional_link_url = None;
    let mut partial_node_optional_img = None;
    let mut partial_node_line_break = false;
    let mut partial_node_display = Display::Block;
    let mut partial_node_styles = resolve_full_styles_for_layout_node(&Rc::clone(main_node), &document.all_nodes, &document.style_context);

    let mut childs_to_recurse_on: &Option<Vec<Rc<DomNode>>> = &None;
    match main_node.as_ref() {
        DomNode::Document(node) => {
            childs_to_recurse_on = &node.children;
        },
        DomNode::Element(node) => {
            childs_to_recurse_on = &node.children;

            match &node.name.as_ref().unwrap()[..] {

                "a" => {
                    let opt_href = node.get_attribute_value("href");
                    if opt_href.is_some() {
                        partial_node_optional_link_url = Some(Url::from_base_url(&opt_href.unwrap(), Some(main_url)));
                    } else {
                        partial_node_optional_link_url = None;
                    }

                    partial_node_display = Display::Inline;
                }

                "b" => {
                    partial_node_styles.insert("font-weight".to_owned(), "bold".to_owned());
                    partial_node_display = Display::Inline;
                }

                "br" => {
                    partial_node_line_break = true;
                    partial_node_display = Display::Inline;
                }

                "body" => {
                    //for now this needs the default for all fields
                }

                "div" =>  {
                    //for now this needs the default for all fields
                }

                "h1"|"h2"|"h3"|"h4"|"h5"|"h6" => {
                    //for now this needs the default for all fields
                }

                "head" => {
                    //for now this needs the default for all fields
                }

                "html" => {
                    //for now this needs the default for all fields
                }

                "img" => {
                    let image_src = node.get_attribute_value("src").expect("can't handle img without src yet..."); //TODO: handle the un-expect'ed case
                    let image_url = Url::from_base_url(&image_src, Some(main_url));
                    partial_node_optional_img = Some(resource_loader::load_image(&image_url));

                    partial_node_display = Display::Inline;

                    childs_to_recurse_on = &None; //images should not have children (its a tag that does not have a close tag, formally)
                }

                "p" =>  {

                }

                //TODO: this one might not be neccesary any more after we fix our html parser to not try to parse the javascript
                "script" => { partial_node_visible = false; }

                //TODO: same as for "script", do these need nodes in the DOM? probably not
                "style" => { partial_node_visible = false; }

                //TODO: eventually we want to do something else with the title (update the window title or so)
                "title" => { partial_node_visible = false; }

                default => {
                    debug_log_warn(format!("unknown tag: {}", default));
                }
            }
        }
        DomNode::Attribute(_) => {
            //We should always handle these in their parents nodes
            panic!("We should never have to handle attributes by themselves")
        },
        DomNode::Text(node) => {
            partial_node_text = Option::Some(node.text_content.to_string());
            partial_node_non_breaking_space_positions = node.non_breaking_space_positions.clone();
            partial_node_display = Display::Inline;
        }

    }

    let id_of_node_being_built = get_next_layout_node_interal_id();

    let new_childeren = if let Some(ref children) = childs_to_recurse_on {
        let mut temp_children = Vec::new();

        for child in children {
            match child.as_ref() { DomNode::Attribute(_) => { continue; }, _ => {} }

            temp_children.push(build_layout_tree(child, document, all_nodes, id_of_node_being_built, platform, main_url));
        }

        let all_display_types = temp_children.iter().map(|child| &child.display).collect::<Vec<&Display>>();

        if all_display_types.contains(&&Display::Inline) && all_display_types.contains(&&Display::Block) {
            let mut temp_children_with_anonymous_blocks = Vec::new();
            let mut temp_buffer_for_inline_children = Vec::new();

            for child in temp_children {
                match child.display {
                    Display::Block => {
                        if temp_buffer_for_inline_children.len() > 0 {
                            let anonymous_block_node = build_anonymous_block_layout_node(partial_node_visible, id_of_node_being_built,
                                                                                         temp_buffer_for_inline_children, all_nodes, &partial_node_styles);

                            temp_children_with_anonymous_blocks.push(anonymous_block_node);
                            temp_buffer_for_inline_children = Vec::new();
                        }

                        temp_children_with_anonymous_blocks.push(child);
                    },
                    Display::Inline => { temp_buffer_for_inline_children.push(child); },
                }
            }

            if temp_buffer_for_inline_children.len() > 0 {
                let anonymous_block_node = build_anonymous_block_layout_node(partial_node_visible, id_of_node_being_built,
                                                                             temp_buffer_for_inline_children, all_nodes, &partial_node_styles);
                temp_children_with_anonymous_blocks.push(anonymous_block_node);
            }

            Some(temp_children_with_anonymous_blocks)
        } else {
            Some(temp_children)
        }
    } else {
        None
    };

    let layout_rect = LayoutRect {
        text: partial_node_text,
        non_breaking_space_positions: partial_node_non_breaking_space_positions,
        image: partial_node_optional_img,
        location: RefCell::new(ComputedLocation::NotYetComputed),
    };

    let new_node = LayoutNode {
        internal_id: id_of_node_being_built,
        display: partial_node_display,
        visible: partial_node_visible,
        line_break: partial_node_line_break,
        children: new_childeren,
        parent_id: parent_id,
        styles: partial_node_styles,
        optional_link_url: partial_node_optional_link_url,
        rects: RefCell::new(vec![layout_rect]),
        from_dom_node: Some(Rc::clone(main_node)),
    };

    let rc_new_node = Rc::new(new_node);
    all_nodes.insert(id_of_node_being_built, Rc::clone(&rc_new_node));

    return rc_new_node;
}


fn build_anonymous_block_layout_node(visible: bool, parent_id: usize, inline_children: Vec<Rc<LayoutNode>>,
                                     all_nodes: &mut HashMap<usize, Rc<LayoutNode>>, styles: &HashMap<String, String>) -> Rc<LayoutNode> {
    let id_of_node_being_built = get_next_layout_node_interal_id();

    let anonymous_node = LayoutNode {
        internal_id: id_of_node_being_built,
        display: Display::Block,
        visible: visible,
        line_break: false,
        children: Some(inline_children),
        parent_id: parent_id,
        styles: styles.clone(),
        optional_link_url: None,
        rects: RefCell::new(vec![LayoutRect::get_default_non_computed_rect()]),
        from_dom_node: None,
    };

    let anon_rc = Rc::new(anonymous_node);
    all_nodes.insert(anon_rc.internal_id, Rc::clone(&anon_rc));
    return anon_rc;
}
