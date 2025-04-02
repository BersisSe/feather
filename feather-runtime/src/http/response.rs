use std::str::FromStr;
use bytes::Bytes; // Import Bytes
use http::{HeaderMap, HeaderName, HeaderValue, StatusCode};

#[derive(Debug, Clone, Default)]
pub struct HttpResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Option<Bytes>, // Use Bytes instead of String
}

impl HttpResponse {
    /// Creates a new `HttpResponse` with a status code and optional body.
    pub fn new(status: StatusCode, body: Option<Bytes>) -> Self {
        let mut headers = HeaderMap::new();
        if let Some(ref body) = body {
            headers.insert("Content-Length", body.len().to_string().parse().unwrap());
            headers.insert("Content-Type", "application/octet-stream".parse().unwrap());
        }

        Self { status, headers, body }
    }

    /// Convenience method for creating a 200 OK response with a body.
    pub fn ok(body: impl Into<String>) -> Self {
        let mut headers = HeaderMap::new();
        let body = body.into();
        headers.insert("Content-Length", body.len().to_string().parse().unwrap());
        headers.insert("Content-Type", "text/plain".parse().unwrap());
        Self {
            status: StatusCode::OK,
            headers,
            body: Some(Bytes::from(body)),
        }
    }
    pub fn add_header(&mut self, key: &str ,value: &str) -> Option<()>{
        if let Ok(val) = HeaderValue::from_str(value) {
            if let Ok(key) = HeaderName::from_str(key){
                self.headers.insert(key, val);
            }
            return None;
        }
        None 
    }
    /// Converts the `HttpResponse` into a raw HTTP response string.
    pub fn to_string(&self) -> String {
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
}