use crate::utils::error::Error;
use bytes::Bytes;
use http::{Extensions, HeaderMap, Method, Uri, Version};
use std::fmt;


#[derive(Debug, Clone)]
pub struct Request {
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
    pub body: Bytes, 
    /// The extensions of the request.
    pub extensions: Extensions, 
}

impl Request {
    /// Parses the body of the request as Serde JSON Value. Returns an error if the body is not valid JSON.
    /// This method is useful for parsing JSON payloads in requests.
    pub fn json(&self) -> Result<serde_json::Value, Error> {
        serde_json::from_slice(&self.body).map_err(|e| {
            Error::ParseError(format!("Failed to parse JSON body: {}", e))
        })
    }
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} for {}: Body Data: {} ",
            self.method,
            self.uri,
            String::from_utf8_lossy(&self.body)
        ) 
    }
}
