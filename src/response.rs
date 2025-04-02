use std::{collections::HashMap, fs::File};

/// A response which passes through the middleware chain.
pub struct Response {
    /// The [HTTP status code] of the response.
    ///
    /// [HTTP status code]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Status
    pub(crate) status_code: u16,
    /// The body of the response.
    pub(crate) body: Option<String>,
    /// The file to be sent in the response.
    pub(crate) file: Option<File>,
    /// The headers of the response.
    pub(crate) headers: HashMap<String, String>,
}

impl Default for Response {
    fn default() -> Self {
        Self {
            status_code: 200,
            body: None,
            file: None,
            headers: HashMap::new(),
        }
    }
}

impl Response {
    pub const OK: u16 = 200;
    pub const NOT_FOUND: u16 = 404;

    /// Create a new response with the given status code and body.
    pub fn new(status_code: u16, body: impl Into<String>) -> Self {
        Self {
            status_code,
            body: Some(body.into()),
            file: None,
            headers: HashMap::new(),
        }
    }

    /// Create a new response with a status code of 200 (OK) and the given body.
    pub fn ok(body: impl Into<String>) -> Self {
        Self::new(Self::OK, body)
    }

    #[must_use]
    /// Create a new response with the given status code and file.
    pub fn from_file(status_code: u16, file: File) -> Self {
        Self {
            status_code,
            body: None,
            file: Some(file),
            headers: HashMap::new(),
        }
    }

    /// Add a header to the response.
    pub fn with_header(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.headers.insert(key.into(), value.into());
        self
    }
}
