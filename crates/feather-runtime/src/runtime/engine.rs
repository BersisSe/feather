use crate::http::{Request, Response};
use crate::utils::worker::{Job, PoolConfig, TaskPool};
use crate::utils::{Message, Queue};
use std::io::{self, BufReader, BufWriter, Read, Result as IoResult, Write};
use std::net::{Ipv4Addr, TcpListener, ToSocketAddrs};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

pub struct EngineConfig {
    /// The Address of the Engine(IP,Port)
    pub address: (Ipv4Addr, u16),
    pub worker_config: PoolConfig,
}

/// The Engine is the main struct of the Runtime.  
/// It drives the runtime Accepting Requests Answering them etc.
pub struct Engine {
    listener: TcpListener,
    messages: Arc<Queue<Message>>,
    shutdown_flag: Arc<AtomicBool>,
    tasks: Arc<TaskPool>,
}

impl Engine {
    /// Creates a new `Engine` instance without a config
    pub fn new(addr: impl ToSocketAddrs) -> Self {
        let listener = TcpListener::bind(addr).expect("Failed to bind to address");
        let messages = Queue::with_capacity(16);
        let shutdown_flag = Arc::new(AtomicBool::new(false));
        let tasks = TaskPool::new();
        let server = Self {
            listener,
            messages: Arc::new(messages),
            shutdown_flag,
            tasks: Arc::new(tasks),
        };
        server
    }
    /// Creates a new `Engine` instance with a config
    pub fn with_config(config: EngineConfig) -> Self {
        let listener = TcpListener::bind(config.address).expect("Failed to bind to address");
        let messages = Queue::with_capacity(16);
        let shutdown_flag = Arc::new(AtomicBool::new(false));
        let tasks = TaskPool::with_config(config.worker_config);
        let server = Self {
            listener,
            messages: Arc::new(messages),
            shutdown_flag,
            tasks: Arc::new(tasks),
        };
        server
    }
    /// Add a new task to the internal TaskPool works like `thread::spawn` but its managed the Engine
    pub fn spawn(&self, task: impl Into<Job>) {
        self.tasks.add_task(task.into());
    }
    /// Trigger the shutdown flag to stop the Engine.
    /// This method will unblock the thread that is waiting for a message.
    /// It will also stop the acceptor thread.
    /// This method should be called when the Engine is shutting down.
    /// Its Called when the Engine is dropped.
    pub fn shutdown(&self) {
        self.messages.unblock();
        self.shutdown_flag.store(true, Ordering::SeqCst);
    }

    /// Starts Acceptor thread.
    /// This thread will accept incoming connections and push them to the queue.
    /// The thread will run until the Engine is shutdown.
    pub fn start(&self) {
        let inside_closer = self.shutdown_flag.clone();
        let inside_queue = self.messages.clone();
        let server = self.listener.try_clone().unwrap();
        let tasks = self.tasks.clone();
        // Start the Acceptor thread
        thread::spawn(move || {
            let tasks = tasks;
            log::debug!("Running Acceptor");

            while !inside_closer.load(Ordering::SeqCst) {
                match server.accept() {
                    Ok((stream, _)) => {
                        let inside_queue = inside_queue.clone();
                        tasks.add_task(Job::Task(Box::new(move || {
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
                                        if let Ok(mut request) = Request::parse(&buffer[..n]) {
                                            request.set_stream(stream.try_clone().unwrap());
                                            inside_queue.push(Message::Request(request))
                                        }
                                    }
                                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                                        log::debug!("Connection timed out");
                                        stream.shutdown(std::net::Shutdown::Both).unwrap_or(());
                                        break;
                                    }
                                    Err(ref e) if e.kind() == io::ErrorKind::ConnectionReset => {
                                        log::debug!("Connection reset by peer");
                                        stream.shutdown(std::net::Shutdown::Both).unwrap_or(());
                                        break;
                                    }
                                    Err(e) => {
                                        log::debug!("Error reading stream: {}", e);
                                        break;
                                    }
                                }
                            }
                        })));
                    }
                    Err(e) => {
                        log::debug!("Error accepting connection: {}", e);
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
    pub fn recv(&self) -> IoResult<Request> {
        match self.messages.pop() {
            Some(Message::Error(e)) => Err(e),
            Some(Message::Request(r)) => Ok(r),
            None => Err(io::Error::new(io::ErrorKind::Other, "No message available")),
        }
    }
    /// Returns the address the Engine is Bound to.  
    pub fn address(&self) -> String {
        self.listener.local_addr().unwrap().to_string()
    }
    /// Unblocks the thread that is waiting for a message.
    /// this medhod allows graceful shutdown of the Engine's Runtime.
    pub fn unblock(&self) {
        self.messages.unblock();
    }

    /// Iterates over incoming requests and handles them using the provided closure.
    /// The closure should take a `HttpRequest` and return a `HttpResponse`.
    /// This method will block until a request is available.
    /// It will also handle the response and write it to the stream.
    pub fn for_each<F>(&self, mut handle: F) -> io::Result<()>
    where
        F: FnMut(Request) -> (Request, Response),
    {
        let engine = self;
        loop {
            match engine.recv() {
                Ok(request) => {
                    let connect = &request.connection.as_deref().unwrap_or("keep-alive").to_lowercase();
                    let (request, mut response) = handle(request);
                    response.add_header("connection", &connect);
                    if let Some(mut stream) = request.take_stream() {
                        let mut writer = BufWriter::new(&mut stream);
                        writer.write_all(&response.to_raw())?;
                        match writer.flush() {
                            Ok(_) => {}
                            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => {
                                log::debug!("Client disconnected");
                                continue;
                            }
                            Err(e) => {
                                log::debug!("Error writing response: {}", e);
                                break;
                            }
                        };
                        if connect == "close" {
                            break;
                        }
                    }
                }
                Err(e) => {
                    log::debug!("Error receiving message: {}", e);
                    break;
                }
            }
        }
        Ok(())
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.shutdown();
    }
}
