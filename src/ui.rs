use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::dom::Document;
use crate::layout::LayoutNode;
use crate::color::Color;
use crate::navigation::History;
use crate::network::url::Url;
use crate::platform::{
    KeyCode,
    Platform,
    Position
};
use crate::ui_components::{
    NavigationButton,
    PageComponent,
    Scrollbar,
    TextField
};


pub const CONTENT_TOP_LEFT_X: f32 = 0.0;
pub const CONTENT_TOP_LEFT_Y: f32 = HEADER_HEIGHT;

pub const HEADER_HEIGHT: f32 = 50.0;

pub const UI_BASIC_COLOR: Color = Color::new(212, 208, 200);
pub const UI_BASIC_DARKER_COLOR: Color = Color::new(116, 107, 90);

pub const MAIN_SCROLLBAR_WIDTH: f32 = 20.0;


#[cfg_attr(debug_assertions, derive(Debug))]
pub enum FocusTarget {
    None,
    MainContent,
    AddressBar,
    ScrollBlock, //TODO: eventually we could have more scrollbars, so maybe make scrollbars page components
    Component(Rc<RefCell<PageComponent>>),
}

pub struct WindowDimensions {
    pub screen_width: f32,
    pub screen_height: f32,
    pub content_viewport_width: f32,
    pub content_viewport_height: f32,
}

pub struct UIState {
    pub addressbar: TextField,
    pub current_scroll_y: f32,
    pub back_button: NavigationButton,
    pub forward_button: NavigationButton,
    pub history: History,
    pub currently_loading_page: bool,
    pub animation_tick: u32,
    pub focus_target: FocusTarget,
    pub main_scrollbar: Scrollbar, //TODO: eventually this should become a dynamic page component in the list, because there might be more than 1 scrollbar
    pub window_dimensions: WindowDimensions,
}
impl UIState {
    pub fn new(screen_width: f32, screen_height: f32) -> UIState {

        let scrollbar_x_pos = screen_width - MAIN_SCROLLBAR_WIDTH;
        let scrollbar_height = screen_height - HEADER_HEIGHT;
        let main_scrollbar = Scrollbar {
            x: scrollbar_x_pos,
            y: HEADER_HEIGHT,
            width: screen_width - scrollbar_x_pos,
            height: scrollbar_height,
            content_size: 0.0,
            content_viewport_height: 0.0,
            block_height: scrollbar_height,
            block_y: HEADER_HEIGHT,
            enabled: false,
        };

        let mut ui_state = UIState {
            addressbar: TextField::new(100.0, 10.0, screen_width - 200.0, 35.0, true),
            current_scroll_y: 0.0,
            back_button: NavigationButton { x: 15.0, y: 15.0, forward: false, enabled: false },
            forward_button: NavigationButton { x: 55.0, y: 15.0, forward: true, enabled: false },
            history: History { list: Vec::new(), position: 0, currently_navigating_from_history: false },
            currently_loading_page: false,
            animation_tick: 0,
            focus_target: FocusTarget::None,
            main_scrollbar: main_scrollbar,
            window_dimensions: WindowDimensions { screen_height, screen_width, content_viewport_height: 0.0, content_viewport_width: 0.0 },
        };

        ui_state.update_window_dimensions(screen_width, screen_height);

        return ui_state;
    }

    pub fn update_window_dimensions(&mut self, screen_width: f32, screen_height: f32) {
        self.window_dimensions.screen_width = screen_width;
        self.window_dimensions.screen_height = screen_height;

        self.window_dimensions.content_viewport_width = screen_width - MAIN_SCROLLBAR_WIDTH;
        self.window_dimensions.content_viewport_height = screen_height - HEADER_HEIGHT;

        self.main_scrollbar.update_content_viewport_size(self.window_dimensions.content_viewport_width,
                                                         self.window_dimensions.content_viewport_height, self.current_scroll_y);
        self.addressbar.width = screen_width - 200.0;
    }
}

pub fn render_ui(platform: &mut Platform, ui_state: &mut UIState) {
    update_animation_state(ui_state);
    render_header(platform, ui_state);

    ui_state.main_scrollbar.render(platform);
}


pub fn register_in_history(ui_state: &mut UIState, url: &Url) {
    if ui_state.history.list.len() > (ui_state.history.position + 1) {
        let last_idx_to_keep = ui_state.history.position;
        for idx in ((last_idx_to_keep + 1)..ui_state.history.list.len()).rev() {
            ui_state.history.list.remove(idx);
        }
    }
    ui_state.history.list.push(url.clone());
    ui_state.history.position = ui_state.history.list.len() - 1;
    if ui_state.history.position > 0 {
        ui_state.back_button.enabled = true;
    }
}


pub fn update_history_buttons(ui_state: &mut UIState) {
    ui_state.forward_button.enabled = ui_state.history.list.len() > ui_state.history.position + 1;
    ui_state.back_button.enabled = ui_state.history.position > 0;
}


pub fn handle_keyboard_input(platform: &mut Platform, input: Option<&String>, key_code: Option<KeyCode>, ui_state: &mut UIState) {

    match &ui_state.focus_target {
        FocusTarget::None => {},
        FocusTarget::MainContent => {},
        FocusTarget::AddressBar => {
            ui_state.addressbar.handle_keyboard_input(platform, input, key_code);
        },
        FocusTarget::ScrollBlock => {},
        FocusTarget::Component(component) => {
            match component.borrow_mut().deref_mut() {
                PageComponent::Button(_) => {
                    //TODO: handle enter here
                }
                PageComponent::TextField(text_field) => {
                    text_field.handle_keyboard_input(platform, input, key_code);
                },
            }
        },
    }
}


