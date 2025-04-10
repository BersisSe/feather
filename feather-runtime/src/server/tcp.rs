use std::net::TcpListener;
use std::io::{ self, BufReader, BufWriter, Read, Write };
use std::sync::Arc;
use std::time::Duration;
use rusty_pool::ThreadPool;
use crate::http::{ HttpRequest, HttpResponse };

pub struct Server {
    listener: TcpListener,
    num_threads: usize,
}

impl Server {
    /// Creates a new `Server` instance.
    pub fn new(address: String, num_threads: usize) -> Self {
        let listener = TcpListener::bind(address).expect("Failed to bind to address");

        Self { listener, num_threads }
    }

    pub fn incoming(self) -> IncomingRequests {
        let num_threads = self.num_threads;
        IncomingRequests { server: Arc::new(self), num_threads }
    }
}

/// A struct to encapsulate incoming requests to the server.
pub struct IncomingRequests {
    server: Arc<Server>,
    num_threads: usize,
}

impl IncomingRequests {
    pub fn for_each<F>(self, handle: F) -> io::Result<()>
        where F: FnMut(HttpRequest) -> HttpResponse + Send + Clone + 'static
    {
        let thread_pool = ThreadPool::new(2, self.num_threads, Duration::from_millis(100));
        let listener = self.server.listener.try_clone()?; // Keep the listener in blocking mode
        loop{
            match listener.accept() {
                Ok((stream, _)) => {
                    stream.set_nodelay(true).unwrap(); // Disable Nagle's algorithm
                    let mut handle = handle.clone();
                    

                    thread_pool.execute(
                        Box::new(move || {
                            let mut buf_reader = BufReader::with_capacity(4096, &stream);
                            let mut buf_writer = BufWriter::with_capacity(4096, &stream);
                            let mut buffer = [0u8; 4096];

                            loop {
                                buffer.fill(0);
                                match buf_reader.read(&mut buffer) {
                                    Ok(0) => {
                                        break;
                                    } // Connection closed
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
                                    Err(_) => {
                                        break;
                                    }
                                }
                            }
                        })
                    );
                }
                
                Err(_) => {
                    // Handle other errors (e.g., connection errors)
                    continue;
                }
            }
        }
           
        
    }
}
