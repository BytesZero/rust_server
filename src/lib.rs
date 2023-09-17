use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// 创建线程池
    ///
    /// 线程池中线程的数量
    ///
    /// # Panics
    ///
    /// `new` 函数在 size 为 0 时会 panic
    ///
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let (sender, recevier) = mpsc::channel();
        let recevier = Arc::new(Mutex::new(recevier));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&recevier)));
        }
        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }
    /// 在线程池中执行一个任务
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

// impl Drop for ThreadPool {
//     fn drop(&mut self) {
//         println!("Sending terminate message to all workers.");
//         drop(self.sender.take());
//         for worker in &mut self.workers {
//             println!("Shutting down worker {}", worker.id);
//             if let Some(thread) = worker.thread.take() {
//                 thread.join().unwrap();
//             }
//         }
//     }
// }

impl Worker {
    /// 创建一个新的 Worker
    ///
    /// id 是这个 Worker 在线程池中的编号
    fn new(id: usize, recevier: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = recevier.lock().unwrap().recv();
            match job {
                Ok(job) => {
                    println!("Worker {} got a job; executing.", id);
                    job();
                }
                Err(_) => {
                    println!("Worker {} got a job; executing.", id);
                    break;
                }
            }
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}
