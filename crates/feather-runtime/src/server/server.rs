use tungstenite::{accept, WebSocket};
use crate::http::{Request, Response, parse};
use crate::utils::worker::{Job, TaskPool};
use crate::utils::{Connection, Message, Queue};
use std::io::{self, BufReader, BufWriter, Read, Result as IoResult, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use super::websocket;
use parking_lot::Mutex;
pub struct Server {
    messages: Arc<Queue<Message>>,
    shutdown_flag: Arc<AtomicBool>,
    ws_routes: Arc<Mutex<Vec<websocket::WSRoute>>>
}

impl Server {
    /// Creates a new `Server` instance.  
    /// Server Will need to started via `start` method.  
    /// Server Will Listen for connections until its dropped.  
    pub fn new() -> Self {
        let messages = Queue::with_capacity(8);
        let shutdown_flag = Arc::new(AtomicBool::new(false));

        let server = Self {
            messages: Arc::new(messages),
            shutdown_flag,
            ws_routes: Arc::new(Mutex::new(Vec::new()))
        };

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
    pub fn start(&self, addr:impl ToSocketAddrs) {
        let inside_closer = self.shutdown_flag.clone();
        let inside_queue = self.messages.clone();
        let ws_routes = self.ws_routes.clone();
        let server = TcpListener::bind(addr).expect("Failed to bind to address");

        // Start the Acceptor thread
        thread::spawn(move || {
            let tasks = TaskPool::new();
            log::debug!("Running Acceptor");

            while !inside_closer.load(Ordering::SeqCst) {
                match server.accept() {
                    Ok((stream, _)) => {
                        let inside_queue = inside_queue.clone();
                        let ws_routes = ws_routes.clone();
                        tasks.add_task(Job::Task(Box::new( move || {
                            // Peek at the first 1024 bytes to check for WebSocket upgrade
                            let mut peek_buf = [0u8; 1024];
                            let n = match stream.peek(&mut peek_buf) {
                                Ok(n) => n,
                                Err(e) => {
                                    log::error!("Error peeking stream: {}", e);
                                    return;
                                }
                            };
                            let req_str = String::from_utf8_lossy(&peek_buf[..n]);
                            let is_ws = ws_routes.lock().iter().any(|route| {
                                req_str.contains("Upgrade: websocket") && req_str.contains(route.path)
                            });
                            if is_ws {
                                // Let tungstenite handle the handshake
                                for route in ws_routes.lock().iter_mut() {
                                    if req_str.contains("Upgrade: websocket") && req_str.contains(route.path) {
                                        match accept(stream) {
                                            Ok(ws) => {
                                                log::debug!("WebSocket connection accepted for route: {}", route.path);
                                                (route.handler)(ws);
                                            }
                                            Err(e) => {
                                                log::error!("WebSocket handshake failed: {}", e);
                                            }
                                        }
                                        return;
                                    }
                                }
                            }
                            // Not a WebSocket, proceed with normal HTTP parsing
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
                                                stream: stream.try_clone().unwrap(),
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
    /// Starts receiving messages from the server but only accepts the selected route as a WebSocket  
    /// After the WebSocket Handshake succeeds it just returns the WebSocket Object to the caller
    pub fn attach_websocket<F>(&mut self, path: &'static str,handler: F)
    where F: FnMut(WebSocket<TcpStream>) + 'static + Send + Sync
    {
        self.ws_routes.lock().push(websocket::WSRoute { 
            path, 
            handler: Box::new(handler) 
        });
    }

    /// Blocks until a message is available to receive.
    /// If the queue is empty, it will wait until a message is available.
    /// If the queue is unblocked, it will return an error.
    pub fn recv(&self) -> IoResult<Connection> {
        match self.messages.pop() {
            Some(Message::Error(e)) => Err(e),
            Some(Message::Request(c)) => Ok(c),
            None => Err(io::Error::new(io::ErrorKind::Other, "No message available")),
        }
    }
    
    /// Unblocks the thread that is waiting for a message.
    /// this medhod allows graceful shutdown of the server.
    pub fn unblock(&self) {
        self.messages.unblock();
    }

    /// Returns a IncomingRequests object.
    /// This object can be used to handle&respond to incoming requests.
    pub fn incoming(&self) -> IncomingRequests<'_> {
        IncomingRequests { server: self }
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
        F: FnMut(Request) -> Response,
    {
        let server = self.server;
        loop {
            match server.recv() {
                Ok(connection) => {
                    let request = connection.request.clone();
                    let connect = request
                        .connection
                        .as_deref()
                        .unwrap_or("keep-alive")
                        .to_lowercase();
                    let mut response = handle(request);
                    response.add_header("connection", &connect);
                    let mut writer = BufWriter::new(connection.stream);
                    writer.write_all(response.to_raw().as_bytes())?;
                    match writer.flush() {
                        Ok(_) => {}
                        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => {
                            log::debug!("Client disconnected");
                            continue;
                        }
                        Err(e) => {
                            log::error!("Error writing response: {}", e);
                            break;
                        }
                    };
                    if connect == "close" {
                        break;
                    }
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
