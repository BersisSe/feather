use bytes::Bytes;
use http::StatusCode;
#[cfg(feature = "log")]
use log::{debug, info, warn};
use may::net::{TcpListener, TcpStream};
use num_cpus;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
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
    /// Provide the Server Identification header in responses (default: true)
    pub server_identification: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            max_body_size: 8192,
            read_timeout_secs: 30,
            workers: num_cpus::get(),
            stack_size: 64 * 1024,
            server_identification: true,
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
        // Configure may runtime
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

    /// Formats the current UTC time as an HTTP-date string per RFC 7231 §7.1.1.1
    fn http_date_now() -> String {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Days since Unix epoch calendar date (Gregorian, no external crate)
        let days_since_epoch = secs / 86400;
        let time_of_day = secs % 86400;
        let hh = time_of_day / 3600;
        let mm = (time_of_day % 3600) / 60;
        let ss = time_of_day % 60;

        // Weekday: Unix epoch (1970-01-01) was a Thursday (index 3)
        const DAYS: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        let weekday = DAYS[((days_since_epoch + 4) % 7) as usize];

        let mut year = 1970u64;
        let mut remaining = days_since_epoch;
        loop {
            let days_in_year = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) { 366 } else { 365 };
            if remaining < days_in_year { break; }
            remaining -= days_in_year;
            year += 1;
        }
        let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        const MONTH_DAYS: [u64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        const MONTHS: [&str; 12] = ["Jan","Feb","Mar","Apr","May","Jun","Jul","Aug","Sep","Oct","Nov","Dec"];
        let mut month = 0usize;
        for (i, &d) in MONTH_DAYS.iter().enumerate() {
            let days = if i == 1 && leap { 29 } else { d };
            if remaining < days { month = i; break; }
            remaining -= days;
        }
        let day = remaining + 1;

        format!(
            "{}, {:02} {} {} {:02}:{:02}:{:02} GMT",
            weekday, day, MONTHS[month], year, hh, mm, ss
        )
    }

    /// Helper to send basic HTTP errors with proper headers
    fn send_error(stream: &mut TcpStream, status: StatusCode, message: &str) -> io::Result<()> {
        let mut response = Response::default();
        response.set_status(status.as_u16());
        response.send_text(message);

        // Add standard security headers
        response.add_header("X-Content-Type-Options", "nosniff").ok();
        response.add_header("X-Frame-Options", "DENY").ok();

        // Date header (RFC 7231 §7.1.1.2)
        response.add_header("Date", &Self::http_date_now()).ok();

        // Always close connection on error
        response.add_header("Connection", "close").ok();

        stream.write_all(&response.to_raw())
    }

    /// The main coroutine function: reads, dispatches, and manages stream lifecycle.
    fn conn_handler(mut stream: TcpStream, service: ArcService, config: ServerConfig) -> io::Result<()> {
        let mut keep_alive = true;
        let mut pipeline_buffer: Vec<u8> = Vec::new();
        let remote_addr = stream.peer_addr()?;
        while keep_alive {
            stream.set_read_timeout(Some(std::time::Duration::from_secs(config.read_timeout_secs)))?;

            let body = pipeline_buffer;
            pipeline_buffer = Vec::new();
            // * 1. READ HEADERS
            let mut buffer = body;
            let mut temp = [0u8; 4096];

            loop {
                let prev_len = buffer.len();
                let n = stream.read(&mut temp)?;
                if n == 0 {
                    return Ok(()); // client closed connection, return Ok().
                }

                buffer.extend_from_slice(&temp[..n]);

                // Check for boundary, starting from up to 3 bytes before new data
                let check_from = prev_len.saturating_sub(3);
                if buffer[check_from..].windows(4).any(|w| w == b"\r\n\r\n") {
                    break;
                }

                if buffer.len() > config.max_body_size {
                    Self::send_error(&mut stream, StatusCode::PAYLOAD_TOO_LARGE, "Headers too large")?;
                    return Ok(());
                }
            }

            let header_end = buffer.windows(4).position(|w| w == b"\r\n\r\n").unwrap() + 4;

            let headers_raw = &buffer[..header_end];
            let mut body = buffer[header_end..].to_vec();

            // * 2. PARSE HEADERS ONLY
            let temp_request = match Request::parse(headers_raw, Bytes::new(), remote_addr) {
                Ok(r) => r,
                Err(e) => {
                    Self::send_error(&mut stream, StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e))?;
                    return Ok(());
                }
            };
            // * 2.5 ENFORCE HTTP/1.1 HOST HEADER
            if temp_request.version == http::Version::HTTP_11 && temp_request.headers.get(http::header::HOST).is_none() {
                Self::send_error(&mut stream, StatusCode::BAD_REQUEST, "Missing Host header")?;
                return Ok(());
            }
            // * 3. REJECT CHUNKED ENCODING
            if temp_request.headers.get(http::header::TRANSFER_ENCODING).map(|v| v.as_bytes().eq_ignore_ascii_case(b"chunked")).unwrap_or(false) {
                Self::send_error(&mut stream, StatusCode::NOT_IMPLEMENTED, "Chunked transfer encoding not supported")?;
                return Ok(());
            }

            //* 4. HANDLE CONNECTION HEADER
            keep_alive = match (temp_request.version, temp_request.headers.get(http::header::CONNECTION)) {
                (http::Version::HTTP_11, Some(v)) if v.as_bytes().eq_ignore_ascii_case(b"close") => false,
                (http::Version::HTTP_11, _) => true,
                _ => false,
            };

            //* 5. READ BODY (Content-Length)
            let content_length = temp_request.headers.get(http::header::CONTENT_LENGTH)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(0);

            // Reject oversized bodies BEFORE telling the client to proceed (RFC 7231 §5.1.1)
            if content_length > config.max_body_size {
                Self::send_error(&mut stream, StatusCode::PAYLOAD_TOO_LARGE, "Request body too large")?;
                return Ok(());
            }

            // * 5.5 HANDLE 100-CONTINUE (RFC 7231 §5.1.1)
            if content_length > 0
                && temp_request.headers.get(http::header::EXPECT)
                    .map(|v| v.as_bytes().eq_ignore_ascii_case(b"100-continue"))
                    .unwrap_or(false)
            {
                stream.write_all(b"HTTP/1.1 100 Continue\r\n\r\n")?;
                stream.flush()?;
            }

            if body.len() >= content_length {
                if body.len() > content_length {
                    pipeline_buffer = body.split_off(content_length);
                }
            } else {
                // Case (c): read until we have enough
                while body.len() < content_length {
                    let n = stream.read(&mut temp)?;
                    if n == 0 {
                        Self::send_error(&mut stream, StatusCode::BAD_REQUEST, "Unexpected EOF while reading request body")?;
                        return Ok(());
                    }
                    body.extend_from_slice(&temp[..n]);
                }
                // The final read may have overshot — save the excess
                if body.len() > content_length {
                    pipeline_buffer = body.split_off(content_length);
                }
            }

            // * 6. BUILD FINAL REQUEST
            let request = match Request::parse(headers_raw, Bytes::from(body), remote_addr) {
                Ok(r) => r,
                Err(e) => {
                    Self::send_error(&mut stream, StatusCode::BAD_REQUEST, &format!("Invalid request: {}", e))?;
                    return Ok(());
                }
            };

            //* 7. DISPATCH RESPONSE
            let result = service.handle(request, None);

            match result {
                Ok(ServiceResult::Response(mut response)) => {
                    // Inject Date header if the service didn't set one (RFC 7231 §7.1.1.2)
                    if response.headers.get(http::header::DATE).is_none() {
                        response.add_header("Date", &Self::http_date_now()).ok();
                        if config.server_identification {
                            response.add_header("Server", "feather-runtime").ok();
                        }
                    }
                    let raw = response.to_raw();
                    stream.write_all(&raw)?;
                    stream.flush()?;
                    if !keep_alive {
                        return Ok(());
                    }
                    if let Some(conn) = response.headers.get(http::header::CONNECTION) {
                        if conn.as_bytes().eq_ignore_ascii_case(b"close") {
                            return Ok(());
                        }
                    }
                }

                Ok(ServiceResult::Consumed) => return Ok(()),

                Err(e) => {
                    Self::send_error(&mut stream, StatusCode::INTERNAL_SERVER_ERROR, &format!("Internal error: {}", e))?;
                    return Ok(());
                }
            }
        }

        Ok(())
    }
}