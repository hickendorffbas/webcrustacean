mod color;
mod debug;
mod dom;
mod html_lexer;
#[cfg(test)] mod jsonify; //TODO: would also like to use it for debug, not sure how to configure that. feature flag on the crate maybe?
mod layout;
mod macros;
mod navigation;
mod network;
mod old_html_parser;
mod platform;
mod renderer;
mod resource_loader;
mod script;
mod selection;
mod style;
mod ui;
mod ui_components;

#[cfg(test)] mod reftests;
#[cfg(test)] mod test_util;

use std::{
    cell::RefCell,
    collections::HashMap,
    env,
    ops::DerefMut,
    thread,
    time::{Duration, Instant},
};

use arboard::Clipboard;
use sdl2::{
    event::Event as SdlEvent,
    keyboard::{Keycode, Mod as SdlKeyMod},
    mouse::MouseButton,
};
use threadpool::ThreadPool;

use crate::debug::debug_log_warn;
use crate::dom::Document;
use crate::layout::{
    collect_content_nodes_in_walk_order_for_normal_flow,
    compute_layout,
    FullLayout,
    rebuild_dirty_layout_childs,
};
use crate::navigation::{
    finish_navigate,
    NavigationAction,
    NavigationActionType,
    start_navigate
};
use crate::network::url::Url;
use crate::renderer::render;
use crate::resource_loader::{
    CookieStore,
    ResourceRequestResult,
    ResourceThreadPool,
};
use crate::selection::{
    Selection,
    SelectionRect,
    set_selection_regions
};
use crate::ui::{
    CONTENT_TOP_LEFT_X,
    CONTENT_TOP_LEFT_Y,
    FocusTarget,
    UIState,
};
use crate::ui_components::PageComponent;


//Config:
const TARGET_FPS: u32 = if cfg!(debug_assertions) { 20 } else { 60 };
const STARTING_SCREEN_WIDTH: f32 = 1400.0;
const STARTING_SCREEN_HEIGHT: f32 = 800.0;
const DEFAULT_LOCATION_TO_LOAD: &str = "about:home";
const SCROLL_SPEED: i32 = 50;
const NR_RESOURCE_LOADING_THREADS: usize = 4;
const USER_AGENT: &str = network::UA_WEBCRUSTACEAN_UBUNTU;


//Non-config constants:
const TARGET_MS_PER_FRAME: u128 = 1000 / TARGET_FPS as u128;


fn frame_time_check(start_instant: &Instant) {
    let millis_elapsed = start_instant.elapsed().as_millis();
    let sleep_time_millis = TARGET_MS_PER_FRAME as i64 - millis_elapsed as i64;
    if sleep_time_millis > 1 {
        //If we are more than a millisecond faster than what we need to reach the target FPS, we sleep
        thread::sleep(Duration::from_millis(sleep_time_millis as u64));
    } else {
        debug_log_warn(format!("we did not reach the target FPS, frametime: {}", millis_elapsed));
    }
}


fn handle_left_click(ui_state: &mut UIState, x: f32, y: f32, page_relative_mouse_y: f32, full_layout: &FullLayout, document: &Document) -> NavigationAction {
    let possible_url = ui::handle_possible_ui_click(ui_state, x, y);
    if possible_url.is_some() {
        return NavigationAction::new_get(possible_url.unwrap());
    }

    return full_layout.root_node.borrow().click(x, page_relative_mouse_y, document);
}


pub struct MouseState {
    x: i32,
    y: i32,
    click_start_x: i32,
    click_start_y: i32,
    left_down: bool,
}


fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let mut platform = platform::init_platform(sdl_context, STARTING_SCREEN_WIDTH, STARTING_SCREEN_HEIGHT, false).unwrap();

    let mut resource_thread_pool = ResourceThreadPool { pool: ThreadPool::new(NR_RESOURCE_LOADING_THREADS) };

    let mut mouse_state = MouseState { x: 0, y: 0, click_start_x: 0, click_start_y: 0, left_down: false };
    let mut ui_state = UIState::new(STARTING_SCREEN_WIDTH, STARTING_SCREEN_HEIGHT);

    let document = RefCell::from(Document::new_empty());
    let full_layout_tree = RefCell::from(FullLayout::new_empty());

    let mut cookie_store = CookieStore { cookies_by_domain: HashMap::new() };

    let args: Vec<String> = env::args().collect();
    let start_url = if args.len() < 2 {
        Url::from(&DEFAULT_LOCATION_TO_LOAD.to_owned())
    } else {
        Url::from(&args[1])
    };
    document.borrow_mut().base_url = start_url.clone();
    let mut ongoing_navigation = Some(NavigationAction::new_get(start_url));

    let mut main_page_job_tracker = start_navigate(&ongoing_navigation.as_ref().unwrap(), &platform, &mut ui_state, &cookie_store, &mut resource_thread_pool);

    let mut event_pump = platform.sdl_context.event_pump()?;
    'main_loop: loop {
        let start_loop_instant = Instant::now();

        if ongoing_navigation.is_some() {
            let try_recv_result = main_page_job_tracker.receiver.try_recv();
            if try_recv_result.is_ok() {
                let mut restarted_navigation = false;
                match try_recv_result.unwrap() {
                    ResourceRequestResult::NotFound => { //TODO: we need more types of result here, and the http status code
                        ui_state.currently_loading_page = false;

                        if ongoing_navigation.as_ref().unwrap().from_address_bar && ongoing_navigation.as_ref().unwrap().https_was_inserted {
                            //TODO: we now do this on ResourceRequestResult::NotFound , we need to do it on all 4xx or timeout responses

                            let mut url = match ongoing_navigation.unwrap().action_type {
                                NavigationActionType::Get(url) => { url.clone() },
                                _ => panic!("Invalid state"),
                            };

                            url.scheme = String::from("http");

                            let navigation_action = NavigationAction::new_get_from_addressbar(url, false);
                            main_page_job_tracker = start_navigate(&navigation_action, &platform, &mut ui_state, &cookie_store, &mut resource_thread_pool);
                            ongoing_navigation = Some(navigation_action);
                            restarted_navigation = true;
                        }
                    },
                    ResourceRequestResult::Success(received_result) => {
                        let domain = match &ongoing_navigation.as_ref().unwrap().action_type {
                            NavigationActionType::None => &String::new(),
                            NavigationActionType::Get(url) => &url.host,
                            NavigationActionType::Post(post_data) => &post_data.url.host,
                        };

                        //TODO: I think we want to extract cookies in a more centralized place
                        for new_cookie in received_result.new_cookies {
                            if !cookie_store.cookies_by_domain.contains_key(domain) {
                                cookie_store.cookies_by_domain.insert(domain.clone(), HashMap::new());
                            }
                            cookie_store.cookies_by_domain.get_mut(domain).unwrap().insert(new_cookie.0, new_cookie.1);
                        }

                        finish_navigate(&ongoing_navigation.as_ref().unwrap(), &mut ui_state, &received_result.body, &document, &cookie_store, &full_layout_tree, &mut platform, &mut resource_thread_pool)
                    },
                }

                if !restarted_navigation {
                    ongoing_navigation = None;
                }
            }
        }

        ui_state.current_scroll_y = ui_state.main_scrollbar_vert.update_content_size(full_layout_tree.borrow().page_height(), ui_state.current_scroll_y);
        ui_state.current_scroll_x = ui_state.main_scrollbar_hori.update_content_size(full_layout_tree.borrow().page_width(), ui_state.current_scroll_x);


        #[cfg(feature="timings")] let start_event_pump_instant = Instant::now();
        for event in event_pump.poll_iter() {
            match event {
                SdlEvent::Quit {..} | SdlEvent::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'main_loop;
                },
                SdlEvent::Window { win_event, .. } => {
                    match win_event {
                        sdl2::event::WindowEvent::SizeChanged(width, height) => {
                            ui_state.update_window_dimensions(width as f32, height as f32);
                            document.borrow_mut().document_node.borrow_mut().mark_all_as_dirty();
                        },
                        _ => {},
                    }
                }
                SdlEvent::MouseMotion { x: mouse_x, y: mouse_y, xrel, yrel, .. } => {
                    mouse_state.x = mouse_x;
                    mouse_state.y = mouse_y;

                    if mouse_state.left_down {
                        let selection = Selection { point1_x: mouse_state.click_start_x as f32 + ui_state.current_scroll_x as f32,
                                                    point1_y: mouse_state.click_start_y as f32 + ui_state.current_scroll_y as f32,
                                                    point2_x: mouse_x as f32 + ui_state.current_scroll_x as f32,
                                                    point2_y: mouse_y as f32 + ui_state.current_scroll_y as f32
                                        };

                        match ui_state.focus_target {
                            FocusTarget::None => {},
                            FocusTarget::MainContent => {
                                RefCell::borrow_mut(&full_layout_tree.borrow_mut().root_node).reset_selection();
                                let full_layout_tree = full_layout_tree.borrow();
                                set_selection_regions(&full_layout_tree, &selection);
                            },
                            FocusTarget::AddressBar => {
                                ui_state.addressbar.update_selection(&selection);
                            },
                            FocusTarget::ScrollBlockHori => {
                                ui_state.current_scroll_x = ui_state.main_scrollbar_hori.scroll(xrel as f32, ui_state.current_scroll_x);
                            },
                            FocusTarget::ScrollBlockVert => {
                                ui_state.current_scroll_y = ui_state.main_scrollbar_vert.scroll(yrel as f32, ui_state.current_scroll_y);
                            },
                            FocusTarget::Component(ref mut component) => {
                                match component.borrow_mut().deref_mut() {
                                    ui_components::PageComponent::Button(_) => {},
                                    ui_components::PageComponent::TextField(text_field) => {
                                        text_field.update_selection(&selection);
                                    },
                                }
                            }
                        }
                    }
                },
                SdlEvent::MouseButtonDown { mouse_btn: MouseButton::Left, x: mouse_x, y: mouse_y, .. } => {
                    mouse_state.x = mouse_x;
                    mouse_state.y = mouse_y;
                    mouse_state.click_start_x = mouse_x;
                    mouse_state.click_start_y = mouse_y;
                    mouse_state.left_down = true;

                    RefCell::borrow_mut(&full_layout_tree.borrow_mut().root_node).reset_selection();

                    ui::handle_possible_ui_mouse_down(&full_layout_tree.borrow().root_node, &document, &mut platform, &mut ui_state, mouse_x as f32, mouse_y as f32);
                },
                SdlEvent::MouseButtonUp { mouse_btn: MouseButton::Left, x: mouse_x, y: mouse_y, .. } => {
                    mouse_state.x = mouse_x;
                    mouse_state.y = mouse_y;
                    mouse_state.left_down = false;

                    match ui_state.focus_target {
                        FocusTarget::ScrollBlockHori | FocusTarget::ScrollBlockVert => { ui_state.focus_target = FocusTarget::None; }
                        _ => {}
                    };

                    let abs_movement = (mouse_state.x - mouse_state.click_start_x).abs() + (mouse_state.y - mouse_state.click_start_y).abs();
                    let was_dragging = abs_movement > 4;

                    if !was_dragging {
                        let page_relative_mouse_y = mouse_y as f32 + ui_state.current_scroll_y;
                        let navigation_action = handle_left_click(&mut ui_state, mouse_x as f32, mouse_y as f32, page_relative_mouse_y, &full_layout_tree.borrow(), &document.borrow());

                        //TODO: we should do this above in the next loop, just schedule the action for the next loop?
                        if navigation_action.action_type != NavigationActionType::None {
                            main_page_job_tracker = start_navigate(&navigation_action, &platform, &mut ui_state, &cookie_store, &mut resource_thread_pool);
                            ongoing_navigation = Some(navigation_action);
                        }
                    }
                },
                SdlEvent::MouseWheel { x, y, direction, .. } => {
                    match direction {
                        sdl2::mouse::MouseWheelDirection::Normal => {
                            //TODO: someday it might be nice to implement smooth scrolling (animate the movement over frames)
                            if x != 0 {
                                let new_page_scroll_x = ui_state.current_scroll_x - (x * -1 * SCROLL_SPEED) as f32;
                                ui_state.current_scroll_x = ui_state.main_scrollbar_hori.update_scroll(new_page_scroll_x);
                            }
                            if y != 0 {
                                let new_page_scroll_y = ui_state.current_scroll_y - (y * SCROLL_SPEED) as f32;
                                ui_state.current_scroll_y = ui_state.main_scrollbar_vert.update_scroll(new_page_scroll_y);
                            }
                        },
                        sdl2::mouse::MouseWheelDirection::Flipped => {},
                        sdl2::mouse::MouseWheelDirection::Unknown(_) => debug_log_warn("Unknown mousewheel direction!"),
                    }
                },
                SdlEvent::KeyDown { keycode, keymod, .. } => {
                    if keycode.is_some() {
                        let key_code = platform.convert_key_code(&keycode.unwrap());
                        ui::handle_keyboard_input(&mut platform, None, key_code, &mut ui_state);

                        if keymod.contains(SdlKeyMod::LCTRLMOD) {
                            if keycode.unwrap().name() == "C" {
                                let mut text_for_clipboard = String::new();

                                match ui_state.focus_target {
                                    FocusTarget::None => {},
                                    FocusTarget::MainContent => {
                                        full_layout_tree.borrow().root_node.borrow().get_selected_text(&mut text_for_clipboard);
                                    },
                                    FocusTarget::AddressBar => {
                                        if ui_state.addressbar.has_selection_active() {
                                            text_for_clipboard = ui_state.addressbar.get_selected_text();
                                        }
                                    },
                                    FocusTarget::ScrollBlockHori => {},
                                    FocusTarget::ScrollBlockVert => {},
                                    FocusTarget::Component(ref component) => {
                                        match *component.borrow() {
                                            PageComponent::Button(_) => {}, //There is nothing to copy from a button
                                            PageComponent::TextField(ref text_field) => {
                                                text_for_clipboard = text_field.get_selected_text();
                                            }
                                        }
                                    },
                                }

                                if !text_for_clipboard.is_empty() {
                                    let mut clipboard = Clipboard::new().unwrap();
                                    clipboard.set_text(text_for_clipboard).expect("Unhandled clipboard error");
                                }
                            }

                            if keycode.unwrap().name() == "V" {
                                let clipboard_text = Clipboard::new().unwrap().get_text().expect("Unhandled clipboard error");

                                match ui_state.focus_target {
                                    FocusTarget::AddressBar => {
                                        ui_state.addressbar.insert_text(&platform, &clipboard_text);
                                    },
                                    FocusTarget::Component(ref mut component) => {
                                        match *component.borrow_mut() {
                                            PageComponent::Button(_) => {}, //There is nothing to paste in a button
                                            PageComponent::TextField(ref mut text_field) => {
                                                text_field.insert_text(&platform, &clipboard_text);
                                            },
                                        }
                                    }
                                    _ => {},
                                }
                            }
                        }

                        match ui_state.focus_target {
                            FocusTarget::None => {},
                            FocusTarget::MainContent => {},
                            FocusTarget::ScrollBlockHori => {},
                            FocusTarget::ScrollBlockVert => {},
                            FocusTarget::AddressBar => {
                                //TODO: I still don't understand how this interacts with TextInput below. Why only handle enter here?
                                if keycode.unwrap().name() == "Return" {
                                    let mut url = ui_state.addressbar.text.clone();

                                    //we are not parsing the url yet, since parsing it will default to the file:// protocol
                                    let has_protocol = if let Some(pos) = url.find("://") {
                                        let possible_protocol = &url[..pos];
                                        !possible_protocol.contains('/')
                                    } else {
                                        false
                                    };
                                    if !has_protocol {
                                        url = format!("https://{}", url);
                                    }

                                    let navigation_action = NavigationAction::new_get_from_addressbar(Url::from(&url), !has_protocol);
                                    main_page_job_tracker = start_navigate(&navigation_action, &platform, &mut ui_state, &cookie_store, &mut resource_thread_pool);
                                    ongoing_navigation = Some(navigation_action);
                                }
                            },

                            FocusTarget::Component(ref component) => {
                                if keycode.unwrap().name() == "Return" {
                                    let dom_node = dom::find_dom_node_for_component(&component.borrow(), &document.borrow());
                                    let navigation_action = dom_node.borrow().submit_form(&document.borrow());
                                    main_page_job_tracker = start_navigate(&navigation_action, &platform, &mut ui_state, &cookie_store, &mut resource_thread_pool);
                                    ongoing_navigation = Some(navigation_action);
                                }
                            },
                        }

                    }
                },
                SdlEvent::TextInput { text, .. } => {
                    ui::handle_keyboard_input(&mut platform, Some(&text), None, &mut ui_state);
                },
                _ => {},
            }
        }
        #[cfg(feature="timings")] println!("event pump elapsed millis: {}", start_event_pump_instant.elapsed().as_millis());

        let document_has_dirty_nodes = document.borrow_mut().update_all_dom_nodes(&mut resource_thread_pool, &cookie_store);

        if document_has_dirty_nodes {
            rebuild_dirty_layout_childs(&full_layout_tree.borrow().root_node, &document.borrow(), &platform.font_context);

            let mut content_nodes_in_selection_order = Vec::new();
            collect_content_nodes_in_walk_order_for_normal_flow(&full_layout_tree.borrow().root_node, &mut content_nodes_in_selection_order);
            full_layout_tree.borrow_mut().content_nodes_in_selection_order = content_nodes_in_selection_order;

            compute_layout(&full_layout_tree.borrow().root_node, CONTENT_TOP_LEFT_X, CONTENT_TOP_LEFT_Y,
                           &platform.font_context, ui_state.current_scroll_y, false, false, ui_state.window_dimensions.content_viewport_width);
        }

        #[cfg(feature="timings")] let start_render_instant = Instant::now();
        render(&mut platform, &full_layout_tree.borrow(), &mut ui_state);
        #[cfg(feature="timings")] println!("render elapsed millis: {}", start_render_instant.elapsed().as_millis());

        frame_time_check(&start_loop_instant);
    }

    Ok(())
}
