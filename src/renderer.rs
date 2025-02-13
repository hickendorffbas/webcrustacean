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


pub fn render(platform: &mut Platform, full_layout: &FullLayout, ui_state: &mut UIState) {
    platform.render_clear(Color::WHITE);

    render_layout_node(platform, ui_state, &full_layout.root_node.borrow());

    render_ui(platform, ui_state);

    platform.present();
}


fn render_layout_node(platform: &mut Platform, ui_state: &mut UIState, layout_node: &LayoutNode) {
    let scroll_y = ui_state.current_scroll_y;

    if !layout_node.visible_on_y_location(scroll_y) {
        return;
    }

    match &layout_node.content {
        LayoutNodeContent::TextLayoutNode(text_layout_node) => {
            for layout_rect in text_layout_node.rects.iter() {

                if text_layout_node.background_color != Color::WHITE {
                    let location = &layout_rect.location;
                    platform.fill_rect(location.x, location.y - scroll_y, location.width, location.height, text_layout_node.background_color, 255);
                }

                if layout_rect.selection_rect.is_some() {
                    let selection_rect = layout_rect.selection_rect.as_ref().unwrap();
                    platform.fill_rect(selection_rect.x, selection_rect.y - scroll_y, selection_rect.width, selection_rect.height, Color::DEFAULT_SELECTION_COLOR, 255);
                }

                let render_y = layout_rect.location.y - scroll_y;
                platform.render_text(&layout_rect.text, layout_rect.location.x, render_y, &layout_rect.font, layout_rect.font_color);
            }
        },
        LayoutNodeContent::ImageLayoutNode(image_layout_node) => {
            platform.render_image(&image_layout_node.image, image_layout_node.location.x, image_layout_node.location.y - scroll_y);
        },
        LayoutNodeContent::ButtonLayoutNode(_) => {
            let dom_node = layout_node.from_dom_node.as_ref().unwrap().borrow();
            let component = dom_node.page_component.as_ref().unwrap().borrow();
            match component.deref() {
                PageComponent::Button(button) => { button.render(platform, scroll_y); }
                PageComponent::TextField(_) => { panic!("Invalid state"); }
            }
        },
        LayoutNodeContent::TextInputLayoutNode(_) => {
            let dom_node = layout_node.from_dom_node.as_ref().unwrap().borrow();
            let component = dom_node.page_component.as_ref().unwrap().borrow();
            match component.deref() {
                PageComponent::Button(_) => { panic!("Invalid state"); }
                PageComponent::TextField(text_field) => { text_field.render(ui_state, platform, scroll_y); }
            }
        },
        LayoutNodeContent::BoxLayoutNode(box_node) => {
            if box_node.background_color != Color::WHITE { //TODO: don't think this check is correct (also for text nodes,
                                                           //      because you can have this inside another colored node)
                let location = &box_node.location;
                platform.fill_rect(location.x, location.y - scroll_y, location.width, location.height, box_node.background_color, 255);
            }
        },
        LayoutNodeContent::TableLayoutNode(_) => {
            //eventually we might have something to render here, like a border or something (or is that also on cell level?)
            //for now we render nothing
        }
        LayoutNodeContent::TableCellLayoutNode(_) => {
            //TODO: implement (is there anything to render here aside from potential borders in the future?)
            todo!();
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
