use std::net::TcpListener;
use std::io::{ self, BufReader, BufWriter, Read, Write };
use std::sync::Arc;
use std::time::Duration;
use rusty_pool::ThreadPool;
use crate::http::{ HttpRequest, HttpResponse };

#[derive(Clone)]
pub struct ServerConfig{
    pub address: String,
    pub core_size: usize,
    pub max_size: usize,
    pub idle_timeout: Duration,
    
}

pub struct Server {
    listener: TcpListener,
    config: ServerConfig,
}

impl Server {
    /// Creates a new `Server` instance.
    pub fn new(config: ServerConfig) -> Self {
        let listener = TcpListener::bind(&config.address).expect("Failed to bind to address");
        
        Server {
            listener,
            config,
        }
    }

    pub fn incoming(self) -> IncomingRequests {
        let config = self.config.clone();
        IncomingRequests { server: Arc::new(self), config }
    }
}

/// A struct to encapsulate incoming requests to the server.
pub struct IncomingRequests{
    server: Arc<Server>,
    config: ServerConfig
}

impl IncomingRequests {
    pub fn for_each<F>(self, handle: F) -> io::Result<()>
        where F: FnMut(HttpRequest) -> HttpResponse + Send + Clone + 'static
    {
        let thread_pool = ThreadPool::new(
            self.config.core_size,
            self.config.max_size,
            self.config.idle_timeout,
        );
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
