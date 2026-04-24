use core::panic;
use std::collections::HashMap;
use std::env;
use std::fs::{self, metadata};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{Ordering, AtomicUsize};
use std::sync::mpsc::{
    channel,
    Receiver,
    Sender,
    TryRecvError,
};

use chrono::{DateTime, Utc};
use image::{
    ImageBuffer,
    ImageReader,
    RgbaImage
};
use threadpool::ThreadPool;

use crate::{NR_RESOURCE_LOADING_THREADS, resource_loader};
use crate::debug::debug_log_warn;
use crate::job_scheduler::{
    JobResult,
    JobScheduler,
    Task, TaskPayload,
};
use crate::network::url::Url;
use crate::network::{
    http_get_image,
    http_get_text,
    http_post_for_text,
};


//TODO: this should be removed (it will overlap with the job and task ids from the job scheduler) when the jobs are gone from here
static NEXT_JOB_ID: AtomicUsize = AtomicUsize::new(1);
pub fn get_next_job_id() -> usize { NEXT_JOB_ID.fetch_add(1, Ordering::Relaxed) }


pub struct ResourceLoader {
    scheduler: JobScheduler,
    active_jobs: Vec<(Receiver<JobResult>, Task)>, //TODO: should these not live on the job scheduler?
}
impl ResourceLoader {
    pub fn new() -> ResourceLoader {
        let scheduler = JobScheduler::new(NR_RESOURCE_LOADING_THREADS);
        return ResourceLoader { scheduler, active_jobs: Vec::new() }
    }

    pub fn request_text_http_get_text(&mut self, url: &Url, cookies: HashMap<String, String>, task: Task) {
        let job = self.scheduler.submit_http_get_text_job(url, cookies);
        self.active_jobs.push((job, task));
    }

    pub fn request_text_http_get_image(&mut self, url: &Url, cookies: HashMap<String, String>, task: Task) {
        let job = self.scheduler.submit_http_get_image_job(url, cookies);
        self.active_jobs.push((job, task));
    }

    pub fn any_jobs_in_progress(&self) -> bool {
        return self.active_jobs.len() > 0;
    }

    pub fn handle_possible_finished_job(&mut self, task_store: &mut Vec<Task>, cookie_store: &mut CookieStore) {
        //This method handles max 1 finished job, because we call it each frame this is fine

        let mut finshed_job_result = None;
        let finished_job_position = self.active_jobs.iter_mut().position(|job| {
            match job.0.try_recv() {
                Ok(data) => {
                    finshed_job_result = Some(data);
                    true
                },
                Err(TryRecvError::Empty) => false,
                _ => {
                   todo!(); //TODO: probably always an error?
                }
            }
        });

        if finished_job_position.is_none() {
            return;
        }

        let (_, mut future_task) = self.active_jobs.remove(finished_job_position.unwrap());

        match finshed_job_result.unwrap() {
            JobResult::ResourceRequestResultString { value } => {
                match value {
                    ResourceRequestResult::Success { body, new_cookies, domain } => {

                        //TODO: I think we want to extract cookies in a more centralized place
                        for new_cookie in new_cookies {
                            if !cookie_store.cookies_by_domain.contains_key(&domain) {
                                cookie_store.cookies_by_domain.insert(domain.clone(), HashMap::new());
                            }
                            cookie_store.cookies_by_domain.get_mut(&domain).unwrap().insert(new_cookie.0, new_cookie.1);
                        }

                        match &mut future_task.payload {
                            TaskPayload::ParseJs { script_data } => {
                                *script_data = body;
                            },
                            TaskPayload::StartParseHtml { html } => {
                                *html = body;
                            },
                            _ => {
                                panic!("Unsupported task payload for this jobresult");
                            }
                        }

                        future_task.ready = true;
                        task_store.push(future_task);
                    },
                    ResourceRequestResult::NotFound => {

                        //if ongoing_navigation.as_ref().unwrap().from_address_bar && ongoing_navigation.as_ref().unwrap().https_was_inserted {
                            //let mut url = match ongoing_navigation.unwrap().action_type {
                            //    NavigationActionType::Get(url) => { url.clone() },
                            //    _ => panic!("Invalid state"),
                            //};

                            //url.scheme = String::from("http");

                            //let navigation_action = NavigationAction::new_get_from_addressbar(url, false);
                            //main_page_job_tracker = start_navigate(&navigation_action, &platform, &mut ui_state, &cookie_store, &mut resource_thread_pool);
                            //ongoing_navigation = Some(navigation_action);
                            //restarted_navigation = true;

                            todo!(); //TODO: submit a new job with the same task id behind it (don't push the task yet)

                        //} else {
                        //    todo!(); //TODO: implement
                        //}

                    },
                }
            },
            JobResult::ResourceRequestResultImage { value } => {
                let image_result = match value {
                    ResourceRequestResult::Success { body, new_cookies, domain } => {

                        //TODO: I think we want to extract cookies in a more centralized place
                        for new_cookie in new_cookies {
                            if !cookie_store.cookies_by_domain.contains_key(&domain) {
                                cookie_store.cookies_by_domain.insert(domain.clone(), HashMap::new());
                            }
                            cookie_store.cookies_by_domain.get_mut(&domain).unwrap().insert(new_cookie.0, new_cookie.1);
                        }

                        body
                    },
                    ResourceRequestResult::NotFound => {
                        resource_loader::fallback_image()
                    },
                };

                match &mut future_task.payload {
                    TaskPayload::SetImageOnDomNode { dom_node_id: _, image  } => {
                        *image = Some(Rc::from(image_result));
                    },
                    _ => {
                        panic!("Unsupported task payload for this jobresult");
                    }
                }

                future_task.ready = true;
                task_store.push(future_task);
            },
        }
    }
}



