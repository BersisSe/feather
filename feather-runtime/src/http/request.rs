use httparse;
use http::{ HeaderMap, Method, Uri, Version };
use std::{fmt, str::FromStr };

#[derive(Debug,Clone)]
pub struct HttpRequest {
    pub method: Method,
    pub uri: Uri,
    pub version: Version,
    pub headers: HeaderMap,
    pub body: String,
}

impl HttpRequest {
    /// Parses a raw HTTP request into an `HttpRequest` struct.
    pub fn parse(raw: &[u8]) -> Result<Self, String> {
        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut request = httparse::Request::new(&mut headers);
        request.parse(raw).unwrap();
        let method = Method::from_str(request.method.unwrap_or("nil")).unwrap();
        let uri: Uri = request.path.ok_or("Miss").unwrap().parse().unwrap();
        let version = match request.version {
            Some(0) => Version::HTTP_10,
            Some(1) => Version::HTTP_11,
            _ => Version::HTTP_11,
        };
        let mut header_map = HeaderMap::new();
        for header in request.headers.iter() {
            let name = http::header::HeaderName::from_bytes(header.name.as_bytes()).unwrap();
            let value = http::header::HeaderValue::from_bytes(header.value).unwrap();
            header_map.insert(name, value);
        };
        let body_start = raw.windows(4).position(|w| w == b"\r\n\r\n").map(|pos| pos + 4).unwrap_or(raw.len());
        let body = String::from_utf8(raw[body_start..].to_vec()).unwrap();

        Ok(Self {
            method,
            uri,
            version,
            headers: header_map,
            body,
        })
    }
}

impl fmt::Display for HttpRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} for {}: Body Data: {} ",self.method, self.uri,self.body.to_string())
    }
}