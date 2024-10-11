use env_logger;
use log::{error, info, warn};
use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::str;

mod threadpool;

enum Command {
    Upload,
    Search,
    Unknown,
}

struct Handler {
    stream: TcpStream,
}

impl Handler {
    pub fn new(stream: TcpStream) -> Handler {
        Handler { stream }
    }

    fn handle_upload(&mut self) {
        let mut stream = &self.stream;

        let mut buffer = [0; 1024];
        stream
            .read_exact(&mut buffer[..8])
            .expect("Failed to read file size");
        let file_size = u64::from_be_bytes(buffer[..8].try_into().unwrap());

        stream
            .read_exact(&mut buffer[..4])
            .expect("Failed to read file name length");
        let file_name_len = u32::from_be_bytes(buffer[..4].try_into().unwrap());

        stream
            .read_exact(&mut buffer[..file_name_len as usize])
            .expect("Failed to read file name");
        let file_name =
            str::from_utf8(&buffer[..file_name_len as usize]).expect("Invalid UTF-8 file name");

        info!("Receiving file: {}", file_name);

        let upload_path = Path::new("uploads").join(file_name);
        let mut file = match File::create(&upload_path) {
            Ok(f) => f,
            Err(e) => {
                error!("Failed to create file: {}", e);
                return;
            }
        };

        let mut bytes_remaining = file_size;
        while bytes_remaining > 0 {
            let bytes_to_read = std::cmp::min(buffer.len() as u64, bytes_remaining) as usize;
            match stream.read(&mut buffer[..bytes_to_read]) {
                Ok(bytes_read) => {
                    if bytes_read == 0 {
                        warn!("Client disconnected unexpectedly.");
                        return;
                    }
                    file.write_all(&buffer[..bytes_read])
                        .expect("Failed to write file content");
                    bytes_remaining -= bytes_read as u64;
                }
                Err(e) => {
                    error!("Failed to read file content: {}", e);
                    return;
                }
            }
        }

        info!("File upload complete");
    }

    fn handle_client(&mut self) {
        let mut stream = &self.stream;
        let mut buffer = [0; 1024];

        stream
            .read_exact(&mut buffer[..6])
            .expect("Failed to read command");
        let command = match &buffer[..6] {
            b"UPLOAD" => Command::Upload,
            b"SEARCH" => Command::Search,
            _ => Command::Unknown,
        };

        match command {
            Command::Upload => self.handle_upload(),
            Command::Search => {
                info!("Search command received");
                // Search functionality will be implemented here.
            }
            Command::Unknown => {
                error!("Unknown command received");
            }
        }
    }
}

fn main() {
    env_logger::init();
    let listener = TcpListener::bind("127.0.0.1:7878").expect("Could not bind to address");

    std::fs::create_dir_all("uploads").expect("Failed to create uploads directory");

    info!("Server listening on 127.0.0.1:7878");

    let thread_pool = threadpool::ThreadPool::new(4);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                info!("New connection established");
                let mut handler = Handler::new(stream);
                thread_pool.execute(move || handler.handle_client());
            }
            Err(e) => {
                error!("Error accepting connection: {}", e);
            }
        }
    }
}
