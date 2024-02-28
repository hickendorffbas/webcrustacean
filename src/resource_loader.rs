use std::fs;
use std::sync::atomic::{Ordering, AtomicUsize};
use std::sync::mpsc::{channel, Receiver, Sender};

use image::DynamicImage;
use image::io::Reader as ImageReader;
use threadpool::ThreadPool;

use crate::debug::debug_log_warn;
use crate::network::url::Url;
use crate::network::{
    http_get_image,
    http_get_text,
};


static NEXT_JOB_ID: AtomicUsize = AtomicUsize::new(1);
pub fn get_next_job_id() -> usize { NEXT_JOB_ID.fetch_add(1, Ordering::Relaxed) }

struct ResourceRequestJob<T> {
    #[allow(dead_code)] job_id: usize, //TODO: check if we want to use this (probably for logging / debugging?)
    url: Url,
    sender: Sender<T>,
}
#[derive(Debug)]
pub struct ResourceRequestJobTracker<T> {
    pub job_id: usize,
    pub receiver: Receiver<T>,
}


pub struct ResourceThreadPool {
    pub pool: ThreadPool,
}
impl ResourceThreadPool {
    fn fire_and_forget_load_image(&mut self, job: ResourceRequestJob<DynamicImage>) {
        self.pool.execute(move || {
            let result = load_image(&job.url);
            job.sender.send(result).expect("Could not send over channel");
        });
    }
    fn fire_and_forget_load_text(&mut self, job: ResourceRequestJob<String>) {
        self.pool.execute(move || {
            let result = load_text(&job.url);
            job.sender.send(result).expect("Could not send over channel");
        });
    }
}


pub fn schedule_load_text(url: &Url, resource_thread_pool: &mut ResourceThreadPool) -> ResourceRequestJobTracker<String> {
    let (sender, receiver) = channel::<String>();
    let job_id = get_next_job_id();

    let job = ResourceRequestJob { job_id, url: url.clone(), sender };
    let job_tracker = ResourceRequestJobTracker { job_id, receiver };

    resource_thread_pool.fire_and_forget_load_text(job);

    return job_tracker;
}


fn load_text(url: &Url) -> String {
    if url.scheme == "file" {
        let mut local_path = String::from("//");
        local_path.push_str(&url.path.join("/"));
        let read_result = fs::read_to_string(local_path);
        if read_result.is_err() {
            debug_log_warn(format!("Could not load text: {}", url.to_string()));
            return String::new();
        }

        return read_result.unwrap();
    }

    let file_content_result = http_get_text(url);

    if file_content_result.is_err() {
        debug_log_warn(format!("Could not load text: {}", url.to_string()));
        return String::new();
    }

    return file_content_result.unwrap();
}


pub fn schedule_load_image(url: &Url, resource_thread_pool: &mut ResourceThreadPool) -> ResourceRequestJobTracker<DynamicImage> {
    let (sender, receiver) = channel::<DynamicImage>();
    let job_id = get_next_job_id();

    let job = ResourceRequestJob { job_id, url: url.clone(), sender };
    let job_tracker = ResourceRequestJobTracker { job_id, receiver };

    resource_thread_pool.fire_and_forget_load_image(job);

    return job_tracker;
}


fn load_image(url: &Url) -> DynamicImage {
    if url.scheme == "file" {
        let mut local_path = String::from("//");
        local_path.push_str(&url.path.join("/"));
        let read_result = ImageReader::open(local_path);
        if read_result.is_err() {
            debug_log_warn(format!("Could not load image: {}", url.to_string()));
            return fallback_image();
        }

        let file_data = read_result.unwrap();
        let format_guess_result = file_data.with_guessed_format();

        if format_guess_result.is_ok() {
            return format_guess_result.ok().unwrap().decode().expect("decoding the image failed"); //TODO: we need to handle this in a better way
        } else {
            panic!("decoding the image failed"); //TODO: we need to handle this in a better way
        }
    }

    let extension = url.file_extension();
    if extension.is_some() && extension.unwrap() == "svg".to_owned() {
        //svg is currently not implemented
        debug_log_warn(format!("Svg's are not supported currently: {}", url.to_string()));
        return fallback_image();
    }
    if url.scheme == "data".to_owned() {
        //data scheme is currently not implemented
        debug_log_warn(format!("the data: scheme is not supported currently: {}", url.to_string()));
        return fallback_image();
    }

    #[cfg(debug_assertions)] println!("loading {}", url.to_string());

    let image_result = http_get_image(url);
    if image_result.is_err() {
        debug_log_warn(format!("Could not load image: {}", url.to_string()));
        return fallback_image();
    }

    return image_result.unwrap();
}


pub fn fallback_image() -> DynamicImage {
    //TODO: this should become one of those "broken image"-images
    return DynamicImage::new_rgb8(1, 1);
}
