use clap::{Parser, Subcommand};
use std::{
    error::Error,
    fs::{metadata, File},
    io::{BufWriter, Read, Write},
    net::{SocketAddr, TcpStream},
    path::Path,
    str,
};

const SERVER_ADDRESS: &str = "127.0.0.1:7878";
const MAX_BUFFER_SIZE: usize = 8192;
const MAX_STATUS_SIZE: usize = 7;

fn send_command(command: &str, payload: Vec<u8>) -> Result<String, Box<dyn Error>> {
    let server_address: SocketAddr = SERVER_ADDRESS.parse()?;
    let mut stream = TcpStream::connect(server_address)?;

    let mut data = Vec::new();
    data.extend(command.as_bytes());
    data.extend(payload);

    for chunk in data.chunks(MAX_BUFFER_SIZE) {
        stream.write_all(chunk)?;
    }

    let mut buffer = [0; MAX_BUFFER_SIZE];
    let bytes_read = stream.read(&mut buffer)?;

    let response = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();

    let status = &response[..MAX_STATUS_SIZE];
    if status == "*ERROR*" {
        return Err("Server error, aborting".into());
    }

    println!("Server response: {status}");

    Ok(response)
}

fn send_command_and_download_bytes(
    command: &str,
    payload: Vec<u8>,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let server_address: SocketAddr = SERVER_ADDRESS.parse()?;
    let mut stream = TcpStream::connect(server_address)?;

    stream.write_all(command.as_bytes())?;
    stream.write_all(&payload)?;

    let mut response = Vec::new();
    let mut buffer = [0; MAX_BUFFER_SIZE];

    loop {
        let bytes_read = stream.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        response.extend_from_slice(&buffer[..bytes_read]);
    }

    let status = &response[..MAX_STATUS_SIZE];
    println!("Server response: {}", String::from_utf8_lossy(status).to_string());
    if status == *b"*ERROR*" {
        return Err("Server error, aborting".into());
    }


    Ok(response)
}

fn upload(file_path: &str) -> Result<(), Box<dyn Error>> {
    if !Path::new(file_path).is_file() {
        println!("File does not exist.");
        return Ok(());
    }

    let file_size = metadata(file_path)?.len();
    let mut payload = Vec::new();
    payload.extend_from_slice(&file_size.to_be_bytes());

    let mut file = File::open(file_path)?;
    file.read_to_end(&mut payload)?;

    println!("Uploading file: {}", file_path);

    let response = send_command("UPLOAD", payload)?;

    if response == "SUCCESS" {
        println!("File '{}' uploaded successfully.", file_path);
    } else {
        println!("Failed to upload file '{}'.", file_path);
    }

    Ok(())
}

fn search(term: &str) -> Result<(), Box<dyn Error>> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&(term.len() as u64).to_be_bytes());
    payload.extend_from_slice(term.as_bytes());

    println!("Searching for term: {}", term);

    let response = send_command("SEARCH", payload)?;

    if !response.contains("SUCCESS") {
        println!("Term '{}' not found", term);
        return Ok(());
    }

    let documents = &response[MAX_STATUS_SIZE..];
    if documents.is_empty() {
        println!("No documents found containing '{}'", term);
        return Ok(());
    }

    println!("Documents containing '{term}': {documents}");

    Ok(())
}

fn delete(document_id: u64) -> Result<(), Box<dyn Error>> {
    println!("Deleting document ID: {}", document_id);

    let mut payload = Vec::new();
    payload.extend_from_slice(&document_id.to_be_bytes());

    let response = send_command("DELETE", payload)?;

    if response == "DELETED" {
        println!("Document '{}' deleted successfully.", document_id);
    } else {
        println!(
            "Document '{}' not found or could not be deleted.",
            document_id
        );
    }

    Ok(())
}

fn download(document_id: u64) -> Result<(), Box<dyn Error>> {
    println!("Downloading document ID: {}", document_id);

    let mut payload = Vec::new();
    payload.extend_from_slice(&document_id.to_be_bytes());

    let response = match send_command_and_download_bytes("IMPORT", payload) {
        Ok(response) => response,
        Err(_) => {
            println!("Document '{}' not found.", document_id);
            return Ok(());
        }
    };

    if !response.starts_with(b"SUCCESS") {
        println!("Document '{}' not found.", document_id);
        return Ok(());
    }

    let file_content = &response[MAX_STATUS_SIZE..];

    let file_name = format!("document_{}.txt", document_id);

    let mut file = BufWriter::new(File::create(file_name)?);

    file.write_all(file_content)?;

    println!("Document '{}' downloaded successfully.", document_id);

    Ok(())
}

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Upload {
        #[arg(short, long, help = "Path to the file to upload")]
        file_path: String,
    },
    Search {
        #[arg(short, long, help = "Term to search for")]
        term: String,
    },
    Delete {
        #[arg(short, long, help = "ID of the document to delete")]
        document_id: u64,
    },
    Download {
        #[arg(short, long, help = "ID of the document to download")]
        document_id: u64,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Upload { file_path } => upload(&file_path)?,
        Commands::Search { term } => search(&term)?,
        Commands::Delete { document_id } => delete(document_id)?,
        Commands::Download { document_id } => download(document_id)?,
    }

    Ok(())
}
