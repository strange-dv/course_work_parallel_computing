use super::{inverted_index::InvertedIndex, UPLOADS_DIR};
use crate::scheduler::{Scheduler, Task};
use log::{error, info};
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str;
use std::sync::{Arc, Mutex};
use thiserror::Error;

const BUFFER_SIZE: usize = 8192;

enum Command {
    Upload,
    Search,
    Delete,
    Import,
    Unknown(Vec<u8>),
}

#[derive(Error, Debug)]
enum HandlerError {
    #[error("Failed to create file: {0}")]
    FileNotCreated(std::io::Error),

    #[error("Failed to read file size")]
    FailedToReadSize(std::io::Error),

    #[error("Client disconnected unexpectedly")]
    ClientDisconnected(std::io::Error),

    #[error("Failed to read search term")]
    FailedToReadSearchTerm(std::io::Error),

    #[error("Failed to decode search term")]
    FailedToDecodeSearchTerm(std::str::Utf8Error),

    #[error("Failed to read document ID")]
    FailedToReadDocumentId(std::io::Error),

    #[error("Failed to write response")]
    FailedToWriteResponse(std::io::Error),

    #[error("Failed to write content to file)")]
    FailedToWriteFile(std::io::Error),

    #[error("Failed to write response")]
    FailedToWrite(std::io::Error),
}

type HandlerResult<T> = std::result::Result<T, HandlerError>;

pub struct Handler {
    stream: TcpStream,
    inverted_index: Arc<Mutex<InvertedIndex>>,
    scheduler: Arc<Mutex<Scheduler>>,
}

impl Handler {
    pub fn new(
        stream: TcpStream,
        inverted_index: Arc<Mutex<InvertedIndex>>,
        scheduler: Arc<Mutex<Scheduler>>,
    ) -> Handler {
        Handler {
            stream,
            inverted_index,
            scheduler,
        }
    }

    pub fn handle_client(&mut self) {
        let mut stream = &self.stream;
        let mut buffer = [0; 6];

        if let Err(e) = stream.read_exact(&mut buffer) {
            error!("Failed to read command: {e:#?}");
            return;
        }

        let command = match &buffer {
            b"UPLOAD" => Command::Upload,
            b"SEARCH" => Command::Search,
            b"DELETE" => Command::Delete,
            b"IMPORT" => Command::Import,
            _ => Command::Unknown(buffer.to_vec()),
        };

        if let Err(e) = match command {
            Command::Upload => self.handle_upload(),
            Command::Search => self.handle_search(),
            Command::Delete => self.handle_delete(),
            Command::Import => self.handle_download(),
            Command::Unknown(command) => {
                error!("Unknown command received: {command:?}");
                return;
            }
        } {
            if let Err(e) = self.write_response(b"*ERROR*") {
                error!("Failed to write error response: {e:#?}");
            }
            error!("Error handling command: {e:#?}");
        }
    }

    fn handle_upload(&self) -> HandlerResult<()> {
        let mut stream = &self.stream;

        let file_size = self
            .read_usize()
            .map_err(|e| HandlerError::FailedToReadSize(e))?;

        let filename = format!("{}.txt", uuid::Uuid::new_v4().to_string());

        info!("Receiving file: {}", filename);

        let upload_path = format!("{UPLOADS_DIR}/{filename}");

        let mut file = File::create(&upload_path).map_err(|e| HandlerError::FileNotCreated(e))?;

        let buffer_size = std::cmp::min(file_size, BUFFER_SIZE);
        let mut buffer = vec![0; buffer_size];

        let mut bytes_remaining = file_size;

        while bytes_remaining > 0 {
            match stream.read(&mut buffer) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        break;
                    }

                    file.write_all(&buffer[..bytes_read])
                        .map_err(|e| HandlerError::FailedToWriteFile(e))?;

                    bytes_remaining -= bytes_read;
                }
                Err(e) => {
                    return Err(HandlerError::ClientDisconnected(e));
                }
            }
        }

        let task = Task::AddDocument(upload_path.clone());

        self.scheduler.lock().unwrap().run(task);

        self.write_response(b"SUCCESS")?;

        info!("File upload complete");

        Ok(())
    }

    fn handle_search(&self) -> HandlerResult<()> {
        let mut stream = &self.stream;

        let search_term_size = self
            .read_usize()
            .map_err(|e| HandlerError::FailedToReadSize(e))?;

        let mut buffer = vec![0; search_term_size];

        stream
            .read_exact(&mut buffer)
            .map_err(|e| HandlerError::FailedToReadSearchTerm(e))?;

        let search_term =
            str::from_utf8(&buffer).map_err(|e| HandlerError::FailedToDecodeSearchTerm(e))?;

        info!("Searching for term: {search_term}");

        let document_ids = self
            .inverted_index
            .lock()
            .unwrap()
            .search(search_term)
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<String>>()
            .join(",");

        let mut response = Vec::new();
        response.extend_from_slice(b"SUCCESS");
        response.extend_from_slice(&document_ids.as_bytes());

        self.write_response(&response)?;
               
        info!("Search complete.");

        Ok(())
    }

    fn handle_delete(&self) -> HandlerResult<()> {
        let document_id = self
            .read_usize()
            .map_err(|e| HandlerError::FailedToReadDocumentId(e))?;

        info!("Deleting document with ID: {document_id}");

        if !self
            .inverted_index
            .lock()
            .unwrap()
            .document_exists(document_id as u64)
        {
            return self.write_response(b"MISSING");
        }

        let task = Task::DeleteDocument(document_id as u64);

        self.scheduler.lock().unwrap().run(task);

        self.write_response(b"DELETED")?;

        info!("Document deleted");

        Ok(())
    }

    fn handle_download(&self) -> HandlerResult<()> {
        let document_id = self
            .read_usize()
            .map_err(|e| HandlerError::FailedToReadDocumentId(e))?;

        info!("Downloading document with ID: {document_id}");

        let document_path = match self
            .inverted_index
            .lock()
            .unwrap()
            .get_document_path(document_id as u64)
        {
            Some(path) => path,
            None => {
                error!("Requested document not found");
                return Err(HandlerError::FileNotCreated(std::io::Error::from(
                    std::io::ErrorKind::NotFound,
                )));
            }
        };

        self.write_response(b"SUCCESS")?;

        let file = &mut File::open(document_path).map_err(|e| HandlerError::FileNotCreated(e))?;

        loop {
            let buffer = file
                .bytes()
                .take(BUFFER_SIZE)
                .collect::<Result<Vec<u8>, _>>()
                .map_err(|e| HandlerError::FailedToWrite(e))?;

            if buffer.is_empty() {
                break;
            }

            self.write_response(&buffer)?;
        }
        Ok(())
    }

    fn write_response(&self, response: &[u8]) -> HandlerResult<()> {
        let mut stream = &self.stream;

        stream
            .write_all(response)
            .map_err(|e| HandlerError::FailedToWriteResponse(e))
    }

    fn read_usize(&self) -> std::io::Result<usize> {
        let mut buffer = [0; 8];

        let mut stream = &self.stream;

        stream.read_exact(&mut buffer)?;

        Ok(usize::from_be_bytes(buffer))
    }
}
