use crate::http::{Request, Response};
use std::io::{self, BufReader, BufWriter, Read, Result as IoResult, Write};
use std::cell::RefCell;

use std::net::ToSocketAddrs;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use may::go;
use may::net::TcpListener;
use may::Config;


/// The Server is the main struct of the Runtime.  
/// It drives the runtime Accepting Requests Answering them etc.
pub struct Server {
    listener: TcpListener,
    shutdown_flag: Arc<AtomicBool>,
}

impl Server {
    /// Creates a new `Engine` instance without a config
    pub fn new(addr: impl ToSocketAddrs) -> Self {
        // Bigger Pools for better Concurency
        Config.set_pool_capacity(200);
        Config.set_stack_size(256 * 1024);
        
        // Advanced socket2 usage for tuning
        let addr = addr.to_socket_addrs().unwrap().next().expect("Invalid address");
       
        let listener: TcpListener = socket.into();
        listener.set_nonblocking(true).expect("Failed to set nonblocking");

        let messages = Queue::with_capacity(256); 
        let shutdown_flag = Arc::new(AtomicBool::new(false));
        let server = Self {
            listener,
            messages: Arc::new(messages),
            shutdown_flag,
        };
        server
    }
    /// Creates a new `Engine` instance with a config
    pub fn with_config(config: EngineConfig) -> Self {
        // Advanced socket2 usage for tuning
        let addr = SocketAddr::from(config.address);
        let domain = match addr {
            SocketAddr::V4(_) => Domain::IPV4,
            SocketAddr::V6(_) => Domain::IPV6,
        };
        let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP)).expect("Failed to create socket");
        socket.set_reuse_address(true).ok();
        #[cfg(unix)]
        socket.set_reuse_port(true).ok();
        socket.set_nodelay(true).ok();
        socket.set_recv_buffer_size(1 << 20).ok();
        socket.set_send_buffer_size(1 << 20).ok();
        let backlog = 2048;
        socket.bind(&addr.into()).expect("Failed to bind socket");
        socket.listen(backlog).expect("Failed to listen on socket");
        let listener: TcpListener = socket.into();
        listener.set_nonblocking(true).expect("Failed to set nonblocking");

        let messages = Queue::with_capacity(500);
        let shutdown_flag = Arc::new(AtomicBool::new(false));
        let server = Self {
            listener,
            messages: Arc::new(messages),
            shutdown_flag,
        };
        server
    }
    /// Add a new task to the internal TaskPool works like `thread::spawn` but its managed bythe Engine
    pub fn spawn(&self, task: impl FnOnce() + Send + 'static) {
        go!(task);
    }
    /// Trigger the shutdown flag to stop the Engine.
    /// This method will unblock the thread that is waiting for a message.
    /// It will also stop the acceptor thread.
    /// This method should be called when the Engine is shutting down.
    /// Its Called when the Engine is dropped.
    pub fn shutdown(&self) {
        self.messages.unblock();
        self.shutdown_flag.store(true, Ordering::Relaxed);
    }

    /// Starts Acceptor thread.
    /// This thread will accept incoming connections and push them to the queue.
    /// The thread will run until the Engine is shutdown.
    pub fn start(&self) {
        let inside_closer = self.shutdown_flag.clone();
        let inside_queue = self.messages.clone();
        let server = self.listener.try_clone().unwrap();
        // Start the Acceptor thread
        go!(move || {
            #[cfg(feature = "log")]
            log::debug!("Acceptor thread started");

            use std::thread::sleep;
            use std::time::Duration;

            //* Thread-local buffer pool for connection handlers
            thread_local! {
                static BUFFER_POOL: RefCell<Vec<u8>> = RefCell::new(vec![0u8; 4096]);
            }
            while !inside_closer.load(Ordering::Relaxed) {
                match server.accept() {
                    Ok((stream, _)) => {
                        // Socket tuning: TCP_NODELAY, buffer size, timeout
                        let _ = stream.set_nodelay(true);
                        let _ = stream.set_read_timeout(Some(Duration::from_secs(25)));
                        let _ = stream.set_write_timeout(Some(Duration::from_secs(25)));
                        let inside_queue = inside_queue.clone();
                        go!(move || {
                            let mut buf_reader = BufReader::with_capacity(4096, stream.try_clone().unwrap());
                            BUFFER_POOL.with(|buf_cell| {
                                let mut buffer = buf_cell.borrow_mut();
                                loop {
                                    buffer.fill(0);
                                    
                                    match buf_reader.read(&mut buffer[..]) {
                                        Ok(0) => {
                                            // Connection closed
                                            break;
                                        }
                                        Ok(n) => {
                                            if let Ok(mut request) = Request::parse(&buffer[..n]) {
                                                
                                                if let Ok(write_socket) = stream.try_clone() {
                                                    request.set_stream(write_socket);
                                                }
                                                inside_queue.push(Message::Request(request))
                                            }
                                        }
                                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                                            #[cfg(feature = "log")]
                                            log::debug!("Connection timed out");
                                            let _ = stream.shutdown(std::net::Shutdown::Both);
                                            break;
                                        }
                                        Err(ref e) if e.kind() == io::ErrorKind::ConnectionReset => {
                                            #[cfg(feature = "log")]
                                            log::debug!("Connection reset by peer");
                                            let _ = stream.shutdown(std::net::Shutdown::Both);
                                            break;
                                        }
                                        Err(e) => {
                                            #[cfg(feature = "log")]
                                            log::debug!("Error reading stream: {}", e);
                                            break;
                                        }
                                    }
                                }
                            });
                        });
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        //? Non-blocking accept: avoid busy-wait 
                        sleep(Duration::from_millis(1));
                        continue;
                    }
                    Err(e) => {
                        #[cfg(feature = "log")]
                        log::debug!("Error accepting connection: {}", e);
                        sleep(Duration::from_millis(1));
                        continue;
                    }
                }
            }
            #[cfg(feature = "log")]
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
    pub fn for_each<F>(self, mut handle: F) -> io::Result<()>
    where
        F: FnMut(&mut Request) -> Response,
    {
        let engine = self;
        
        loop {
            
            match engine.recv() {
                Ok(mut request) => {
                   
                    let is_close = matches!(request.connection, Some(crate::http::ConnectionState::Close));
                    let connection_header = if is_close { "close" } else { "keep-alive" };
                    let mut response = handle(&mut request);
                    response.add_header("connection", connection_header);
                    if let Some(mut stream) = request.take_stream() {
                        let mut writer = BufWriter::new(&mut stream);
                        writer.write_all(&response.to_raw())?;
                        match writer.flush() {
                            Ok(_) => {}
                            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => {
                                #[cfg(feature = "log")]
                                log::debug!("Client disconnected");
                                continue;
                            }
                            Err(e) => {
                                #[cfg(feature = "log")]
                                log::debug!("Error writing response: {}", e);
                                break;
                            }
                        };
                        if is_close {
                            break;
                        }
                    }
                }
                Err(e) => {
                    #[cfg(feature = "log")]
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
