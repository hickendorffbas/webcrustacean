use std::cell::RefCell;

use crate::dom::{Document, PostData};
use crate::html_parser::HtmlParser;
use crate::layout;
use crate::layout::{compute_layout, FullLayout};
use crate::network::url::Url;
use crate::platform::Platform;
use crate::resource_loader::{
    self,
    CookieStore,
    ResourceRequestJobTracker,
    ResourceRequestResult,
    ResourceThreadPool
};
use crate::style::compute_styles;
use crate::ui::{
    self,
    UIState,
    CONTENT_TOP_LEFT_X,
    CONTENT_TOP_LEFT_Y
};


pub struct NavigationAction {
    pub action_type: NavigationActionType,
    pub from_address_bar: bool,
    pub https_was_inserted: bool,
}
impl NavigationAction {
    pub fn new_get(url: Url) -> NavigationAction {
        return NavigationAction { action_type: NavigationActionType::Get(url), from_address_bar: false, https_was_inserted: false };
    }
    pub fn new_get_from_addressbar(url: Url, https_was_inserted: bool) -> NavigationAction {
        return NavigationAction { action_type: NavigationActionType::Get(url), from_address_bar: true, https_was_inserted };
    }
    pub fn new_post(post_data: PostData) -> NavigationAction {
        return NavigationAction { action_type: NavigationActionType::Post(post_data), from_address_bar: false, https_was_inserted: false };
    }
    pub fn new_none() -> NavigationAction {
        return NavigationAction { action_type: NavigationActionType::None, from_address_bar: false, https_was_inserted: false };
    }
}


#[derive(PartialEq)]
pub enum NavigationActionType {
    None,
    Get(Url),
    Post(PostData),
}


pub struct History {
    pub list: Vec<Url>,  //TODO: this should become a list of navigation actions, I think
    pub position: usize,
    pub currently_navigating_from_history: bool,
}


pub fn start_navigate(navigation_action: &NavigationAction, platform: &Platform, ui_state: &mut UIState, cookie_store: &CookieStore,
                      resource_thread_pool: &mut ResourceThreadPool) -> ResourceRequestJobTracker<ResourceRequestResult<String>> {

    let tracker = match &navigation_action.action_type {
        NavigationActionType::None => {
            panic!("Illegal state"); // we should not get in this method if we have nothing to navigate to...
        },
        NavigationActionType::Get(url) => {
            ui_state.addressbar.set_text(platform, url.to_string());

            if !ui_state.history.currently_navigating_from_history {
                ui::register_in_history(ui_state, &url);
            }

            resource_loader::schedule_load_text(&url, cookie_store, resource_thread_pool) //TODO: should this be a different thread pool, or rename it?
        },
        NavigationActionType::Post(post_data) => {
            ui_state.addressbar.set_text(platform, post_data.url.to_string());

            if !ui_state.history.currently_navigating_from_history {
                //TODO: we should actually record the postdata in the history. Or actually the whole page, and not request again? How do other browsers do this?
                ui::register_in_history(ui_state, &post_data.url);
            }

            resource_loader::submit_post(&post_data.url, cookie_store, &post_data.fields, resource_thread_pool) //TODO: should this be a different thread pool, or rename it?
        }
    };


    ui_state.currently_loading_page = true;
    ui_state.history.currently_navigating_from_history = false;
    ui::update_history_buttons(ui_state);

    return tracker;
}


pub fn finish_navigate(navigation_action: &NavigationAction, ui_state: &mut UIState, page_content: String, document: &RefCell<Document>,
                       cookie_store: &CookieStore, full_layout: &RefCell<FullLayout>, platform: &mut Platform, resource_thread_pool: &mut ResourceThreadPool) {

    let url = match &navigation_action.action_type {
        NavigationActionType::None => {
            panic!("Illegal state"); // we should not get in this method if we have nothing to navigate to...
        },
        NavigationActionType::Get(url) => { url },
        NavigationActionType::Post(post_data) => { &post_data.url },
    };

    let mut parser = HtmlParser::new(page_content, url.clone());
    parser.parse();
    document.replace(parser.document);

    compute_styles(&document.borrow().document_node, &document.borrow().all_nodes, &document.borrow().style_context);

    document.borrow_mut().document_node.borrow_mut().post_construct(platform);
    document.borrow_mut().update_all_dom_nodes(resource_thread_pool, cookie_store);

    #[cfg(feature="timings")] let start_layout_instant = Instant::now();
    full_layout.replace(layout::build_full_layout(&document.borrow(), &platform.font_context));

    ui_state.current_scroll_y = 0.0;
    ui_state.currently_loading_page = false;

    compute_layout(&full_layout.borrow().root_node, CONTENT_TOP_LEFT_X, CONTENT_TOP_LEFT_Y,
                   &platform.font_context, ui_state.current_scroll_y, false, true, ui_state.window_dimensions.content_viewport_width);

    #[cfg(feature="timings")] println!("layout elapsed millis: {}", start_layout_instant.elapsed().as_millis());
}
