use crate::inverted_index::InvertedIndex;
use crate::threadpool::ThreadPool;
use std::sync::{Arc, Mutex};

pub enum Task {
    AddDocument(String),
    DeleteDocument(u64),
}

pub struct Scheduler {
    inverted_index: Arc<Mutex<InvertedIndex>>,
    thread_pool: ThreadPool,
}

impl Scheduler {
    pub fn new(inverted_index: Arc<Mutex<InvertedIndex>>) -> Self {
        let cpu_count = num_cpus::get();
        let thread_pool = ThreadPool::new(cpu_count);

        Scheduler {
            inverted_index,
            thread_pool,
        }
    }
    pub fn run(&self, task: Task) {
        let inverted_index = Arc::clone(&self.inverted_index);
        self.thread_pool.execute(move || match task {
            Task::AddDocument(document) => {
                let index = inverted_index.lock().unwrap();
                index.add_document(document);
            }
            Task::DeleteDocument(document_id) => {
                let index = inverted_index.lock().unwrap();
                if let Err(e) = index.delete_document(document_id) {
                    log::error!("Failed to delete document: {e}");
                }
            }
        });
    }
}
