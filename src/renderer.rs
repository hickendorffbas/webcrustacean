use std::ops::Deref;

use crate::color::Color;
use crate::layout::{
    FullLayout,
    LayoutNode,
    LayoutNodeContent
};
use crate::platform::Platform;
use crate::ui::{UIState, render_ui};
use crate::ui_components::PageComponent;


pub const CULLING_MARGIN: f32 = 0.0;


pub fn render(platform: &mut Platform, full_layout: &FullLayout, ui_state: &mut UIState) {
    platform.render_clear(Color::WHITE);

    render_layout_node(platform, ui_state, &full_layout.root_node.borrow());

    render_ui(platform, ui_state);

    platform.present();
}


fn render_layout_node(platform: &mut Platform, ui_state: &mut UIState, layout_node: &LayoutNode) {
    let scroll_x = ui_state.current_scroll_x;
    let scroll_y = ui_state.current_scroll_y;

    if !layout_node.visible_on_location(scroll_x, scroll_y, ui_state.window_dimensions.content_viewport_width + CULLING_MARGIN,
                                        ui_state.window_dimensions.content_viewport_height + CULLING_MARGIN) {
        return;
    }

    match &layout_node.content {
        LayoutNodeContent::TextLayoutNode(text_layout_node) => {
            for css_text_box in text_layout_node.css_text_boxes.iter() {

                if text_layout_node.background_color != Color::WHITE {
                    let location = &css_text_box.css_box;
                    platform.fill_rect(location.x - scroll_x, location.y - scroll_y, location.width, location.height, text_layout_node.background_color, 255);
                }

                if css_text_box.selection_rect.is_some() {
                    let selection_rect = css_text_box.selection_rect.as_ref().unwrap();
                    platform.fill_rect(selection_rect.x - scroll_x, selection_rect.y - scroll_y, selection_rect.width, selection_rect.height, Color::DEFAULT_SELECTION_COLOR, 255);
                }

                let render_x = css_text_box.css_box.x - scroll_x;
                let render_y = css_text_box.css_box.y - scroll_y;
                platform.render_text(&css_text_box.text, render_x, render_y, &text_layout_node.font, text_layout_node.font_color);
            }
        },
        LayoutNodeContent::ImageLayoutNode(image_layout_node) => {
            platform.enable_blending();
            platform.render_image(&image_layout_node.image, image_layout_node.css_box.x - scroll_x, image_layout_node.css_box.y - scroll_y);
            if image_layout_node.selected {
                platform.fill_rect(image_layout_node.css_box.x - scroll_x, image_layout_node.css_box.y - scroll_y, image_layout_node.image.width() as f32,
                                   image_layout_node.image.height() as f32, Color::DEFAULT_SELECTION_COLOR, 100);
            }
            platform.disable_blending();
        },
        LayoutNodeContent::ButtonLayoutNode(_) => {
            let dom_node = layout_node.from_dom_node.as_ref().unwrap().borrow();
            let component = dom_node.page_component.as_ref().unwrap().borrow();
            match component.deref() {
                PageComponent::Button(button) => { button.render(platform, scroll_x, scroll_y); }
                PageComponent::TextField(_) => { panic!("Invalid state"); }
            }
        },
        LayoutNodeContent::TextInputLayoutNode(_) => {
            let dom_node = layout_node.from_dom_node.as_ref().unwrap().borrow();
            let component = dom_node.page_component.as_ref().unwrap().borrow();
            match component.deref() {
                PageComponent::Button(_) => { panic!("Invalid state"); }
                PageComponent::TextField(text_field) => { text_field.render(ui_state, platform, scroll_x, scroll_y); }
            }
        },
        LayoutNodeContent::AreaLayoutNode(area_node) => {
            if area_node.background_color != Color::WHITE { //TODO: don't think this check is correct (also for text nodes,
                                                           //      because you can have this inside another colored node)
                let css_box = &area_node.css_box;
                platform.fill_rect(css_box.x - scroll_x, css_box.y - scroll_y, css_box.width, css_box.height, area_node.background_color, 255);
            }
        },
        LayoutNodeContent::TableLayoutNode(_) => {
            //eventually we might have something to render here, like a border or something (or is that also on cell level?)
            //for now we render nothing
        }
        LayoutNodeContent::TableCellLayoutNode(_) => {
            //TODO: implement (is there anything to render here aside from potential borders in the future?)
            //for now we render nothing
        }
        LayoutNodeContent::NoContent => {},
    }

    if layout_node.children.is_some() {
        for child in layout_node.children.as_ref().unwrap() {
            if child.borrow().visible {
                render_layout_node(platform, ui_state, &child.borrow());
            }
        }
    }
}
