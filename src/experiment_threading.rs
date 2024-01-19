use threadpool::ThreadPool;
use std::sync::mpsc::{channel, Sender};
use std::{thread, time};


//TODO: return the response over the channel
//TODO: build a update loop over all the inprogress work, checking for results on the channel


struct ResourceRequestJob {
    job_id: usize,
    url: String,
    sender: Sender<usize>,
}


struct PoolContext {
    pool: ThreadPool,
}
impl PoolContext {
    fn fire_and_forget(&mut self, job: ResourceRequestJob) {
        self.pool.execute(move || {
            run_long_job(job.job_id, &job.url);
            job.sender.send(job.job_id).expect("Could not send over channel");
        });
    }
}


fn run_long_job(job_id: usize, url: &String) {
    println!("{job_id} starting to get {url}");
    thread::sleep(time::Duration::from_millis((url.len() * 100) as u64));
    println!("{job_id} finished");
}


pub fn run_experiment() {
    //TODO: This makes no sense, building a mut struct out of non-mut members, and later mutting the members?
    let pool = ThreadPool::new(3);
    let mut pool_context = PoolContext { pool };

    let (tx, rx) = channel();
    
    pool_context.fire_and_forget(ResourceRequestJob { job_id: 1, url: String::from("http://www.google.com"), sender: tx.clone() });
    pool_context.fire_and_forget(ResourceRequestJob { job_id: 2, url: String::from("http://www.google.com"), sender: tx.clone() });
    pool_context.fire_and_forget(ResourceRequestJob { job_id: 3, url: String::from("http://www.google.com"), sender: tx.clone() });
    pool_context.fire_and_forget(ResourceRequestJob { job_id: 4, url: String::from("http://www.google.com"), sender: tx.clone() });
    pool_context.fire_and_forget(ResourceRequestJob { job_id: 5, url: String::from("http://www.google.com"), sender: tx.clone() });
    pool_context.fire_and_forget(ResourceRequestJob { job_id: 6, url: String::from("http://www.google.com"), sender: tx.clone() });


    for _ in 0..6 {
        println!("RECV: {}", rx.recv().unwrap());
    }

    println!("DONE");
}
