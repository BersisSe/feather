//! Built-in middleware for common functionality.
//!
//! This module provides ready-to-use middleware for logging, CORS, and static file serving.

use super::common::Middleware;
use crate::{Outcome, internals::AppContext, next};

use feather_runtime::http::{Request, Response};
#[cfg(feature = "log")]
use log::info;
use std::{
    fs::{self, File},
    io::{self, Read},
    path::Path,
};

/// Logs incoming HTTP requests.
///
/// This middleware logs the HTTP method and path of each request, then passes
/// the request to the next middleware without modification.
///
/// Requires the `log` feature to be enabled.
///
/// # Example
///
/// ```rust,ignore
/// use feather::{App, middlewares::builtins::Logger};
///
/// let mut app = App::new();
/// app.use_middleware(Logger);
/// ```
#[cfg(feature = "log")]
pub struct Logger;

impl Middleware for Logger {
    fn handle(&self, _request: &mut Request, _: &mut Response, _: &AppContext) -> Outcome {
        #[cfg(feature = "log")]
        info!("{} {}", _request.method, _request.uri.path(),);
        next!()
    }
}

#[derive(Default)]
/// Adds CORS (Cross-Origin Resource Sharing) headers to responses.
///
/// This middleware adds the `Access-Control-Allow-Origin` header to all responses,
/// allowing browsers to make cross-origin requests to your API.
///
/// # Example
///
/// ```rust,ignore
/// use feather::{App, middlewares::builtins::Cors};
///
/// let mut app = App::new();
///
/// // Allow all origins
/// app.use_middleware(Cors::default());
///
/// // Allow specific origin
/// app.use_middleware(Cors::new("https://example.com".to_string()));
/// ```
pub struct Cors(Option<String>);

impl Cors {
    /// Create a CORS middleware for a specific origin.
    ///
    /// # Arguments
    ///
    /// * `origin` - The allowed origin (e.g., `<https://example.com>`)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let cors = Cors::new("https://example.com".to_string());
    /// app.use_middleware(cors);
    /// ```
    #[must_use]
    pub const fn new(origin: String) -> Self {
        Self(Some(origin))
    }
}

impl Middleware for Cors {
    fn handle(&self, _: &mut Request, response: &mut Response, _: &AppContext) -> Outcome {
        response.add_header("Access-Control-Allow-Origin", self.0.as_deref().unwrap_or("*"))?;
        next!()
    }
}

/// Serves static files from a directory.
///
/// This middleware serves static files (HTML, CSS, JavaScript, images, etc.) from
/// a specified directory. It automatically detects content types based on file extensions.
/// returns HTTP errors for invalid paths.
/// # Security
///
/// - Path traversal attacks are prevented (.. is not allowed)
/// - Directory listing is disabled
/// - Only files are served, not directories
///
/// # Example
///
/// ```rust,ignore
/// use feather::{App, middlewares::builtins::ServeStatic};
///
/// let mut app = App::new();
/// app.use_middleware(ServeStatic::new("./public".to_string()));
/// ```
//TODO FIX WIN ERRORS
pub struct ServeStatic(String);

impl ServeStatic {
    /// Create a new static file server for the given directory.
    ///
    /// # Arguments
    ///
    /// * `directory` - Path to the directory containing static files
    ///
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let serve = ServeStatic::new("./public".to_string());
    /// app.use_middleware(serve);
    /// ```
    #[must_use = "This middleware must be added to the app with use_middleware()"]
    pub const fn new(directory: String) -> Self {
        Self(directory)
    }

    fn handle_io_error(&self, e: io::Error, path: &Path, response: &mut Response) {
        let status_code = match e.kind() {
            io::ErrorKind::PermissionDenied => 403,
            io::ErrorKind::NotFound => 404,
            _ => 500, // Internal Server Error for other IO issues
        };

        eprintln!(
            "ServeStatic: Error accessing path {:?} (Base: {}): {} - Responding with {}",
            path, &self.0, e, status_code
        );

        response.set_status(status_code);
        match status_code {
            404 => response.send_text("404 Not Found"),
            403 => response.send_text("403 Forbidden"),
            _ => response.send_text("500 Internal Server Error"),
        };
    }

    fn guess_content_type(path: &Path) -> &'static str {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("html") | Some("htm") => "text/html; charset=utf-8",
            Some("css") => "text/css; charset=utf-8",
            Some("js") => "application/javascript; charset=utf-8",
            Some("json") => "application/json",
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            Some("svg") => "image/svg+xml",
            Some("ico") => "image/x-icon",
            Some("txt") => "text/plain; charset=utf-8",
            _ => "application/octet-stream", // Default binary type
        }
    }
}

impl Middleware for ServeStatic {
    fn handle(&self, request: &mut Request, response: &mut Response, _: &AppContext) -> Outcome {
        let requested_path = request.uri.path().trim_start_matches('/');
        let base_dir = Path::new(&self.0);
        let mut target_path = base_dir.join(requested_path);

        match target_path.canonicalize() {
            Ok(canonical_path) => {
                // Need to canonicalize base_dir too for reliable comparison
                match base_dir.canonicalize() {
                    Ok(canonical_base) => {
                        if !canonical_path.starts_with(&canonical_base) {
                            // Path tried to escape the base directory!
                            eprintln!(
                                "ServeStatic: Forbidden path traversal attempt: Requested '{}', Resolved '{}' outside base '{}'",
                                requested_path,
                                canonical_path.display(),
                                canonical_base.display()
                            );
                            response.set_status(403);
                            response.send_text("403 Forbidden");
                            return next!();
                        }
                        target_path = canonical_path;
                    }
                    Err(e) => {
                        // Failed to canonicalize base directory - major configuration issue
                        self.handle_io_error(e, base_dir, response);
                        return next!();
                    }
                }
            }
            Err(e) => {
                self.handle_io_error(e, &target_path, response);
                return next!();
            }
        }

        match fs::metadata(&target_path) {
            Ok(metadata) => {
                if metadata.is_file() {
                    match File::open(&target_path) {
                        Ok(mut file) => {
                            let mut buffer = Vec::new();
                            match file.read_to_end(&mut buffer) {
                                Ok(_) => {
                                    let content_type = Self::guess_content_type(&target_path);
                                    response.add_header("Content-Type", content_type)?;
                                    response.add_header("Content-Length", &buffer.len().to_string())?;
                                    response.send_bytes(buffer);
                                }
                                Err(e) => {
                                    self.handle_io_error(e, &target_path, response);
                                }
                            }
                        }
                        Err(e) => {
                            self.handle_io_error(e, &target_path, response);
                        }
                    }
                } else if metadata.is_dir() {
                    eprintln!("ServeStatic: Access denied for directory: {:?}", target_path);
                    response.set_status(403);
                    response.send_text("403 Forbidden");
                } else {
                    eprintln!("ServeStatic: Path is not a file or directory: {:?}", target_path);
                    response.set_status(404);
                    response.send_text("404 Not Found");
                }
            }
            Err(e) => {
                // Error getting metadata (likely Not Found or Permission Denied)
                self.handle_io_error(e, &target_path, response);
            }
        }

        next!()
    }
}