pub fn handle_possible_ui_click(ui_state: &mut UIState, x: f32, y: f32) -> Option<Url> {
    let possible_url = ui_state.back_button.click(x, y, &mut ui_state.history);
    if possible_url.is_some() {
        return possible_url;
    }
    let possible_url = ui_state.forward_button.click(x, y, &mut ui_state.history);
    if possible_url.is_some() {
        return possible_url;
    }

    return None;
}

pub fn handle_possible_ui_mouse_down(root_layout_node: &Rc<RefCell<LayoutNode>>, document: &RefCell<Document>, platform: &mut Platform, ui_state: &mut UIState, x: f32, y: f32) -> Option<Url> {
    let mut any_text_field_has_focus = false;

    if ui_state.addressbar.is_inside(x, y) {
        ui_state.focus_target = FocusTarget::AddressBar;
        ui_state.addressbar.mouse_down(x, y);
        any_text_field_has_focus = true;
    } else if ui_state.main_scrollbar.is_on_scrollblock(x, y) {
        ui_state.focus_target = FocusTarget::ScrollBlock;
    } else {

        let mut component_found = false;

        let possible_dom_node = root_layout_node.borrow().find_dom_node_at_position(x, y + ui_state.current_scroll_y);
        if possible_dom_node.is_some() {
            let dom_node = possible_dom_node.unwrap();
            let borr_dom_node = dom_node.borrow();
            if borr_dom_node.page_component.is_some() {
                let rc_component_clone = borr_dom_node.page_component.as_ref().unwrap().clone();

                match borr_dom_node.page_component.as_ref().unwrap().borrow_mut().deref_mut() {
                    PageComponent::Button(button) => {
                        ui_state.focus_target = FocusTarget::Component(rc_component_clone);
                        button.has_focus = true;
                        component_found = true;
                    },
                    PageComponent::TextField(text_field) => {
                        ui_state.focus_target = FocusTarget::Component(rc_component_clone);
                        component_found = true;
                        any_text_field_has_focus = true;
                        text_field.mouse_down(x, y);
                    },
                }
            }
        }

        if !component_found {
            //TODO: this is not always true (for example when clicking in the top bar but not in the addressbar), but for now we always set focus on the content
            //      it would be more correct to check for the content window size, and set it to None otherwise
            ui_state.focus_target = FocusTarget::MainContent;
        }
    }

    if any_text_field_has_focus {
        platform.enable_text_input();
    } else {
        platform.disable_text_input();
    }

    clear_other_focus(ui_state, document);

    return None;
}


fn clear_other_focus(ui_state: &mut UIState, document: &RefCell<Document>) {

    let mut component_id_with_focus = None;
    let mut addressbar_has_focus = false;

    match &ui_state.focus_target {
        FocusTarget::None => {},
        FocusTarget::MainContent => {},
        FocusTarget::ScrollBlock => {},
        FocusTarget::AddressBar => { addressbar_has_focus = true; },
        FocusTarget::Component(component) => {
            component_id_with_focus = Some(component.borrow().get_id())
        }
    }

    if !addressbar_has_focus {
        ui_state.addressbar.has_focus = false;
        ui_state.addressbar.clear_selection();
    }

    for node in document.borrow().all_nodes.values() {
        let node_borr = node.borrow();
        if node_borr.page_component.is_some() {
            if component_id_with_focus.is_none() || node_borr.page_component.as_ref().unwrap().borrow().get_id() != component_id_with_focus.unwrap() {
                match node_borr.page_component.as_ref().unwrap().borrow_mut().deref_mut() {
                    PageComponent::Button(button) => {
                        button.has_focus = false;
                    }
                    PageComponent::TextField(text_field) => {
                        text_field.has_focus = false;
                        text_field.clear_selection();
                    },
                }
            }
        }
    }

}


fn render_header(platform: &mut Platform, ui_state: &UIState) {
    platform.fill_rect(0.0, 0.0, ui_state.window_dimensions.screen_width, HEADER_HEIGHT, Color::WHITE, 255);

    platform.draw_line(Position { x: 0.0, y: HEADER_HEIGHT - 1.0 },
                       Position { x: ui_state.window_dimensions.screen_width, y: HEADER_HEIGHT - 1.0 },
                       Color::BLACK);

    if ui_state.currently_loading_page {
        render_spinner(platform, ui_state);
    }

    ui_state.back_button.render(platform);
    ui_state.forward_button.render(platform);
    ui_state.addressbar.render(&ui_state, platform, 0.0);
}


fn render_spinner(platform: &mut Platform, ui_state: &UIState) {
    let block_size = 5.0;
    let block_spacing = 15.0;
    let spinner_x_pos = ui_state.addressbar.x + ui_state.addressbar.width + 15.0;
    let spinner_y_pos = (ui_state.addressbar.y + ui_state.addressbar.height / 2.0) - (block_size / 2.0);

    let number_of_blocks = (ui_state.animation_tick % 1000) / 250;

    if number_of_blocks > 0 {
        platform.fill_rect(spinner_x_pos, spinner_y_pos, block_size, block_size, Color::BLACK, 255);
    }
    if number_of_blocks > 1 {
        platform.fill_rect(spinner_x_pos + block_spacing, spinner_y_pos, block_size, block_size, Color::BLACK, 255);
    }
    if number_of_blocks > 2 {
        platform.fill_rect(spinner_x_pos + (block_spacing * 2.0), spinner_y_pos, block_size, block_size, Color::BLACK, 255);
    }
}


fn update_animation_state(ui_state: &mut UIState) {
    let current_millis = SystemTime::now().duration_since(UNIX_EPOCH)
                            .expect("Time went backwards, please check if you entered a wormhole").as_millis();
    ui_state.animation_tick = (current_millis % 10_000) as u32;
}
