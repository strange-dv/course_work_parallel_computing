use crate::inverted_index::InvertedIndex;
use crate::threadpool::ThreadPool;
use log::debug;
use std::sync::Arc;

pub enum Task {
    AddDocument(String),
    DeleteDocument(u64),
}

pub struct Scheduler {
    inverted_index: Arc<InvertedIndex>,
    thread_pool: ThreadPool,
}

impl Scheduler {
    pub fn new(num_threads: usize, inverted_index: Arc<InvertedIndex>) -> Self {
        let thread_pool = ThreadPool::new(num_threads);

        Scheduler {
            inverted_index,
            thread_pool,
        }
    }
    pub fn run(&self, task: Task) {
        let inverted_index = Arc::clone(&self.inverted_index);
        self.thread_pool.execute(move || {
            let start = std::time::Instant::now();

            match task {
                Task::AddDocument(document) => {
                    inverted_index.add_document(document.clone());
                }
                Task::DeleteDocument(document_id) => {
                    if let Err(e) = inverted_index.delete_document(document_id) {
                        log::error!("Failed to delete document: {e}");
                    }
                }
            }

            let elapsed = start.elapsed();
            debug!("Job executed in {elapsed:?}");
        });
    }
}
