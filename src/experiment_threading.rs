use threadpool::ThreadPool;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::{thread, time};


struct ResourceRequestJob {
    job_id: usize,
    url: String,
    sender: Sender<String>,
}
struct ResourceRequestJobTracker {
    job_id: usize,
    receiver: Receiver<String>,
}


struct PoolContext {
    pool: ThreadPool,
}
impl PoolContext {
    fn fire_and_forget(&mut self, job: ResourceRequestJob) {
        self.pool.execute(move || {
            let result = fetch_resource(job.job_id, &job.url);
            job.sender.send(result).expect("Could not send over channel");
        });
    }
}


fn fetch_resource(job_id: usize, url: &String) -> String {
    println!("{job_id} starting to get {url}");
    thread::sleep(time::Duration::from_millis((url.len() * 100) as u64));
    println!("{job_id} finished");

    return String::from("response");
}


static NEXT_JOB_ID: AtomicUsize = AtomicUsize::new(1);
pub fn get_next_job_id() -> usize { NEXT_JOB_ID.fetch_add(1, Ordering::Relaxed) }


fn build_request_job(url: String) -> (ResourceRequestJob, ResourceRequestJobTracker) {
    let (sender, receiver) = channel::<String>();
    let job_id = get_next_job_id();

    return (
        ResourceRequestJob { job_id, url, sender },
        ResourceRequestJobTracker { job_id, receiver },
    );
}


pub fn run_experiment() {
    let mut pool_context = PoolContext { pool: ThreadPool::new(3) };

    let resources: [&str; 5] = [
        "https://www.google.com",
        "http://www.bashickendorff.nl",
        "https://en.wikipedia.org/wiki/Main_Page",
        "https://cnn.com",
        "https://www.rust-lang.org/",
    ];

    let mut jobs = Vec::new();
    for resource in resources {
        let (job, job_tracker) = build_request_job(String::from(resource));

        jobs.push(job_tracker);
        pool_context.fire_and_forget(job);
    }

    for job in jobs {
        println!("RECV {}: {}", job.job_id, job.receiver.recv().unwrap());
    }

    println!("DONE");
}
