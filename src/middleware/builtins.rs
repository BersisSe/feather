use crate::internals::AppContext;

use super::common::{Middleware, MiddlewareResult};
use feather_runtime::http::{Request, Response};
use std::{
    fs::{self, File},
    io::{self, Read},
    path::Path,
};

/// Log incoming requests and transparently pass them to the next middleware.
pub struct Logger;

impl Middleware for Logger {
    fn handle(&self, request: &mut Request, _: &mut Response, _: &mut AppContext) -> MiddlewareResult {
        println!("Request: {request}");
        MiddlewareResult::Next
    }
}

#[derive(Default)]
/// Add [CORS] headers to the response.
///
/// [CORS]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/CORS
pub struct Cors(Option<String>);

impl Cors {
    #[must_use]
    pub const fn new(origin: String) -> Self {
        Self(Some(origin))
    }
}

impl Middleware for Cors {
    fn handle(&self, _: &mut Request, response: &mut Response, _: &mut AppContext) -> MiddlewareResult {
        response.add_header(
            "Access-Control-Allow-Origin",
            self.0.as_deref().unwrap_or("*"),
        );
        MiddlewareResult::Next
    }
}

/// Serve static files from the given path.
pub struct ServeStatic(String);

impl ServeStatic {
    #[must_use = "Put this in a `App.use_middleware()` Method"]
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

        response.status(status_code);
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
    fn handle(
        &self,
        request: &mut Request,
        response: &mut Response,
        _: &mut AppContext,
    ) -> MiddlewareResult {
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
                                requested_path, canonical_path.display(), canonical_base.display()
                            );
                            response.status(403);
                            response.send_text("403 Forbidden");
                            return MiddlewareResult::Next;
                        }
                        target_path = canonical_path;
                    }
                    Err(e) => {
                        // Failed to canonicalize base directory - major configuration issue
                        self.handle_io_error(e, base_dir, response);
                        return MiddlewareResult::Next;
                    }
                }
            }
            Err(e) => {
                self.handle_io_error(e, &target_path, response);
                return MiddlewareResult::Next;
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
                                    response.add_header("Content-Type", content_type);
                                    response.add_header("Content-Length", &buffer.len().to_string());
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
                    response.status(403);
                    response.send_text("403 Forbidden");
                } else {
                    eprintln!("ServeStatic: Path is not a file or directory: {:?}", target_path);
                    response.status(404);
                    response.send_text("404 Not Found");
                }
            }
            Err(e) => {
                // Error getting metadata (likely Not Found or Permission Denied)
                self.handle_io_error(e, &target_path, response);
            }
        }

        MiddlewareResult::Next
    }
}
