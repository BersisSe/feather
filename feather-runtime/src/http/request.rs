

use httparse;
use http::{ HeaderMap, Method, Uri, Version ,Extensions};
use std::{fmt, str::FromStr };
use bytes::Bytes;
use crate::utils::error::Error; // Assuming you have an Error module in utils

#[derive(Debug,Clone)]
pub struct HttpRequest {
    /// The HTTP method of the request.<br>
    /// For example, GET, POST, PUT, DELETE, etc.
    pub method: Method,
    /// The URI of the request.
    pub uri: Uri,
    /// The HTTP version of the request.
    pub version: Version,
    /// The headers of the request.
    pub headers: HeaderMap,
    /// The body of the request.
    pub body: Bytes, // Changed from String to Bytes
    /// The extensions of the request.
    pub extensions: Extensions, // Added extensions field
    
}

impl HttpRequest {
    /// Parses a raw HTTP request into an `HttpRequest` struct.
    pub fn parse(raw: &[u8]) -> Result<Self, Error> {
        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut request = httparse::Request::new(&mut headers);
        request.parse(raw).map_err(|e| Error::ParseError(format!("Failed to parse request: {}", e)))?;
        let method = match Method::from_str(request.method.unwrap_or("nil")) {
            Ok(m) => {m},
            Err(e) => {return Err(Error::ParseError(format!("Failed to parse method: {}", e)))}
        };
        let uri: Uri = request.path.ok_or_else(|| Error::ParseError("Failed to parse URI".to_string()))?.parse().map_err(|e| Error::ParseError(format!("Failed to parse URI: {}", e)))?;
            
        let version = match request.version {
            Some(0) => Version::HTTP_10,
            Some(1) => Version::HTTP_11,
            _ => Version::HTTP_11,
        };
        let mut header_map = HeaderMap::new();
        for header in request.headers.iter() {
            let name = http::header::HeaderName::from_bytes(header.name.as_bytes()).map_err(|e| Error::ParseError(format!("Failed to parse header name: {}", e)))?;
            let value = http::header::HeaderValue::from_bytes(header.value).map_err(|e| Error::ParseError(format!("Failed to parse header value: {}", e)))?;
            header_map.insert(name, value);
        };
        let body_start = raw.windows(4).position(|w| w == b"\r\n\r\n").map(|pos| pos + 4).unwrap_or(raw.len());
        let body = Bytes::copy_from_slice(&raw[body_start..]); // Changed from String to Bytes
        let extensions = Extensions::new();
        Ok(Self {
            method,
            uri,
            version,
            headers: header_map,
            body,
            extensions
        })
    }
}

impl fmt::Display for HttpRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} for {}: Body Data: {} ",self.method, self.uri,String::from_utf8_lossy(&self.body)) // Changed from self.body.to_string() to String::from_utf8_lossy(&self.body)
    }
}


