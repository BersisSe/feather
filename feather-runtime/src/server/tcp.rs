use std::net::TcpListener;
use std::io::{Read, Write, BufReader, BufWriter};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use rusty_pool::ThreadPool;
use crate::http::{HttpRequest, HttpResponse};


pub struct Server {
    listener: TcpListener,
    num_threads: usize,
    shutdown_signal : Arc<AtomicBool>
}

impl Server {
    /// Creates a new `Server` instance.
    pub fn new(address: String, num_threads: usize) -> Self {
        let listener = TcpListener::bind(address).expect("Failed to bind to address");
        let shutdown_signal = Arc::new(AtomicBool::new(false)); // Initialize the shutdown signal
        Self { listener, num_threads, shutdown_signal}
    }
    
    


    pub fn incoming(self) -> IncomingRequests {
        let num_threads = self.num_threads;
        let shutdown_signal = self.shutdown_signal.clone(); // Clone the shutdown signal
        IncomingRequests { server: Arc::new(self), num_threads, shutdown_signal}
    }
}

/// A struct to encapsulate incoming requests to the server.
pub struct IncomingRequests {
    server: Arc<Server>,
    num_threads: usize,
    shutdown_signal : Arc<AtomicBool>
}

impl IncomingRequests {
    pub fn for_each<F>(&self, handle: F)
    where F: FnMut(HttpRequest) -> HttpResponse + Send + Clone + 'static 
    {
        let thread_pool = ThreadPool::new(2, self.num_threads, Duration::from_millis(100));
    
        for stream in self.server.listener.incoming() {
            if self.shutdown_signal.load(Ordering::SeqCst){
                break;
                
            }
            
            let stream = stream.unwrap();
            stream.set_nodelay(true).unwrap(); // Disable Nagle's algorithm
            
            let mut handle = handle.clone();
            thread_pool.execute(Box::new(move || {
                let mut buf_reader = BufReader::with_capacity(4096, &stream);
                let mut buf_writer = BufWriter::with_capacity(4096, &stream);
                let mut buffer = [0u8; 4096];

                loop {
                    buffer.fill(0);
                    match buf_reader.read(&mut buffer) {
                        Ok(0) => break, // Connection closed
                        Ok(n) => {
                            if let Ok(req) = HttpRequest::parse(&buffer[..n]) {
                                // Write response directly
                                let response = handle(req);
                                if buf_writer.write_all(&response.to_bytes()).is_err() {
                                    break;
                                }
                                buf_writer.flush().unwrap_or(());
                            }
                        }
                        Err(_) => break,
                    }
                }
            }));
        }
    }
    /// Signals the server to shut down gracefully.
    pub fn shutdown(&self){
        self.shutdown_signal.store(true, Ordering::SeqCst);
    }
}