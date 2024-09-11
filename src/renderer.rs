use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::color::Color;
use crate::layout::{FullLayout, LayoutNode};
use crate::platform::fonts::Font;
use crate::platform::Platform;
use crate::ui::{UIState, render_ui};


pub fn render(platform: &mut Platform, full_layout: &FullLayout, ui_state: &mut UIState) {
    platform.render_clear(Color::WHITE);

    render_layout_node(platform, &full_layout.root_node.borrow(), &full_layout.all_nodes, ui_state.current_scroll_y);

    render_ui(platform, ui_state);

    platform.present();
}


fn render_layout_node(platform: &mut Platform, layout_node: &LayoutNode, all_nodes: &HashMap<usize, Rc<RefCell<LayoutNode>>>, current_scroll_y: f32) {

    if !layout_node.rects.iter().any(|rect| -> bool { rect.location.is_visible_on_y_location(current_scroll_y) }) {
        return;
    }

    if layout_node.is_submit_button {
        let layout_rect = layout_node.rects.iter().next().unwrap();
        let render_y = layout_rect.location.y - current_scroll_y;

        //TODO: to render the button (and textfield etc.) we would like to defer to the component. Do we store the component on the DOM?

        //TODO: temp debug rendering:
        platform.render_text(&"[SUBMIT BUTTON]".to_owned(), layout_rect.location.x, render_y, &Font::default(), Color::BLACK);
        return;
    }

    if layout_node.is_text_input {
        let layout_rect = layout_node.rects.iter().next().unwrap();
        let render_y = layout_rect.location.y - current_scroll_y;

        //TODO: temp debug rendering:
        platform.render_text(&"[TEXT INPUT]".to_owned(), layout_rect.location.x, render_y, &Font::default(), Color::BLACK);
        return;
    }

    for layout_rect in layout_node.rects.iter() {

        if layout_node.background_color != Color::WHITE {
            let location = &layout_rect.location;
            platform.fill_rect(location.x, location.y - current_scroll_y, location.width, location.height, layout_node.background_color, 255);
        }

        if layout_rect.text_data.is_some() {
            if layout_rect.selection_rect.is_some() {
                let selection_rect = layout_rect.selection_rect.as_ref().unwrap();
                platform.fill_rect(selection_rect.x, selection_rect.y - current_scroll_y, selection_rect.width, selection_rect.height, Color::DEFAULT_SELECTION_COLOR, 255);
            }

            let render_y = layout_rect.location.y - current_scroll_y;
            let text_data = layout_rect.text_data.as_ref().unwrap();
            platform.render_text(&text_data.text, layout_rect.location.x, render_y, &text_data.font, text_data.font_color);
        }
    }

    let possible_img_rect = layout_node.rects.iter().find(|rect| { rect.image.is_some()});
    if possible_img_rect.is_some() {
        debug_assert!(layout_node.rects.len() == 1);
        let rects = &layout_node.rects;
        let rect = rects.iter().next().unwrap();
        platform.render_image(&possible_img_rect.unwrap().image.as_ref().unwrap(), rect.location.x, rect.location.y - current_scroll_y);
    }

    if layout_node.children.is_some() {
        for child in layout_node.children.as_ref().unwrap() {
            if child.borrow().visible {
                render_layout_node(platform, &child.borrow(), all_nodes, current_scroll_y);
            }
        }
    }
}
