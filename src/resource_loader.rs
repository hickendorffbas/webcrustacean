use std::collections::HashMap;
use std::env;
use std::fs::{self, metadata};
use std::path::PathBuf;
use std::sync::atomic::{Ordering, AtomicUsize};
use std::sync::mpsc::{
    channel,
    Receiver,
    Sender
};

use chrono::{DateTime, Utc};
use image::{
    ImageBuffer,
    ImageReader,
    RgbaImage
};
use threadpool::ThreadPool;

use crate::debug::debug_log_warn;
use crate::network::url::Url;
use crate::network::{
    http_get_image,
    http_get_text,
    http_post_for_text,
};


static NEXT_JOB_ID: AtomicUsize = AtomicUsize::new(1);
pub fn get_next_job_id() -> usize { NEXT_JOB_ID.fetch_add(1, Ordering::Relaxed) }

#[derive(PartialEq)]
enum RequestType {
    Get,
    Post,
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub enum ResourceRequestResult<T> {
    NotFound,
    Success(ResourceRequestResultSuccess<T>),
}

#[cfg_attr(debug_assertions, derive(Debug))]
pub struct ResourceRequestResultSuccess<T> {
    pub body: T,
    pub new_cookies: HashMap<String, CookieEntry>,
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
    fn get_for_domain(&self, domain: &String) -> HashMap<String, String> {
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


pub fn schedule_load_text(url: &Url, cookie_store: &CookieStore, resource_thread_pool: &mut ResourceThreadPool) -> ResourceRequestJobTracker<ResourceRequestResult<String>> {
    let (sender, receiver) = channel::<ResourceRequestResult<String>>();
    let job_id = get_next_job_id();

    let cookies = cookie_store.get_for_domain(&url.host);

    let job = ResourceRequestJob { job_id, url: url.clone(), sender, request_type: RequestType::Get, body: None, cookies: cookies };
    let job_tracker = ResourceRequestJobTracker { job_id, receiver };

    resource_thread_pool.fire_and_forget_load_text(job);

    return job_tracker;
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


fn load_text(url: &Url, request_type: RequestType, body: Option<String>, cookies: &HashMap<String, String>) -> ResourceRequestResult<String> {
    //TODO: this should not be text specific, we need to refactor this a bit

    if url.scheme == "about" {
        if request_type == RequestType::Get {
            return ResourceRequestResult::Success(ResourceRequestResultSuccess { body: build_about_page(&url), new_cookies: HashMap::new() });
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

            return ResourceRequestResult::Success(ResourceRequestResultSuccess { body: read_result.unwrap(), new_cookies: HashMap::new() });
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

        let mut html = String::from("<html><h1>Webcrustacean Home<h1><br />");
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
    //TODO: test the folder walking code on windows
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


pub fn schedule_load_image(url: &Url, cookie_store: &CookieStore, resource_thread_pool: &mut ResourceThreadPool) -> ResourceRequestJobTracker<ResourceRequestResult<RgbaImage>> {
    let (sender, receiver) = channel::<ResourceRequestResult<RgbaImage>>();
    let job_id = get_next_job_id();

    let cookies = cookie_store.get_for_domain(&url.host);

    let job = ResourceRequestJob { job_id, url: url.clone(), sender, request_type: RequestType::Get, body: None, cookies: cookies };
    let job_tracker = ResourceRequestJobTracker { job_id, receiver };

    resource_thread_pool.fire_and_forget_load_image(job);

    return job_tracker;
}


fn load_image(url: &Url, cookies: &HashMap<String, String>) -> ResourceRequestResult<RgbaImage> {
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

        return ResourceRequestResult::Success(ResourceRequestResultSuccess { body: dyn_image.to_rgba8(), new_cookies: HashMap::new() });
    }

    let extension = url.file_extension();
    if extension.is_some() && extension.unwrap() == "svg".to_owned() {
        //svg is currently not implemented
        debug_log_warn(format!("Svg's are not supported currently: {}", url.to_string()));
        return ResourceRequestResult::Success(ResourceRequestResultSuccess {  body: fallback_image(), new_cookies: HashMap::new() });
    }
    if url.scheme == "data".to_owned() {
        //data scheme is currently not implemented
        debug_log_warn(format!("the data: scheme is not supported currently: {}", url.to_string()));
        return ResourceRequestResult::Success(ResourceRequestResultSuccess {  body: fallback_image(), new_cookies: HashMap::new() });
    }

    #[cfg(debug_assertions)] println!("loading {}", url.to_string()); //TODO: debug mode should have a more general way of logging all HTTP request/responses

    return http_get_image(url, cookies);
}


pub fn fallback_image() -> RgbaImage {
    //TODO: this should become one of those "broken image"-images
    return ImageBuffer::new(1, 1);
}
