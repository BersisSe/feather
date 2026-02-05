use super::errors::HeaderError;
use bytes::{Bytes, BytesMut};
use http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
#[cfg(feature = "json")]
use serde::Serialize;
use std::{fs::File, io::Read, str::FromStr};

#[derive(Debug, Default)]
pub struct Response {
    /// The HTTP status code of the response.
    /// This is a 3-digit integer that indicates the result of the request.
    pub status: StatusCode,
    /// The headers of the HTTP response.
    /// Headers are key-value pairs that provide additional information about the response.
    pub headers: HeaderMap,
    /// The body of the HTTP response.
    /// This is the content that is sent back to the client.
    /// The body is represented as a `Bytes` object for efficient handling of binary data.
    pub body: Option<Bytes>,
    /// The HTTP version of the response.
    pub version: http::Version,
}

impl Response {
    const MAX_FILE_SIZE_BYTES: u64 = 4 * 1024 * 1024; // 4 MB

    /// Internal helper to set common headers
    fn set_common_headers(&mut self, content_type: Option<&'static str>, len: usize) {
        if let Some(ct) = content_type {
            self.headers.insert(HeaderName::from_static("content-type"), HeaderValue::from_static(ct));
        }
        self.headers.insert(HeaderName::from_static("content-length"), Self::len_to_header_value(len));
    }

    /// Sets the StatusCode of the response and Returns a Muteable Reference to the Response
    /// ```rust,ignore
    /// res.status(200).send_text("hello");
    /// ```
    pub fn set_status(&mut self, status: u16) -> &mut Response {
        self.status = StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        self
    }

    /// Adds a header to the response.
    /// The header is a key-value pair that provides additional information about the response.
    ///
    pub fn add_header(&mut self, key: &str, value: &str) -> Result<(), HeaderError> {
        let val = HeaderValue::from_str(value)?;
        let key = HeaderName::from_str(key)?;
        self.headers.insert(key, val);
        return Ok(());
    }
    /// Converts the `Response` into a raw HTTP response as Bytes.
    pub fn to_raw(&self) -> Bytes {
        let body_len = self.body.as_ref().map_or(0, |b| b.len());
        // Start buffer with a reasonable capacity to avoid reallocations.
        let mut buf = BytesMut::with_capacity(512 + body_len);

        // --- 1. Status Line (HTTP/1.1 200 OK\r\n) ---
        buf.extend_from_slice(b"HTTP/1.1 ");

        // Use itoa::Buffer for stack-allocated status code formatting
        let mut status_buffer = itoa::Buffer::new();
        let status_code_str = status_buffer.format(self.status.as_u16());

        buf.extend_from_slice(status_code_str.as_bytes());
        buf.extend_from_slice(b" ");

        // Canonical Reason (e.g., "OK", "Not Found")
        buf.extend_from_slice(self.status.canonical_reason().unwrap_or("Unknown").as_bytes());
        buf.extend_from_slice(b"\r\n");

        // --- 2. Existing Headers ---
        for (key, value) in &self.headers {
            // Header Name
            buf.extend_from_slice(key.as_str().as_bytes());
            buf.extend_from_slice(b": ");
            // Header Value (already HeaderValue::from_static or from_str)
            buf.extend_from_slice(value.as_bytes());
            buf.extend_from_slice(b"\r\n");
        }

        // --- 3. Date Header Insertion (Crucial for HTTP/1.1) ---
        // Insert Date header if the user hasn't explicitly set it.
        // NOTE: This still uses a string allocation via `to_rfc2822()`.
        // For the absolute fastest approach, this string would be cached system-wide
        // and updated every second.
        if !self.headers.contains_key("date") {
            let date_str = chrono::Utc::now().to_rfc2822();
            buf.extend_from_slice(b"date: ");
            buf.extend_from_slice(date_str.as_bytes());
            buf.extend_from_slice(b"\r\n");
        }

        // --- 4. Content-Length Header Insertion ---
        // Insert Content-Length if it's not set AND there is a body.
        if !self.headers.contains_key("content-length") && body_len > 0 {
            buf.extend_from_slice(b"content-length: ");

            // Use itoa::Buffer for stack-allocated length formatting
            let mut len_buffer = itoa::Buffer::new();
            let len_str = len_buffer.format(body_len);

            buf.extend_from_slice(len_str.as_bytes());
            buf.extend_from_slice(b"\r\n");
        }

        // --- 5. Header/Body Separator ---
        buf.extend_from_slice(b"\r\n");

        // --- 6. Body ---
        if let Some(ref body) = self.body {
            buf.extend_from_slice(body);
        }

        // Convert mutable buffer to immutable Bytes type
        buf.freeze()
    }

