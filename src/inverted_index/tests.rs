
use super::*;
use std::fs::{self, File};
use std::io::Write;

const TEST_STATE_FILE: &str = "index.json";

// Mock STATE_FILE
lazy_static::lazy_static! {
    static ref TEST_STATE_FILE_PATH: String = TEST_STATE_FILE.to_string();
}

fn setup() {
    let _ = fs::remove_file(&*TEST_STATE_FILE_PATH); // Ensure no state file exists
}

fn teardown() {
    let _ = fs::remove_file(&*TEST_STATE_FILE_PATH); // Clean up
}

fn create_test_file(content: &str) -> String {
    let file_path = format!("test_file_{}.txt", uuid::Uuid::new_v4());
    let mut file = File::create(&file_path).expect("Failed to create test file");
    writeln!(file, "{}", content).expect("Failed to write to test file");
    file_path
}

#[test]
fn test_new_creates_empty_index() {
    setup();
    let index = InvertedIndex::new();
    assert_eq!(index.get_document_count(), 0);
    teardown();
}

#[test]
fn test_add_document() {
    setup();
    let index = InvertedIndex::new();
    let file_path = create_test_file("rust programming language");

    index.add_document(file_path.clone());
    assert_eq!(index.get_document_count(), 1);

    let search_results = index.search("rust");
    assert_eq!(search_results.len(), 1);

    fs::remove_file(file_path).unwrap();
    teardown();
}

#[test]
fn test_search() {
    setup();
    let index = InvertedIndex::new();
    let file1 = create_test_file("rust programming language");
    let file2 = create_test_file("rustaceans love rust");

    index.add_document(file1.clone());
    index.add_document(file2.clone());

    let search_results = index.search("rust");
    assert_eq!(search_results.len(), 2);

    fs::remove_file(file1).unwrap();
    fs::remove_file(file2).unwrap();
    teardown();
}

#[test]
fn test_delete_document() {
    setup();
    let index = InvertedIndex::new();
    let file_path = create_test_file("hello world");

    index.add_document(file_path.clone());
    let doc_id = index
        .last_document_id
        .load(std::sync::atomic::Ordering::SeqCst)
        - 1;

    assert!(index.document_exists(doc_id));
    index.delete_document(doc_id).unwrap();
    assert!(!index.document_exists(doc_id));

    teardown();
}

#[test]
fn test_save_and_load() {
    setup();
    {
        let index = InvertedIndex::new();
        let file_path = create_test_file("save and load test");
        index.add_document(file_path.clone());
        fs::remove_file(file_path).unwrap();
    }

    let index = InvertedIndex::new();
    assert_eq!(index.get_document_count(), 1);
    teardown();
}

#[test]
fn test_get_document_path() {
    setup();
    let index = InvertedIndex::new();
    let file_path = create_test_file("document path test");

    index.add_document(file_path.clone());
    let doc_id = index
        .last_document_id
        .load(std::sync::atomic::Ordering::SeqCst)
        - 1;

    assert_eq!(index.get_document_path(doc_id), Some(file_path.clone()));
    fs::remove_file(file_path).unwrap();
    teardown();
}
