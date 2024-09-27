use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::color::Color;
use crate::layout::{
    FullLayout,
    LayoutNode,
    LayoutNodeContent
};
use crate::platform::{fonts::Font, Platform};
use crate::ui::{UIState, render_ui};


pub fn render(platform: &mut Platform, full_layout: &FullLayout, ui_state: &mut UIState) {
    platform.render_clear(Color::WHITE);

    render_layout_node(platform, &full_layout.root_node.borrow(), &full_layout.all_nodes, ui_state.current_scroll_y);

    render_ui(platform, ui_state);

    platform.present();
}


fn render_layout_node(platform: &mut Platform, layout_node: &LayoutNode, all_nodes: &HashMap<usize, Rc<RefCell<LayoutNode>>>, current_scroll_y: f32) {

    if !layout_node.visible_on_y_location(current_scroll_y) {
        return;
    }

    match &layout_node.content {
        LayoutNodeContent::TextLayoutNode(text_layout_node) => {
            for layout_rect in text_layout_node.rects.iter() {

                if text_layout_node.background_color != Color::WHITE {
                    let location = &layout_rect.location;
                    platform.fill_rect(location.x, location.y - current_scroll_y, location.width, location.height, text_layout_node.background_color, 255);
                }

                if layout_rect.selection_rect.is_some() {
                    let selection_rect = layout_rect.selection_rect.as_ref().unwrap();
                    platform.fill_rect(selection_rect.x, selection_rect.y - current_scroll_y, selection_rect.width, selection_rect.height, Color::DEFAULT_SELECTION_COLOR, 255);
                }

                let render_y = layout_rect.location.y - current_scroll_y;
                platform.render_text(&layout_rect.text, layout_rect.location.x, render_y, &layout_rect.font, layout_rect.font_color);
            }
        },
        LayoutNodeContent::ImageLayoutNode(image_layout_node) => {
            platform.render_image(&image_layout_node.image, image_layout_node.location.x, image_layout_node.location.y - current_scroll_y);
        },
        LayoutNodeContent::ButtonLayoutNode(button_layout_node) => {
            let render_y = button_layout_node.location.y - current_scroll_y;

            //TODO: to render the button (and textfield etc.) we would like to defer to the component. Do we store the component on the DOM?

            //TODO: temp debug rendering:
            platform.render_text(&"[SUBMIT BUTTON]".to_owned(), button_layout_node.location.x, render_y, &Font::default(), Color::BLACK);
        },
        LayoutNodeContent::TextInputLayoutNode(text_input_layout_node) => {
            let render_y = text_input_layout_node.location.y - current_scroll_y;

            //TODO: temp debug rendering:
            platform.render_text(&"[TEXT INPUT]".to_owned(), text_input_layout_node.location.x, render_y, &Font::default(), Color::BLACK);
        },
        LayoutNodeContent::BoxLayoutNode(box_node) => {
            if box_node.background_color != Color::WHITE { //TODO: don't think this check is correct (also for text nodes,
                                                           //      because you can have this inside another colored node)
                let location = &box_node.location;
                platform.fill_rect(location.x, location.y - current_scroll_y, location.width, location.height, box_node.background_color, 255);
            }
        },
        LayoutNodeContent::NoContent => {},
    }

    if layout_node.children.is_some() {
        for child in layout_node.children.as_ref().unwrap() {
            if child.borrow().visible {
                render_layout_node(platform, &child.borrow(), all_nodes, current_scroll_y);
            }
        }
    }
}
