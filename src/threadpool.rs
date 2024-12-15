use log::info;
use std::{sync::Arc, thread};

use crate::channel::Channel;

pub struct ThreadPool {
    workers: Vec<Worker>,
    channel: Option<Arc<Channel<Job>>>, // sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + Sync + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let channel = Arc::new(Channel::new());

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&channel)));
        }

        ThreadPool {
            workers,
            channel: Some(channel),
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        let job = Box::new(f);

        self.channel.as_ref().unwrap().send(job);
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.channel.take());

        for worker in &mut self.workers {
            info!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Channel<Job>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.receive();

            match message {
                Some(job) => {
                    info!("Worker {id} got a job; executing.");

                    job();
                }
                None => {
                    info!("Worker {id} disconnected; shutting down.");
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
