use std::io::{self, BufReader, BufWriter, Read, Result as IoResult, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use crate::http::{Request, Response, parse};
use crate::utils::worker::{TaskPool,Job};
use crate::utils::{Connection, Message, Queue};

#[derive(Clone)]
pub struct ServerConfig {
    pub address: String,
}

pub struct Server {
    listener: TcpListener,
    messages: Arc<Queue<Message>>,
    shutdown_flag: Arc<AtomicBool>,
}

impl Server {
    /// Creates a new `Server` instance.
    /// Using This method will instantly start the server.
    /// Server Will Listen for connections until its dropped.
    pub fn new(config: ServerConfig) -> Self {
        let listener = TcpListener::bind(&config.address).expect("Failed to bind to address");
        let messages = Queue::with_capacity(8);
        let shutdown_flag = Arc::new(AtomicBool::new(false));

        let server = Self { 
            listener, 
            messages: Arc::new(messages),
            shutdown_flag,
        };
        server.start();
        server
    }
    /// Trigger the shutdown flag to stop the server.
    /// This method will unblock the thread that is waiting for a message.
    /// It will also stop the acceptor thread.
    /// This method should be called when the server is shutting down.
    /// Its Called when the server is dropped.
    pub fn shutdown(&self) {
        self.messages.unblock();
        self.shutdown_flag.store(true, Ordering::SeqCst);
    }

    /// Starts Acceptor thread.
    /// This thread will accept incoming connections and push them to the queue.
    /// The thread will run until the server is shutdown.
    fn start(&self) {
        let inside_closer = self.shutdown_flag.clone();
        let inside_queue = self.messages.clone();
        let server = self.listener.try_clone().unwrap();

        // Start the Acceptor thread
        thread::spawn(move || {
            let tasks = TaskPool::new();
            log::debug!("Running Acceptor");

            while !inside_closer.load(Ordering::SeqCst) {
                match server.accept() {
                    Ok((stream, _)) => {
                        let inside_queue = inside_queue.clone();
                        tasks.add_task(Job::Task(Box::new(move||{
                            let mut buf_reader = BufReader::with_capacity(4096, &stream);
                            let mut buffer = [0u8; 4096];

                            loop {
                                buffer.fill(0);
                                match buf_reader.read(&mut buffer) {
                                    Ok(0) => {
                                        stream.shutdown(std::net::Shutdown::Both).unwrap_or(());
                                        break;
                                    }
                                    Ok(n) => {
                                        if let Ok(request) = parse(&buffer[..n]) {
                                            inside_queue.push(Message::Request(Connection { 
                                                request, 
                                                stream: stream.try_clone().unwrap() 
                                            }));
                                        }
                                    }
                                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                                        log::debug!("Connection timed out");
                                        stream.shutdown(std::net::Shutdown::Both).unwrap_or(());
                                        break;
                                    }
                                    Err(ref e) if e.kind() == io::ErrorKind::ConnectionReset => {
                                        log::warn!("Connection reset by peer");
                                        stream.shutdown(std::net::Shutdown::Both).unwrap_or(());
                                        break;
                                    }
                                    Err(e) => {
                                        log::error!("Error reading stream: {}", e);
                                        break;
                                    }
                                }
                            }
                            
                        })));
                    }
                    Err(e) => {
                        log::error!("Error accepting connection: {}", e);
                        continue;
                    }
                }
            }
            log::debug!("Acceptor thread shutting down");
        });
    }

    /// Blocks until a message is available to receive.
    /// If the queue is empty, it will wait until a message is available.
    /// If the queue is unblocked, it will return an error.
    pub fn recv(&self) -> IoResult<Connection>{
        match self.messages.pop() {
            Some(Message::Error(e)) => Err(e),
            Some(Message::Request(c)) => Ok(c),
            None => Err(io::Error::new(io::ErrorKind::Other, "No message available")),
        }
    }
    pub fn address(&self) -> String {
        self.listener.local_addr().unwrap().to_string()
    }
    /// Unblocks the thread that is waiting for a message.
    /// this medhod allows graceful shutdown of the server.
    pub fn unblock(&self) {
        self.messages.unblock();
    }

    /// Returns a IncomingRequests object.
    /// This object can be used to handle&respond to incoming requests.
    pub fn incoming(&self) -> IncomingRequests<'_> {
        IncomingRequests {
            server: self,
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// A struct to encapsulate incoming requests to the server.
pub struct IncomingRequests<'a> {
    server: &'a Server,
}

impl IncomingRequests<'_> {
    /// Iterates over incoming requests and handles them using the provided closure.
    /// The closure should take a `HttpRequest` and return a `HttpResponse`.
    /// This method will block until a request is available.
    /// It will also handle the response and write it to the stream.
    pub fn for_each<F>(self, mut handle: F) -> io::Result<()>
    where
        F: FnMut(Request) -> Response + Send + Clone + 'static,
    {
        let server = self.server;
        loop {
            match server.recv() {
                Ok(connection) => {
                    let request = connection.request.clone();
                    let response = handle(request);
                    let mut writer = BufWriter::new(connection.stream);
                    writer.write_all(response.to_string().as_bytes())?;
                    match writer.flush() {
                        Ok(_) => {},
                        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => {
                            log::debug!("Client disconnected");
                            continue;
                        }
                        Err(e) => {
                            log::error!("Error writing response: {}", e);
                            break;
                        }
                    };
                }
                Err(e) => {
                    log::error!("Error receiving message: {}", e);
                    break;
                }
            }
        }
        Ok(())
    }
}
