use super::{inverted_index::InvertedIndex, UPLOADS_DIR};
use log::{error, info, warn};
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str;
use std::sync::{Arc, Mutex};

const BUFFER_SIZE: usize = 8192;

enum Command {
    Upload,
    Search,
    Delete,
    Unknown,
}

pub struct Handler {
    stream: TcpStream,
    index: Arc<Mutex<InvertedIndex>>,
}

impl Handler {
    pub fn new(stream: TcpStream, inverted_index: Arc<Mutex<InvertedIndex>>) -> Handler {
        Handler {
            stream,
            index: inverted_index,
        }
    }

    pub fn handle_client(&mut self) {
        let mut stream = &self.stream;
        let mut buffer = [0; 6];

        stream
            .read_exact(&mut buffer)
            .expect("Failed to read command");

        let command = match &buffer {
            b"UPLOAD" => Command::Upload,
            b"SEARCH" => Command::Search,
            b"DELETE" => Command::Delete,
            _ => Command::Unknown,
        };

        match command {
            Command::Upload => self.handle_upload(),
            Command::Search => self.handle_search(),
            Command::Delete => self.handle_delete(),
            Command::Unknown => {
                error!("Unknown command received");
            }
        }
    }

    fn handle_upload(&self) {
        let mut stream = &self.stream;

        let file_size = self.read_size();

        let filename = format!("{}.txt", uuid::Uuid::new_v4().to_string());

        info!("Receiving file: {}", filename);

        let upload_path = format!("{UPLOADS_DIR}/{filename}");

        let mut file = match File::create(&upload_path) {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to create file: {}", e);
                // TODO: remove all such return and return errors instead
                return;
            }
        };

        let buffer_size = std::cmp::min(file_size, BUFFER_SIZE);
        let mut buffer = vec![0; buffer_size];

        let mut bytes_remaining = file_size;

        while bytes_remaining > 0 {
            let bytes_to_read = std::cmp::min(buffer_size, bytes_remaining) as usize;

            match stream.read(&mut buffer[..bytes_to_read]) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        warn!("Client disconnected unexpectedly.");
                        return;
                    }
                    if let Err(e) = file.write_all(&buffer[..bytes_read]) {
                        error!("Failed to write file content: {}", e);
                        return;
                    }

                    bytes_remaining -= bytes_read;
                }
                Err(e) => {
                    error!("Failed to read file content: {}", e);
                    return;
                }
            }
        }

        let index = self.index.lock().unwrap();

        index.add_document(upload_path);

        let response = b"UPLOADED";
        stream
            .write_all(response)
            .expect("Failed to write response");

        info!("File upload complete");
    }

    fn handle_search(&self) {
        let mut stream = &self.stream;

        let search_term_size = self.read_size();

        let mut buffer = vec![0; search_term_size];

        if let Err(e) = stream.read_exact(&mut buffer) {
            error!("Failed to read file name: {}", e);
            return;
        }

        let search_term = match str::from_utf8(&buffer) {
            Ok(name) => name,
            Err(_) => {
                error!("Invalid UTF-8 in search term");
                return;
            }
        };

        info!("Searching for term: {search_term}");

        let document_ids = self
            .index
            .lock()
            .unwrap()
            .search(search_term)
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<String>>()
            .join(",");

        let response = b"FOUND";
        stream
            .write_all(response)
            .expect("Failed to write response");

        stream
            .write_all(&document_ids.as_bytes())
            .expect("Failed to write");

        info!("Search complete. Found documents: [{document_ids}]");
    }

    fn handle_delete(&self) {
        let mut stream = &self.stream;

        let mut buffer = [0; 8];

        if let Err(e) = stream.read_exact(&mut buffer) {
            error!("Failed to read document ID: {}", e);
            return;
        }
        let document_id = u64::from_be_bytes(buffer);

        info!("Deleting document with ID: {document_id}");

        self.index.lock().unwrap().delete_document(document_id);

        let response = b"DELETED";
        stream
            .write_all(response)
            .expect("Failed to write response");

        info!("Document deleted");
    }

    fn read_size(&self) -> usize {
        let mut buffer = [0; 8];

        let mut stream = &self.stream;

        stream.read_exact(&mut buffer).expect("Failed to read size");

        usize::from_be_bytes(buffer)
    }
}
