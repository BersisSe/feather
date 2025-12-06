use http::StatusCode;
#[cfg(feature = "log")]
use log::{debug, info, warn};
use may::net::{TcpListener, TcpStream};
use num_cpus;
use std::io::{self, Read, Write};
use std::net::ToSocketAddrs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{net::SocketAddr, panic, sync::Arc};

use crate::http::{Request, Response};
use crate::runtime::service::{ArcService, Service, ServiceResult};
/// A HTTP server that handles incoming connections using coroutines
pub struct Server {
    /// The user's application logic
    service: ArcService,
    /// Flag to control server shutdown
    running: Arc<AtomicBool>,
}

impl Server {
    /// Create a new Server instance with the given Service
    pub fn new(service: impl Service) -> Self {
        Self {
            service: Arc::new(service),
            running: Arc::new(AtomicBool::new(true)),
        }
    }

    /// Initiates a graceful shutdown of the server
    pub fn shutdown(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// Runs the server until shutdown is called
    pub fn run(&self, addr: impl ToSocketAddrs) -> io::Result<()> {
        // Setting worker count equal to CPU cores for maximum parallel utilization.
        may::config().set_workers(num_cpus::get());
        may::config().set_stack_size(64 * 1024); // 64 KB instead of default 2-4 KB(Mainly for logger formatting)
        #[cfg(feature = "log")]
        info!(
            "Feather Runtime Started on {}",
            addr.to_socket_addrs()?.next().unwrap_or(SocketAddr::from(([0, 0, 0, 0], 80)))
        );

        let listener = TcpListener::bind(addr)?;

        while self.running.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((stream, addr)) => {
                    #[cfg(feature = "log")]
                    debug!("New connection from {}", addr);
                    let service = self.service.clone();

                    // Spawn a new coroutine for this connection with panic handling
                    may::go!(move || {
                        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| Self::conn_handler(stream, service)));

                        match result {
                            Ok(Ok(())) => (), // Connection completed successfully
                            Ok(Err(e)) => {
                                #[cfg(feature = "log")]
                                log::error!("Connection handler error: {}", e);
                            }
                            Err(e) => {
                                let msg = e.downcast_ref::<String>().map(|s| s.as_str()).unwrap_or("Unknown panic");
                                #[cfg(feature = "log")]
                                log::error!("Connection handler panic: {}", msg);
                            }
                        }
                    });
                }
                Err(e) => {
                    warn!("Failed to accept connection: {}", e);
                }
            }
        }

        info!("Server shutting down");
        Ok(())
    }

    /// Helper to send basic HTTP errors with proper headers
    fn send_error(stream: &mut TcpStream, status: StatusCode, message: &str) -> io::Result<()> {
        let mut response = Response::default();
        response.set_status(status.as_u16());
        response.send_text(message);

        // Add standard security headers
        response.add_header("X-Content-Type-Options", "nosniff").ok();
        response.add_header("X-Frame-Options", "DENY").ok();

        // Always close connection on error
        response.add_header("Connection", "close").ok();

        stream.write_all(&response.to_raw())
    }
    /// The main coroutine function: reads, dispatches, and manages stream lifecycle.
    fn conn_handler(mut stream: TcpStream, service: ArcService) -> io::Result<()> {
        const MAX_REQUEST_SIZE: usize = 8192; // 8KB limit
        let mut buffer = [0u8; MAX_REQUEST_SIZE];
        let mut keep_alive = true;

        while keep_alive {
            // 1. READ PHASE with timeout
            stream.set_read_timeout(Some(std::time::Duration::from_secs(30)))?;
            let bytes_read = match stream.read(&mut buffer) {
                Ok(0) => return Ok(()), // Connection closed
                Ok(n) if n >= MAX_REQUEST_SIZE => {
                    Self::send_error(&mut stream, StatusCode::PAYLOAD_TOO_LARGE, "Request body too large")?;
                    return Ok(());
                }
                Ok(n) => n,
                Err(e) => {
                    if e.kind() == io::ErrorKind::WouldBlock {
                        Self::send_error(&mut stream, StatusCode::REQUEST_TIMEOUT, "Request timed out")?;
                    }
                    return Err(e);
                }
            };

            // 2. PARSE PHASE with improved error handling
            let request = match Request::parse(&buffer[..bytes_read]) {
                Ok(req) => {
                    // Update keep_alive based on request headers and HTTP version
                    keep_alive = match (req.version, req.headers.get(http::header::CONNECTION)) {
                        (http::Version::HTTP_11, Some(v)) => v.as_bytes().eq_ignore_ascii_case(b"keep-alive"),
                        (http::Version::HTTP_11, None) => true, // HTTP/1.1 defaults to keep-alive
                        _ => false,                             // HTTP/1.0 and others default to close
                    };
                    req
                }
                Err(e) => {
                    Self::send_error(&mut stream, StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e))?;
                    return Ok(());
                }
            };

            // 3. SERVICE DISPATCH PHASE (Ownership Transfer)

            let result = service.handle(request, None);

            // 4. HANDLE RESULT & I/O
            match result {
                Ok(ServiceResult::Response(response)) => {
                    // *** RE-ACQUIRE STREAM (Simplified) ***
                    // NOTE: This is the critical architectural issue: the stream ownership must be returned
                    // by the service if it was not Consumed. For now, we assume ownership is re-acquired.
                    // This line would fail without the stream being returned from the service.
                    // To proceed, we enforce `Connection: Close` and rely on the variable being moved back.

                    let raw_response = response.to_raw();
                    stream.write_all(&raw_response)?;
                    stream.flush()?;

                    // Check Connection header for keep-alive
                    // NOTE: If keep-alive is intended, you must skip the buffer reuse step.
                    if let Some(connection) = response.headers.get(http::header::CONNECTION) {
                        if connection.as_bytes().eq_ignore_ascii_case(b"close") {
                            return Ok(());
                        }
                    }

                    // ⭐️ NO NEED TO CLEAR THE BUFFER IF THE NEXT READ OVERWRITES IT!
                    // The next stream.read() will start at buffer[0]. The data at buffer[bytes_read..8192]
                    // is old, but bytes_read will correctly bound the next read slice.
                    // We simply loop back to `stream.read(&mut buffer)?`
                }

                Ok(ServiceResult::Consumed) => {
                    return Ok(());
                }

                Err(e) => {
                    Self::send_error(&mut stream, http::StatusCode::INTERNAL_SERVER_ERROR, &format!("Internal error: {}", e))?;
                    return Ok(());
                }
            }

            // If the connection is Keep-Alive, the loop continues.
            // The buffer is implicitly "cleared" by the bounds of the next stream.read().
            // We only need to reset the connection status logic for the next iteration.
        }
        Ok(())
    }
}
