use course_work_parallel_computing::{
    handler::Handler, inverted_index::InvertedIndex, threadpool::ThreadPool, UPLOADS_DIR,
};
use env_logger;
use log::{error, info};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

fn main() {
    env_logger::init();
    let listener = TcpListener::bind("127.0.0.1:7878").expect("Could not bind to address");

    std::fs::create_dir_all(UPLOADS_DIR).expect("Failed to create uploads directory");

    info!("Server listening on 127.0.0.1:7878");

    let cpu_count = num_cpus::get();
    let thread_pool = ThreadPool::new(cpu_count);

    let inverted_index = Arc::new(Mutex::new(InvertedIndex::new()));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                info!("New connection established");

                let mut handler = Handler::new(stream, Arc::clone(&inverted_index));

                thread_pool.execute(move || handler.handle_client());
            }
            Err(e) => {
                error!("Error accepting connection: {}", e);
            }
        }
    }
}
