use crate::dom::PostData;
use crate::html_parser::{HtmlParser, ParserState};
use crate::job_scheduler::{Task, TaskPayload};
use crate::network::url::Url;
use crate::platform::Platform;
use crate::resource_loader::{CookieStore, ResourceLoader};
use crate::ui::{self, UIState};


#[cfg_attr(debug_assertions, derive(Debug))]
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


#[cfg_attr(debug_assertions, derive(Debug))]
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
                      resource_loader: &mut ResourceLoader, html_parser: &mut HtmlParser) {

    match &navigation_action.action_type {
        NavigationActionType::None => {
            panic!("Illegal state"); // we should not get in this method if we have nothing to navigate to...
        },
        NavigationActionType::Get(url) => {
            ui_state.addressbar.set_text(platform, url.to_string());

            if !ui_state.history.currently_navigating_from_history {
                ui::register_in_history(ui_state, &url);
            }

            let future_task = Task::new_task_not_yet_ready(TaskPayload::StartParseHtml { html: String::new() });
            html_parser.reset();
            html_parser.state = ParserState::WaitingForContent { task_id: future_task.id };
            resource_loader.request_text_http_get_text(&url, cookie_store.get_for_domain(&url.host), future_task);

        },
        NavigationActionType::Post(post_data) => {
            ui_state.addressbar.set_text(platform, post_data.url.to_string());

            if !ui_state.history.currently_navigating_from_history {
                //TODO: we should actually record the postdata in the history. Or actually the whole page, and not request again? How do other browsers do this?
                ui::register_in_history(ui_state, &post_data.url);
            }

            todo!(); //TODO: How do do POST with the new setup?
            //resource_loader::submit_post(&post_data.url, cookie_store, &post_data.fields, resource_thread_pool) //TODO: should this be a different thread pool, or rename it?
        }
    };

    ui_state.current_scroll_x = 0.0;
    ui_state.current_scroll_y = 0.0;
    ui_state.currently_loading_page = true;
    ui_state.history.currently_navigating_from_history = false;
    ui::update_history_buttons(ui_state);
}


pub fn progress_html_parser(parser: &mut HtmlParser, resource_loader: &mut ResourceLoader, cookie_store: &CookieStore, task_store: &mut Vec<Task>) {
    while !parser.is_done() {
        parser.step();

        let state = std::mem::replace(&mut parser.state, ParserState::ContinueParsing);
        match state {
            ParserState::WaitingToStart => {
                parser.state = state;
                return;
            },
            ParserState::WaitingForContent { task_id } => {
                for task in task_store.iter() {
                    if task.id == task_id && task.finished {
                        parser.state = ParserState::ContinueParsing;
                        return;
                    }
                }
                parser.state = state;
                return;
            },
            ParserState::ContinueParsing => {}
            ParserState::WaitingForScriptRun { task_id } => {
                for task in task_store.iter() {
                    if task.id == task_id && task.finished {
                        parser.state = ParserState::ContinueParsing;
                        return;
                    }
                }
                parser.state = state;
                return;
            },
            ParserState::ShouldDownloadScript(url) => {
                let future_task = Task::new_task_not_yet_ready(TaskPayload::ParseJs { script_data: String::new() });
                parser.state = ParserState::WaitingForScriptRun { task_id: future_task.id };
                resource_loader.request_text_http_get_text(&url, cookie_store.get_for_domain(&url.host), future_task);
            },
            ParserState::ShouldExecuteScript { script } => {
                let task = Task::new(TaskPayload::ParseJs { script_data: script });
                let task_id = task.id;
                task_store.push(task);
                parser.state = ParserState::WaitingForScriptRun { task_id };
                return;
            },
            ParserState::Done => {
                parser.state = state;
                return;
            },
        }
    };
}
