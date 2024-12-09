#[cfg(test)]
mod tests;
mod tokenize;

use super::STATE_FILE;
use log::{error, info};
use std::collections::HashSet;
use std::collections::{BTreeSet, HashMap};
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct InvertedIndex {
    // Word -> IDs
    index: Arc<RwLock<HashMap<String, BTreeSet<u64>>>>,
    // ID -> Document file path
    documents: Arc<RwLock<HashMap<u64, String>>>,
    // ID counter
    last_document_id: AtomicU64,
}

impl InvertedIndex {
    pub fn new() -> Self {
        if let Some(index) = Self::load() {
            info!("State file found, loading index");
            return index;
        }

        info!("State file not found, creating new index");

        InvertedIndex {
            index: Arc::new(RwLock::new(HashMap::new())),
            documents: Arc::new(RwLock::new(HashMap::new())),
            last_document_id: AtomicU64::new(0),
        }
    }

    fn load() -> Option<Self> {
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
            index: Arc::new(RwLock::new(index)),
            documents: Arc::new(RwLock::new(documents)),
            last_document_id: AtomicU64::new(last_document_id),
        })
    }

    pub fn save(&self) {
        info!("Saving index state");
        let index = self.index.read().unwrap();

        let documents = self.documents.read().unwrap();

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

        {
            let mut documents = self.documents.write().unwrap();
            documents.insert(document_id, path.clone());
        }

        {
            let mut index = self.index.write().unwrap();

            for word in content {
                index
                    .entry(word)
                    .and_modify(|ids| {
                        ids.insert(document_id);
                    })
                    .or_insert_with(|| {
                        let mut ids = BTreeSet::new();
                        ids.insert(document_id);
                        ids
                    });
            }
        }
    }

    pub fn search(&self, query: &str) -> HashSet<u64> {
        let words = tokenize::tokenize(query);

        let index = self.index.read().unwrap();

        let result = words
            .iter()
            .flat_map(|word| index.get(word))
            .flatten()
            .cloned()
            .collect();

        info!("Search results: {result:#?}");

        result
    }

    pub fn delete_document(&self, document_id: u64) -> std::io::Result<()> {
        {
            let mut documents = self.documents.write().unwrap();

            if let Some(path) = documents.remove(&document_id) {
                std::fs::remove_file(&path)?;
            }
        }

        {
            let mut index = self.index.write().unwrap();

            for ids in index.values_mut() {
                ids.retain(|&id| id != document_id);
            }
        }

        info!("Document deleted: {document_id}");

        Ok(())
    }

    pub fn document_exists(&self, document_id: u64) -> bool {
        self.documents.read().unwrap().contains_key(&document_id)
    }

    pub fn get_document_path(&self, document_id: u64) -> Option<String> {
        self.documents.read().unwrap().get(&document_id).cloned()
    }

    pub fn get_document_count(&self) -> usize {
        self.documents.read().unwrap().len()
    }
}

impl Drop for InvertedIndex {
    fn drop(&mut self) {
        self.save();
    }
}
