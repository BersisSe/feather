use bytes::Bytes;
use http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use serde::Serialize;
use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, Default)]
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
    /// But The Other Method like `json()`,`body()`can be used to get the body in different formats.
    pub body: Option<Bytes>,
    /// The HTTP version of the response.
    pub version: http::Version,
}

impl Response {
    /// Sets the StatusCode of the response and Returns a Muteable Reference to the Response so you can things like
    /// ```rust
    /// res.status(200).send_text("eyo");
    /// ```
    /// The StatusCode is a 3-digit integer that indicates the result of the request.    
    pub fn status(&mut self, status: u16) -> &mut Response {
        self.status = StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        self
    }
    /// Adds a header to the response.
    /// The header is a key-value pair that provides additional information about the response.
    pub fn add_header(&mut self, key: &str, value: &str) -> Option<()> {
        if let Ok(val) = HeaderValue::from_str(value) {
            if let Ok(key) = HeaderName::from_str(key) {
                self.headers.insert(key, val);
            }
            return None;
        }
        None
    }
    /// Converts the `HttpResponse` into a raw HTTP response string.
    pub fn to_raw(&self) -> String {
        let mut response = format!(
            "HTTP/1.1 {} {}\r\n",
            self.status.as_u16(),
            self.status.canonical_reason().unwrap_or("Unknown")
        );

        for (key, value) in &self.headers {
            response.push_str(&format!("{}: {}\r\n", key, value.to_str().unwrap()));
        }

        response.push_str("\r\n");

        if let Some(ref body) = self.body {
            response.push_str(&String::from_utf8_lossy(body));
        }
        response
    }

    /// Converts the `HttpResponse` into a raw HTTP response as bytes.
    pub fn to_bytes(&self) -> Bytes {
        let mut response = self.to_string().into_bytes();
        if let Some(ref body) = self.body {
            response.extend_from_slice(body);
        }

        Bytes::from(response)
    }

    pub fn send_text(&mut self, data: impl Into<String>) {
        let body = data.into();
        self.body = Some(Bytes::from(body));
        self.headers
            .insert("Content-Type", "text/plain".parse().unwrap());
        self.headers.insert(
            "Content-Length",
            self.body
                .as_ref()
                .unwrap()
                .len()
                .to_string()
                .parse()
                .unwrap(),
        );
        self.headers
            .insert("Date", chrono::Utc::now().to_string().parse().unwrap());
    }
    pub fn send_bytes(&mut self, data: impl Into<Vec<u8>>) {
        let body = data.into();
        self.headers
            .insert("Date", chrono::Utc::now().to_string().parse().unwrap());
        self.body = Some(Bytes::from(body));
        self.headers.insert(
            "Content-Length",
            self.body
                .as_ref()
                .unwrap()
                .len()
                .to_string()
                .parse()
                .unwrap(),
        );
    }
    pub fn send_html(&mut self, data: impl Into<String>) {
        let body = data.into();
        self.body = Some(Bytes::from(body));
        self.headers
            .insert("Date", chrono::Utc::now().to_string().parse().unwrap());
        self.headers
            .insert("Content-Type", "text/html".parse().unwrap());
        self.headers.insert(
            "Content-Length",
            self.body
                .as_ref()
                .unwrap()
                .len()
                .to_string()
                .parse()
                .unwrap(),
        );
    }
    pub fn send_json<T: Serialize>(&mut self, data: T) {
        match serde_json::to_string(&data) {
            Ok(json) => {
                self.body = Some(Bytes::from(json));
                self.headers
                    .insert("Date", chrono::Utc::now().to_string().parse().unwrap());
                self.headers
                    .insert("Content-Type", HeaderValue::from_static("application/json"));
                self.headers.insert(
                    "Content-Length",
                    self.body
                        .as_ref()
                        .unwrap()
                        .len()
                        .to_string()
                        .parse()
                        .unwrap(),
                );
            }
            Err(_) => {
                self.headers
                    .insert("Date", chrono::Utc::now().to_string().parse().unwrap());
                self.status = StatusCode::INTERNAL_SERVER_ERROR;
                self.body = Some(Bytes::from("Internal Server Error"));
                self.headers
                    .insert("Content-Type", HeaderValue::from_static("text/plain"));
                self.headers.insert(
                    "Content-Length",
                    self.body
                        .as_ref()
                        .unwrap()
                        .len()
                        .to_string()
                        .parse()
                        .unwrap(),
                );
            }
        }
    }
}

impl Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