#[derive(PartialEq)]
pub enum RequestType {
    Get,
    Post,
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub enum ResourceRequestResult<T> {
    NotFound,
    Success {
        body: T,
        new_cookies: HashMap<String, CookieEntry>,
        domain: String, //TODO: should this be domain or host?
    },
}

struct ResourceRequestJob<T> {
    #[allow(dead_code)] job_id: usize, //TODO: check if we want to use this (probably for logging / debugging?)
    url: Url,
    sender: Sender<T>,
    request_type: RequestType,
    body: Option<String>,
    cookies: HashMap<String, String>,
}
#[derive(Debug)]
pub struct ResourceRequestJobTracker<T> {
    #[allow(dead_code)] pub job_id: usize, //TODO: check if we want to use this (probably for logging / debugging?)
    pub receiver: Receiver<T>,
}


pub struct CookieStore {
    pub cookies_by_domain: HashMap<String, CookieStoreForDomain>,
}
impl CookieStore {
    pub fn get_for_domain(&self, domain: &String) -> HashMap<String, String> {
        let mut cookies = HashMap::new();
        let domain_entries = self.cookies_by_domain.get(domain);
        if domain_entries.is_some() {
            for (key, value) in domain_entries.unwrap() {
                cookies.insert(key.clone(), value.value.clone());
            }
        }
        return cookies;
    }
}

type CookieStoreForDomain = HashMap<String, CookieEntry>;


#[cfg_attr(debug_assertions, derive(Debug))]
pub struct CookieEntry {
    pub value: String,
    #[allow(dead_code)] pub expiry_time: DateTime<Utc>,  //TODO: use this
}


pub struct ResourceThreadPool {
    pub pool: ThreadPool,
}
impl ResourceThreadPool {
    fn fire_and_forget_load_image(&mut self, job: ResourceRequestJob<ResourceRequestResult<RgbaImage>>) {
        self.pool.execute(move || {
            let result = load_image(&job.url, &job.cookies);
            job.sender.send(result).expect("Could not send over channel");
        });
    }
    fn fire_and_forget_load_text(&mut self, job: ResourceRequestJob<ResourceRequestResult<String>>) {
        self.pool.execute(move || {
            let result = load_text(&job.url, job.request_type, job.body, &job.cookies);
            job.sender.send(result).expect("Could not send over channel");
        });
    }
}


pub fn submit_post(url: &Url, cookie_store: &CookieStore, fields: &HashMap<String, String>,
                   resource_thread_pool: &mut ResourceThreadPool) -> ResourceRequestJobTracker<ResourceRequestResult<String>> {
    let (sender, receiver) = channel::<ResourceRequestResult<String>>();
    let job_id = get_next_job_id();

    //TODO: we need to esape values here I think, what if "&" is in a post value?
    let body = fields.iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<String>>().join("&");

    let cookies = cookie_store.get_for_domain(&url.host);

    let job = ResourceRequestJob { job_id, url: url.clone(), sender, request_type: RequestType::Post, body: Some(body), cookies: cookies };
    let job_tracker = ResourceRequestJobTracker { job_id, receiver };

    resource_thread_pool.fire_and_forget_load_text(job);

    return job_tracker;
}


pub fn load_text(url: &Url, request_type: RequestType, body: Option<String>, cookies: &HashMap<String, String>) -> ResourceRequestResult<String> {
    //TODO: this should not be text specific, we need to refactor this a bit

    if url.scheme == "about" {
        if request_type == RequestType::Get {
            return ResourceRequestResult::Success { body: build_about_page(&url), new_cookies: HashMap::new(), domain: url.host.clone() };
        } else {
            todo!(); //TODO: report some kind of non-crashing error
        }
    }

    if url.scheme == "file" {
        if request_type == RequestType::Get {
            let mut local_path = String::from("//");
            local_path.push_str(&url.path.join("/"));
            let read_result = fs::read_to_string(local_path);
            if read_result.is_err() {
                return ResourceRequestResult::NotFound;
            }

            return ResourceRequestResult::Success { body: read_result.unwrap(), new_cookies: HashMap::new(), domain: url.host.clone() };
        } else {
            todo!(); //TODO: report some kind of non-crashing error
        }
    }

    let file_content_result = match request_type {
        RequestType::Get => http_get_text(url, cookies),
        RequestType::Post => http_post_for_text(url, body.unwrap_or(String::new()), cookies),
    };

    return file_content_result;
}


fn build_about_page(url: &Url) -> String {

    if url.path.len() == 1 && url.path.iter().next().unwrap().as_str() == "home" {
        let our_path = env::current_dir().unwrap();
        let mut local_file_urls = Vec::new();

        get_all_html_in_folder(our_path, &mut local_file_urls);

        let mut html = String::from("<html><h1>Webcrustacean Home</h1><br />");
        for local_file_url in local_file_urls {
            let file_url = &local_file_url.into_os_string().into_string().unwrap();
            html += format!("<a href=\"file://{file_url}\">{file_url}</a><br />").as_str();
        }

        return html;
    }

    //TODO: this error should not just be debug-logged, it should return this, and then render the 404 page, if this was the main page load...
    debug_log_warn(format!("Could not load text: {}", url.to_string()));
    return String::new();
}


fn get_all_html_in_folder(folder_path: PathBuf, local_file_urls: &mut Vec<PathBuf>) {
    let files_in_current_folder = fs::read_dir(folder_path).unwrap();
    for file in files_in_current_folder {
        let path = file.as_ref().unwrap().path();
        if metadata(&path).unwrap().is_dir() {
            if !path.ends_with("reftest_data") { //TODO: this check is an ugly hack to filter out test data. We need to make a nicer about::home
                                                 //      maybe with a configurable folder to list, or just a link to a folder listing page (file:://)
                get_all_html_in_folder(path, local_file_urls);
            }
        } else {
            if path.extension().is_some() && path.extension().unwrap().to_str().unwrap() == "html" {
                local_file_urls.push(path);
            }
        }
    }
    local_file_urls.sort();
}


pub fn load_image(url: &Url, cookies: &HashMap<String, String>) -> ResourceRequestResult<RgbaImage> {
    if url.scheme == "file" {
        let mut local_path = String::from("//");
        local_path.push_str(&url.path.join("/"));
        let read_result = ImageReader::open(local_path);
        if read_result.is_err() {
            return ResourceRequestResult::NotFound;
        }

        let file_data = read_result.unwrap();
        let format_guess_result = file_data.with_guessed_format();

        let dyn_image = if format_guess_result.is_ok() {
            format_guess_result.ok().unwrap().decode().expect("decoding the image failed") //TODO: we need to handle this in a better way
        } else {
            panic!("decoding the image failed"); //TODO: we need to handle this in a better way
        };

        return ResourceRequestResult::Success { body: dyn_image.to_rgba8(), new_cookies: HashMap::new(), domain: url.host.clone() };
    }

    let extension = url.file_extension();
    if extension.is_some() && extension.unwrap() == "svg".to_owned() {
        //svg is currently not implemented
        debug_log_warn(format!("Svg's are not supported currently: {}", url.to_string()));
        return ResourceRequestResult::Success {  body: fallback_image(), new_cookies: HashMap::new(), domain: url.host.clone() };
    }
    if url.scheme == "data".to_owned() {
        //data scheme is currently not implemented
        debug_log_warn(format!("the data: scheme is not supported currently: {}", url.to_string()));
        return ResourceRequestResult::Success {  body: fallback_image(), new_cookies: HashMap::new(), domain: url.host.clone() };
    }

    #[cfg(debug_assertions)] println!("loading {}", url.to_string()); //TODO: debug mode should have a more general way of logging all HTTP request/responses

    return http_get_image(url, cookies);
}


pub fn fallback_image() -> RgbaImage {
    //TODO: this should become one of those "broken image"-images
    return ImageBuffer::new(1, 1);
}
