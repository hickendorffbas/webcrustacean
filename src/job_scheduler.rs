use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{
    channel,
    Receiver,
    Sender,
};
use std::thread;
use image::RgbaImage;
use threadpool::ThreadPool;

use crate::network::url::Url;
use crate::resource_loader::{
    self,
    RequestType,
    ResourceRequestResult,
};


static NEXT_JOB_ID: AtomicUsize = AtomicUsize::new(1);
pub fn get_next_job_id() -> usize { NEXT_JOB_ID.fetch_add(1, Ordering::Relaxed) }

static NEXT_TASK_ID: AtomicUsize = AtomicUsize::new(1);
pub fn get_next_task_id() -> usize { NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed) }

#[cfg_attr(debug_assertions, derive(Debug))]
pub enum TaskPayload {
    ParseJs { script_data: String },
    StartParseHtml { html: String },
    SetImageOnDomNode { dom_node_id: usize, image: Option<Rc<RgbaImage>>}
}

//TODO: all this stuff should live in some task store module
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct Task {
    pub id: usize,
    pub payload: TaskPayload,
    pub ready: bool,
    pub finished: bool,
}
impl Task {
    pub fn new(payload: TaskPayload) -> Task {
        return Task::new_with_id(get_next_task_id(), payload);
    }
    pub fn new_with_id(id: usize, payload: TaskPayload) -> Task {
        return Task {
            id,
            payload,
            ready: true,
            finished: false,
        };
    }
    pub fn new_task_not_yet_ready(payload: TaskPayload) -> Task {
        return Task {
            id: get_next_task_id(),
            payload,
            ready: false,
            finished: false,
        }
    }
}


pub enum Job {
    HttpGetText {
        job_id: usize, //TODO: do we use this for anything?
        location: Url,
        cookies: HashMap<String, String>,
        result_sender: Sender<JobResult>,
    },
    HttpGetImage {
        job_id: usize, //TODO: do we use this for anything?
        location: Url,
        cookies: HashMap<String, String>,
        result_sender: Sender<JobResult>,
    }
}

pub enum JobResult {
    ResourceRequestResultString {
        value: ResourceRequestResult<String>,
    },
    ResourceRequestResultImage {
        value: ResourceRequestResult<RgbaImage>,
    },}

pub struct JobScheduler {
    sender: Sender<Job>,
    _dispatcher: thread::JoinHandle<()>,
}

impl JobScheduler {
    pub fn new(max_concurrent_jobs: usize) -> Self {
        let (sender, receiver) = channel::<Job>();
        let pool = ThreadPool::new(max_concurrent_jobs);

        let dispatcher = thread::spawn(move || {
            while let Ok(job) = receiver.recv() {
                pool.execute(move || {
                    Self::run_job(job);
                });
            }

            pool.join();
        });

        Self {
            sender,
            _dispatcher: dispatcher,
        }
    }

    pub fn submit_http_get_text_job(&self, url: &Url, cookies: HashMap<String, String>) -> Receiver<JobResult> {
        let (tx, rx) = channel();

        let job_id = get_next_job_id();
        let job = Job::HttpGetText { job_id, location: url.clone(), cookies: cookies, result_sender: tx };

        let _ = self.sender.send(job);
        return rx;
    }

    pub fn submit_http_get_image_job(&self, url: &Url, cookies: HashMap<String, String>) -> Receiver<JobResult> {
        let (tx, rx) = channel();

        let job_id = get_next_job_id();
        let job = Job::HttpGetImage { job_id, location: url.clone(), cookies: cookies, result_sender: tx };

        let _ = self.sender.send(job);
        return rx;
    }

    fn run_job(job: Job) {
        match job {
            Job::HttpGetText { job_id: _, location, cookies, result_sender } => {
                let result = resource_loader::load_text(&location, RequestType::Get, None, &cookies);
                let _ = result_sender.send(JobResult::ResourceRequestResultString { value: result });
            },
            Job::HttpGetImage { job_id: _, location, cookies, result_sender } => {
                let result = resource_loader::load_image(&location, &cookies);
                let _ = result_sender.send(JobResult::ResourceRequestResultImage { value: result });
            }
        }
    }
}
