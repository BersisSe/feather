use std::str::FromStr;
use super::{ConnectionState, Request};
use crate::utils::error::Error;
use bytes::Bytes;
use http::{Extensions, HeaderMap, Method, Uri, Version};

pub fn parse(raw: &[u8]) -> Result<Request, Error> {
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut request = httparse::Request::new(&mut headers);
    let mut connection= None;
    request
        .parse(raw)
        .map_err(|e| Error::ParseError(format!("Failed to parse request: {}", e)))?;
    let method = match Method::from_str(request.method.unwrap_or("nil")) {
        Ok(m) => m,
        Err(e) => return Err(Error::ParseError(format!("Failed to parse method: {}", e))),
    };
    let uri: Uri = request
        .path
        .ok_or_else(|| Error::ParseError("Failed to parse URI".to_string()))?
        .parse()
        .map_err(|e| Error::ParseError(format!("Failed to parse URI: {}", e)))?;

    let version = match request.version {
        Some(0) => Version::HTTP_10,
        Some(1) => Version::HTTP_11,
        _ => Version::HTTP_11,
    };
    let mut header_map = HeaderMap::new();
    for header in request.headers.iter() {
        let name = http::header::HeaderName::from_bytes(header.name.as_bytes())
            .map_err(|e| Error::ParseError(format!("Failed to parse header name: {}", e)))?;
        let value = http::header::HeaderValue::from_bytes(header.value)
            .map_err(|e| Error::ParseError(format!("Failed to parse header value: {}", e)))?;
        
        if name.as_str().eq_ignore_ascii_case("connection"){
            connection = ConnectionState::parse(value.to_str().unwrap_or(""))
        }
        else {
            connection = None
        }
        header_map.insert(name, value);
    }
    let body_start = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|pos| pos + 4)
        .unwrap_or(raw.len());
    let body = Bytes::copy_from_slice(&raw[body_start..]); // Changed from String to Bytes
    let extensions = Extensions::new();
    Ok(Request {
        method,
        uri,
        version,
        headers: header_map,
        body,
        extensions,
        connection
    })
}