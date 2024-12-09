use course_work_parallel_computing::scheduler::Scheduler;
use course_work_parallel_computing::{
    handler::Handler, inverted_index::InvertedIndex, threadpool::ThreadPool, UPLOADS_DIR,
};
use env_logger;
use log::{error, info};
use std::net::TcpListener;
use std::sync::Arc;

const HANDLER_THREAD_POOL_SIZE: usize = 10;
const SCHEDULER_THREAD_POOL_SIZE: usize = 10000;

fn main() {
    env_logger::init();
    let listener = TcpListener::bind("127.0.0.1:7878").expect("Could not bind to address");

    std::fs::create_dir_all(UPLOADS_DIR).expect("Failed to create uploads directory");

    info!("Server listening on 127.0.0.1:7878");

    let inverted_index = Arc::new(InvertedIndex::new());

    let inverted_index_save_handle = Arc::clone(&inverted_index);
    ctrlc::set_handler(move || {
        inverted_index_save_handle.save();
        std::process::exit(0);
    }).expect("Failed to set Ctrl-C handler");

    let handler_thread_pool = ThreadPool::new(HANDLER_THREAD_POOL_SIZE);

    let scheduler = Arc::new(Scheduler::new(
        SCHEDULER_THREAD_POOL_SIZE,
        Arc::clone(&inverted_index),
    ));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                info!("New connection established");

                let mut handler =
                    Handler::new(stream, Arc::clone(&inverted_index), Arc::clone(&scheduler));

                handler_thread_pool.execute(move || handler.handle_client());
            }
            Err(e) => {
                error!("Error accepting connection: {e:#?}");
            }
        }
    }
}