    /// Sends given String as given text
    pub fn send_text(&mut self, data: impl Into<String>) {
        let body = data.into();
        self.body = Some(Bytes::from(body));
        self.set_common_headers(Some("text/plain;charset=utf-8"), self.body.as_ref().unwrap().len());
    }

    /// Sends Given Bytes as plain text
    pub fn send_bytes(&mut self, data: impl Into<Vec<u8>>) {
        let body = data.into();
        self.body = Some(Bytes::from(body));
        self.set_common_headers(None, self.body.as_ref().unwrap().len());
    }

    ///Takes a String(Should be valid HTML) and sends it's as Html
    pub fn send_html(&mut self, data: impl Into<String>) {
        let body = data.into();
        self.body = Some(Bytes::from(body));
        self.headers.insert(HeaderName::from_static("content-type"), HeaderValue::from_static("text/html"));
        let len = self.body.as_ref().unwrap().len();
        self.headers.insert(HeaderName::from_static("content-length"), Self::len_to_header_value(len));
    }

    /// Takes a Serializeable object and sends it as json.
    #[cfg(feature = "json")]
    pub fn send_json<T: Serialize>(&mut self, data: &T) {
        match serde_json::to_string(&data) {
            Ok(json) => {
                self.body = Some(Bytes::from(json));
                self.headers.insert(HeaderName::from_static("content-type"), HeaderValue::from_static("application/json"));
                let len = self.body.as_ref().unwrap().len();
                self.headers.insert(HeaderName::from_static("content-length"), Self::len_to_header_value(len));
            }
            Err(_) => {
                self.status = StatusCode::INTERNAL_SERVER_ERROR;
                self.body = Some(Bytes::from("Internal Server Error"));
                self.headers.insert(HeaderName::from_static("content-type"), HeaderValue::from_static("text/plain"));
                let len = self.body.as_ref().unwrap().len();
                self.headers.insert(HeaderName::from_static("content-length"), Self::len_to_header_value(len));
            }
        }
    }

    /// Take a [File] Struct and sends it as a file.
    /// File size is limited to 4MB. For larger files, chunked transfer\[WIP] is recommended.
    pub fn send_file(&mut self, mut file: File) {
        let metadata = match file.metadata() {
            Ok(m) => m,
            Err(_) => {
                self.status = StatusCode::INTERNAL_SERVER_ERROR;
                self.body = Some(Bytes::from("Failed to read file metadata."));
                return;
            }
        };

        // ENFORCE LIMIT: 4MB
        if metadata.len() > Self::MAX_FILE_SIZE_BYTES {
            self.status = StatusCode::PAYLOAD_TOO_LARGE; // 413
            self.body = Some(Bytes::from("File size exceeds 4MB limit. Use chunked encoding for larger files."));
            return;
        }

        let mut buffer = Vec::new();
        match file.read_to_end(&mut buffer) {
            Ok(_) => {
                self.body = Some(Bytes::from(buffer));
                let len = self.body.as_ref().unwrap().len();
                self.headers.insert(HeaderName::from_static("content-length"), Self::len_to_header_value(len));
                // ? NOTE: Consider adding feature : Content-Type based on file extension
            }
            Err(_) => {
                self.status = StatusCode::INTERNAL_SERVER_ERROR;
                self.body = Some(Bytes::from("Internal Server Error during file read."));
            }
        }
    }

    pub fn redirect(&mut self, location: &str, permanent: bool) {
        let status = if permanent {
            StatusCode::MOVED_PERMANENTLY
        } else {
            StatusCode::FOUND
        };
        self.set_status(status.as_u16());
        self.headers.insert(HeaderName::from_static("location"), HeaderValue::from_str(location).unwrap());
        self.body = Some(Bytes::from(format!("Redirecting to {}", location)));
        let len = self.body.as_ref().unwrap().len();
        self.headers.insert(HeaderName::from_static("content-length"), Self::len_to_header_value(len));
    }

    /// A Utily Function for wrapping HeaderValue for Content-Lenght
    fn len_to_header_value(len: usize) -> HeaderValue {
        let mut buffer = itoa::Buffer::new();
        let len_str = buffer.format(len);

        // ! SAFETY: Content-Length is only ASCII digits, which is safe for HeaderValue::from_bytes
        HeaderValue::from_bytes(len_str.as_bytes()).expect("itoa::Buffer output should be a valid HeaderValue")
    }
}
