//! Built-in middleware for common functionality.
//!
//! This module provides ready-to-use middleware for logging, CORS, and static file serving.

use super::common::Middleware;
use crate::{Outcome, end, internals::AppContext, next};

use feather_runtime::http::{Request, Response};
#[cfg(feature = "log")]
use log::info;
use std::{
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
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
#[cfg(feature = "log")]
impl Middleware for Logger {
    fn handle(&self, _request: &mut Request, _: &mut Response, _: &AppContext) -> Outcome {
        #[cfg(feature = "log")]
        info!("{} {}", _request.method, _request.uri.path());
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
pub struct ServeStatic {
    base_path: PathBuf,
}

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
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self{
            base_path: directory.into()
        }
    }
    /// Internal Strip the Windows UNC Prefix.
    fn strip_unc(path: &Path) -> &Path {
        if let Some(path_str) = path.to_str(){
            if path_str.starts_with(r"\\?\"){
                return Path::new(&path_str[4..]);
            }
        }
        path
    }

    fn handle_io_error(&self, e: io::Error, path: &Path, response: &mut Response) {
        let status_code = match e.kind() {
            io::ErrorKind::PermissionDenied => 403,
            io::ErrorKind::NotFound => 404,
            _ => 500, // Internal Server Error for other IO issues
        };

        eprintln!(
            "ServeStatic: Error accessing path {:?} (Base: {}): {} - Responding with {}",
            path, &self.base_path.display(), e, status_code
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
        
        if requested_path.contains("..") {
            response.set_status(403);
            response.send_text("403 Forbidden");
            return end!(); // Cut of Execution, this is a security risk
        }

        let full_path = self.base_path.join(requested_path);

        match full_path.canonicalize() {
            Ok(canonical_target) => {
                match self.base_path.canonicalize() {
                    Ok(canonical_base) => {
                        let clean_target = Self::strip_unc(&canonical_target);
                        let clean_base = Self::strip_unc(&canonical_base);

                        if !clean_target.starts_with(clean_base) {
                            response.set_status(403);
                            response.send_text("403 Forbidden");
                            return end!(); 
                        }

                        match fs::metadata(clean_target) {
                            Ok(metadata) => {
                                if metadata.is_file() {
                                    match File::open(clean_target) {
                                        Ok(mut file) => {
                                            let mut buffer = Vec::new();
                                            if file.read_to_end(&mut buffer).is_ok() {
                                                let ct = Self::guess_content_type(clean_target);
                                                response.add_header("Content-Type", ct)?;
                                                response.add_header("Content-Length", &buffer.len().to_string())?;
                                                response.send_bytes(buffer);
                                                // We found the file and filled the response.
                                                // We return end!() so the Router doesn't overwrite us with a 404.
                                                return end!(); 
                                            }
                                        }
                                        Err(e) => {
                                            self.handle_io_error(e, clean_target, response);
                                            return end!();
                                        }
                                    }
                                } else if metadata.is_dir() {
                                    // We Return next here ServeStatic Can't serve directories.
                                    // So give control back to the router so if user has defined a handler for the path it will still execute.
                                    return next!();
                                }
                            }
                            Err(e) => {
                                self.handle_io_error(e, clean_target, response);
                                return end!();
                            }
                        }
                    }
                    Err(e) => {
                        self.handle_io_error(e, &self.base_path, response);
                        return end!();
                    }
                }
            }
            Err(_) => {
                // File not found?
                // Just give control back to the Router so it can try match!
                return next!();
            }
        }

        next!()
    }
}