use bytes::Bytes;
use http::StatusCode;
#[cfg(feature = "log")]
use log::{debug, info, warn};
use may::net::{TcpListener, TcpStream};
use num_cpus;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{panic, sync::Arc};

use crate::http::{Request, Response};
use crate::runtime::service::{ArcService, Service, ServiceResult};

/// Configuration for the HTTP server
#[derive(Clone, Debug)]
pub struct ServerConfig {
    /// Maximum request body size in bytes (default: 8192 = 8KB)
    pub max_body_size: usize,
    /// Read timeout in seconds (default: 30)
    pub read_timeout_secs: u64,
    /// Number of worker threads (default: number of CPU cores)
    pub workers: usize,
    /// Stack size per coroutine in bytes (default: 65536 = 64KB)
    pub stack_size: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            max_body_size: 8192,
            read_timeout_secs: 30,
            workers: num_cpus::get(),
            stack_size: 64 * 1024,
        }
    }
}

/// A HTTP server that handles incoming connections using coroutines
pub struct Server {
    /// The user's application logic
    service: ArcService,
    /// Flag to control server shutdown
    running: Arc<AtomicBool>,
    /// Server configuration
    config: ServerConfig,
}

impl Server {
    /// Create a new Server instance with the given Service
    pub fn new(service: impl Service, max_body_size: usize) -> Self {
        let mut config = ServerConfig::default();
        config.max_body_size = max_body_size;
        Self {
            service: Arc::new(service),
            running: Arc::new(AtomicBool::new(true)),
            config,
        }
    }

    /// Create a new Server instance with custom configuration
    pub fn with_config(service: impl Service, config: ServerConfig) -> Self {
        Self {
            service: Arc::new(service),
            running: Arc::new(AtomicBool::new(true)),
            config,
        }
    }

    /// Initiates a graceful shutdown of the server
    pub fn shutdown(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// Runs the server until shutdown is called
    pub fn run(&self, addr: impl ToSocketAddrs) -> io::Result<()> {
        // Configure coroutine runtime
        may::config().set_workers(self.config.workers);
        may::config().set_stack_size(self.config.stack_size);
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
                    let config = self.config.clone();

                    // Spawn a new coroutine for this connection with panic handling
                    may::go!(move || {
                        let result = panic::catch_unwind(panic::AssertUnwindSafe(|| Self::conn_handler(stream, service, config)));

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
                    #[cfg(feature = "log")]
                    warn!("Failed to accept connection: {}", e);
                }
            }
        }

        #[cfg(feature = "log")]
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
    fn conn_handler(
    mut stream: TcpStream,
    service: ArcService,
    config: ServerConfig,
) -> io::Result<()> {
    let mut keep_alive = true;

    while keep_alive {
        stream.set_read_timeout(Some(std::time::Duration::from_secs(
            config.read_timeout_secs,
        )))?;

        /* =========================
         * 1. READ HEADERS
         * ========================= */
        let mut buffer = Vec::new();
        let mut temp = [0u8; 4096];

        loop {
            let n = stream.read(&mut temp)?;
            if n == 0 {
                return Ok(()); // client closed connection
            }

            buffer.extend_from_slice(&temp[..n]);

            if buffer.windows(4).any(|w| w == b"\r\n\r\n") {
                break;
            }

            if buffer.len() > config.max_body_size {
                Self::send_error(
                    &mut stream,
                    StatusCode::PAYLOAD_TOO_LARGE,
                    "Headers too large",
                )?;
                return Ok(());
            }
        }

        let header_end = buffer
            .windows(4)
            .position(|w| w == b"\r\n\r\n")
            .unwrap()
            + 4;

        let headers_raw = &buffer[..header_end];
        let mut body = buffer[header_end..].to_vec();

        /* =========================
         * 2. PARSE HEADERS ONLY
         * ========================= */
        let temp_request = match Request::parse(headers_raw, Bytes::new()) {
            Ok(r) => r,
            Err(e) => {
                Self::send_error(
                    &mut stream,
                    StatusCode::BAD_REQUEST,
                    &format!("Invalid request: {}", e),
                )?;
                return Ok(());
            }
        };

        /* =========================
         * 3. HANDLE CONNECTION HEADER
         * ========================= */
        keep_alive = match (
            temp_request.version,
            temp_request.headers.get(http::header::CONNECTION),
        ) {
            (http::Version::HTTP_11, Some(v))
                if v.as_bytes().eq_ignore_ascii_case(b"close") =>
            {
                false
            }
            (http::Version::HTTP_11, _) => true,
            _ => false,
        };

        /* =========================
         * 4. REJECT CHUNKED ENCODING
         * ========================= */
        if temp_request
            .headers
            .get(http::header::TRANSFER_ENCODING)
            .map(|v| v.as_bytes().eq_ignore_ascii_case(b"chunked"))
            .unwrap_or(false)
        {
            Self::send_error(
                &mut stream,
                StatusCode::NOT_IMPLEMENTED,
                "Chunked transfer encoding not supported",
            )?;
            return Ok(());
        }

        /* =========================
         * 5. READ BODY (Content-Length)
         * ========================= */
        let content_length = temp_request
            .headers
            .get(http::header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);

        if content_length > config.max_body_size {
            Self::send_error(
                &mut stream,
                StatusCode::PAYLOAD_TOO_LARGE,
                "Request body too large",
            )?;
            return Ok(());
        }

        while body.len() < content_length {
            let n = stream.read(&mut temp)?;
            if n == 0 {
                break;
            }

            body.extend_from_slice(&temp[..n]);

            if body.len() > config.max_body_size {
                Self::send_error(
                    &mut stream,
                    StatusCode::PAYLOAD_TOO_LARGE,
                    "Request body too large",
                )?;
                return Ok(());
            }
        }

        /* =========================
         * 6. BUILD FINAL REQUEST
         * ========================= */
        let request = match Request::parse(headers_raw, Bytes::from(body)) {
            Ok(r) => r,
            Err(e) => {
                Self::send_error(
                    &mut stream,
                    StatusCode::BAD_REQUEST,
                    &format!("Invalid request: {}", e),
                )?;
                return Ok(());
            }
        };

        /* =========================
         * 7. DISPATCH
         * ========================= */
        let result = service.handle(request, None);

        match result {
            Ok(ServiceResult::Response(response)) => {
                let raw = response.to_raw();
                stream.write_all(&raw)?;
                stream.flush()?;

                if let Some(conn) = response.headers.get(http::header::CONNECTION) {
                    if conn.as_bytes().eq_ignore_ascii_case(b"close") {
                        return Ok(());
                    }
                }
            }

            Ok(ServiceResult::Consumed) => return Ok(()),

            Err(e) => {
                Self::send_error(
                    &mut stream,
                    StatusCode::INTERNAL_SERVER_ERROR,
                    &format!("Internal error: {}", e),
                )?;
                return Ok(());
            }
        }
    }

    Ok(())
}

}
