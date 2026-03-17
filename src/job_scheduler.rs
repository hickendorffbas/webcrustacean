use std::collections::HashMap;
use std::sync::mpsc;
use std::thread;
use threadpool::ThreadPool;

use crate::network::url::Url;
use crate::resource_loader::{
    load_text,
    RequestType,
    ResourceRequestResult,
};


pub enum Job {
    HttpGetText {
        location: Url,
        cookies: HashMap<String, String>,
        result_sender: mpsc::Sender<JobResult>,
    },
}

pub enum JobResult {
    ResourceRequestResultString {
        value: ResourceRequestResult<String>,
    },
    //TODO: add some binary type for loading images etc.
}

pub struct JobScheduler {
    sender: mpsc::Sender<Job>,
    _dispatcher: thread::JoinHandle<()>,
}

impl JobScheduler {
    pub fn new(max_concurrent_jobs: usize) -> Self {
        let (sender, receiver) = mpsc::channel::<Job>();
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

    pub fn submit_http_get_text_job(&self, url: &Url, cookies: HashMap<String, String>) -> mpsc::Receiver<JobResult> {
        let (tx, rx) = mpsc::channel();

        let job = Job::HttpGetText { location: url.clone(), cookies: cookies, result_sender: tx };

        let _ = self.sender.send(job);
        return rx;
    }

    fn run_job(job: Job) {
        match job {
            Job::HttpGetText { location, cookies, result_sender } => {
                let result = load_text(&location, RequestType::Get, None, &cookies);
                let _ = result_sender.send(JobResult::ResourceRequestResultString { value: result });
            }
        }
    }
}
