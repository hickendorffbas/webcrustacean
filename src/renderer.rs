use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::color::Color;
use crate::layout::{
    FullLayout,
    LayoutNode,
    get_font_given_styles
};
use crate::platform::Platform;
use crate::style::get_color_style_value;
use crate::ui::{UIState, render_ui};


const CURSOR_BLINK_SPEED_MILLIS: u128 = 500;


pub fn render(platform: &mut Platform, full_layout: &FullLayout, ui_state: &mut UIState) {
    platform.render_clear(Color::WHITE);
    update_animation_state(ui_state);

    render_layout_node(platform, &full_layout.root_node.borrow(), &full_layout.all_nodes, ui_state.current_scroll_y);

    debug_assert!(full_layout.root_node.borrow().rects.len() == 1);
    let page_height = full_layout.root_node.borrow().rects.first().unwrap().location.height;

    render_ui(platform, ui_state, page_height);

    platform.present();
}


fn update_animation_state(ui_state: &mut UIState) {

    //TODO: this code is ok, but should live in an update() method on TextField struct, and we should have a way to call updates on components
    //      exactly once per frame from somewhere. This renderer module seems to be more about rendering web content, so I think it should move to ui.rs
    let current_millis = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_millis();
    let cursor_cycle_time = CURSOR_BLINK_SPEED_MILLIS * 2;
    let point_in_cyle = current_millis % cursor_cycle_time;
    ui_state.addressbar.cursor_visible = point_in_cyle > CURSOR_BLINK_SPEED_MILLIS;
}


fn render_layout_node(platform: &mut Platform, layout_node: &LayoutNode, all_nodes: &HashMap<usize, Rc<RefCell<LayoutNode>>>, current_scroll_y: f32) {

    if !layout_node.rects.iter().any(|rect| -> bool { rect.location.is_visible_on_y_location(current_scroll_y) }) {
        return;
    }

    for layout_rect in layout_node.rects.iter() {
        if layout_rect.text.is_some() {
            let (font, font_color) = get_font_given_styles(&layout_node.styles);
            let render_y = layout_rect.location.y - current_scroll_y;
            platform.render_text(layout_rect.text.as_ref().unwrap(), layout_rect.location.x, render_y, &font, font_color);
        } else {
            let background_color = get_color_style_value(&layout_node.styles, "background-color");
            if background_color.is_some() {
                let location = &layout_rect.location;
                platform.fill_rect(location.x, location.y - current_scroll_y, location.width, location.height, background_color.unwrap());
            }
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
