use std::collections::HashMap;
use std::rc::Rc;

use crate::color::Color;
use crate::layout::{
    FullLayout,
    LayoutNode,
    get_font_given_styles
};
use crate::platform::Platform;
use crate::style::resolve_full_styles_for_layout_node;
use crate::ui::render_ui;


pub fn render(platform: &mut Platform, full_layout: &FullLayout, current_scroll_y: f32) {
    platform.render_clear(Color::WHITE);

    render_ui(platform);

    render_layout_node(platform, &full_layout.root_node, &full_layout.all_nodes, current_scroll_y);

    platform.present();
}


fn render_layout_node(platform: &mut Platform, layout_node: &LayoutNode, all_nodes: &HashMap<usize, Rc<LayoutNode>>, current_scroll_y: f32) {
    let resolved_styles = resolve_full_styles_for_layout_node(&layout_node, all_nodes);

    if !layout_node.rects.borrow().iter().any(|rect| -> bool { rect.location.borrow().is_visible_on_y_location(current_scroll_y) }) {
        return;
    }

    for layout_rect in layout_node.rects.borrow().iter() {
        if layout_rect.text.is_some() {
            let (font, font_color) = get_font_given_styles(&resolved_styles);
            let (x, y) = layout_rect.location.borrow().x_y_as_int();

            platform.render_text(layout_rect.text.as_ref().unwrap(), x, y  - current_scroll_y as u32, &font, font_color);
        }
    }

    if layout_node.children.is_some() {
        for child in layout_node.children.as_ref().unwrap() {
            if child.visible {
                render_layout_node(platform, &child, all_nodes, current_scroll_y);
            }
        }
    }
}
