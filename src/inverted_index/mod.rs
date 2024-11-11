#[cfg(test)]
mod tests;
mod tokenize;

use super::STATE_FILE;
use log::{error, info, warn};
use std::collections::{HashMap, BTreeSet};
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct InvertedIndex {
    // Word -> IDs
    index: Arc<Mutex<HashMap<String, BTreeSet<u64>>>>,
    // ID -> Document file path
    documents: Arc<Mutex<HashMap<u64, String>>>,
    // ID counter
    last_document_id: AtomicU64,
}

impl InvertedIndex {
    pub fn new() -> Self {
        if let Some(index) = Self::from_file() {
            info!("State file found, loading index");
            return index;
        }

        warn!("State file not found, creating new index");

        InvertedIndex {
            index: Arc::new(Mutex::new(HashMap::new())),
            documents: Arc::new(Mutex::new(HashMap::new())),
            last_document_id: AtomicU64::new(0),
        }
    }

    fn from_file() -> Option<Self> {
        let raw_data = std::fs::read_to_string(STATE_FILE).ok()?;

        let raw_data: serde_json::Value =
            serde_json::from_str(&raw_data).expect("Failed to parse JSON");

        let index = raw_data["index"]
            .as_object()
            .expect("Failed to parse index")
            .iter()
            .map(|(word, ids)| {
                let ids = ids
                    .as_array()
                    .expect("Failed to parse IDs")
                    .iter()
                    .map(|id| id.as_u64().expect("Failed to parse ID") as u64)
                    .collect();
                (word.to_string(), ids)
            })
            .collect();

        let documents = raw_data["documents"]
            .as_object()
            .expect("Failed to parse documents")
            .iter()
            .map(|(id, document)| {
                let id = id.parse().expect("Failed to parse ID");
                let document = document
                    .as_str()
                    .expect("Failed to parse document")
                    .to_string();
                (id, document)
            })
            .collect();

        let last_document_id = raw_data["last_document_id"]
            .as_u64()
            .expect("Failed to parse last_document_id") as u64;

        Some(Self {
            index: Arc::new(Mutex::new(index)),
            documents: Arc::new(Mutex::new(documents)),
            last_document_id: AtomicU64::new(last_document_id),
        })
    }

    fn to_file(&self) {
        let index = self.index.lock().unwrap();

        let documents = self.documents.lock().unwrap();

        let last_document_id = self
            .last_document_id
            .load(std::sync::atomic::Ordering::SeqCst);

        let data = serde_json::json!({
            "index": index.clone(),
            "documents": documents.clone(),
            "last_document_id": last_document_id,
        });

        let data = serde_json::to_string_pretty(&data).expect("Failed to serialize JSON");

        std::fs::write(STATE_FILE, data).expect("Failed to write file");
    }

    pub fn add_document(&self, path: String) {
        let content = if let Ok(content) = std::fs::read_to_string(&path) {
            tokenize::tokenize(&content)
        } else {
            error!("Failed to read document: {path}");
            return;
        };

        let document_id = self
            .last_document_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let mut documents = self.documents.lock().unwrap();

        let mut index = self.index.lock().unwrap();

        documents.insert(document_id, path.clone());

        for word in content {
            index
                .entry(word.to_string())
                .or_insert_with(BTreeSet::new)
                .insert(document_id);
        }

        drop(documents);
        drop(index);

        self.to_file();

        info!("Document added {document_id} - {path}");
    }

    pub fn search(&self, query: &str) -> Vec<u64> {
        let index = self.index.lock().unwrap();

        let words = tokenize::tokenize(query);

        words
            .iter()
            .flat_map(|word| index.get(word))
            .flatten()
            .cloned()
            .collect()
    }

    pub fn delete_document(&self, document_id: u64) {
        let mut documents = self.documents.lock().unwrap();

        if let Some(path) = documents.remove(&document_id) {
            if let Err(e) = std::fs::remove_file(&path) {
                error!("Failed to delete document: {e}");
            }
        }

        let mut index = self.index.lock().unwrap();

        for ids in index.values_mut() {
            ids.retain(|&id| id != document_id);
        }

        drop(documents);
        drop(index);

        self.to_file();
    }

    pub fn get_document_path(&self, document_id: u64) -> Option<String> {
        let documents = self.documents.lock().unwrap();

        documents.get(&document_id).cloned()
    }
}
