use super::inverted_index::InvertedIndex;
use std::sync::{mpsc::Receiver, Arc, Mutex};

pub enum Task {
    AddDocument(String),
    DeleteDocument(u64),
}

pub struct Scheduler {
    inverted_index: Arc<Mutex<InvertedIndex>>,
    task_receiver: Receiver<Task>,
}

impl Scheduler {
    pub fn new(inverted_index: Arc<Mutex<InvertedIndex>>, task_receiver: Receiver<Task>) -> Self {
        Scheduler {
            inverted_index,
            task_receiver,
        }
    }
    pub fn run(&self) {
        for task in &self.task_receiver {
            match task {
                Task::AddDocument(path) => {
                    let inverted_index = self.inverted_index.lock().unwrap();
                    inverted_index.add_document(path);
                }
                Task::DeleteDocument(id) => {
                    let inverted_index = self.inverted_index.lock().unwrap();
                    inverted_index.delete_document(id);
                }
            }
        }
    }
}
