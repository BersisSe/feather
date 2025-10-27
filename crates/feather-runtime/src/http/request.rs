use super::ConnectionState;
use std::io;
/// Simple alias for error results in this module.
/// We use a boxed std error to avoid depending on the removed crate error type.
pub type Error = Box<dyn std::error::Error + Send + Sync>;
use bytes::Bytes;
use http::{Extensions, HeaderMap, Method, Uri, Version};
use std::str::FromStr;
use std::{borrow::Cow, collections::HashMap, fmt};
use urlencoding::decode;

/// Contains a incoming Http Request
#[derive(Debug)]
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

    /// The route parameters of the request.
    params: HashMap<String, String>,
    // Connection State(Keep-Alive OR Close) of the Request
    pub(crate) connection: Option<ConnectionState>,
}

impl Request {
    /// Parses a Request from raw bytes if parsing fails returns a error
    /// #### Puts a None to as stream of the request!
    pub fn parse(raw: &[u8]) -> Result<Request, Error> {
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut request = httparse::Request::new(&mut headers);
        let mut connection = None;
        request.parse(raw).map_err(|e| -> Error { Box::new(io::Error::new(io::ErrorKind::InvalidData, format!("Failed to parse request: {}", e))) })?;
        let method = match Method::from_str(request.method.unwrap_or("nil")) {
            Ok(m) => m,
            Err(e) => return Err(Box::new(io::Error::new(io::ErrorKind::InvalidData, format!("Failed to parse method: {}", e)))),
        };
        let path = request.path.ok_or_else(|| -> Error { Box::new(io::Error::new(io::ErrorKind::InvalidData, "Failed to parse URI")) })?;
        let uri: Uri = path.parse().map_err(|e| -> Error { Box::new(io::Error::new(io::ErrorKind::InvalidData, format!("Failed to parse URI: {}", e))) })?;

        let version = match request.version {
            Some(0) => Version::HTTP_10,
            Some(1) => Version::HTTP_11,
            _ => Version::HTTP_11,
        };
        let mut header_map = HeaderMap::new();
        for header in request.headers.iter() {
            let name = http::header::HeaderName::from_bytes(header.name.as_bytes()).map_err(|e| -> Error { Box::new(io::Error::new(io::ErrorKind::InvalidData, format!("Failed to parse header name: {}", e))) })?;
            let value = http::header::HeaderValue::from_bytes(header.value).map_err(|e| -> Error { Box::new(io::Error::new(io::ErrorKind::InvalidData, format!("Failed to parse header value: {}", e))) })?;

            if name.as_str().eq_ignore_ascii_case("connection") {
                connection = ConnectionState::parse(value.to_str().unwrap_or(""));
            }

            header_map.insert(name, value);
        }
        let body_start = raw.windows(4).position(|w| w == b"\r\n\r\n").map(|pos| pos + 4).unwrap_or(raw.len());
        let body = Bytes::copy_from_slice(&raw[body_start..]);
        let extensions = Extensions::new();
        Ok(Request {
            method,
            uri,
            version,
            headers: header_map,
            body,
            extensions,
            connection,
            params: HashMap::new(),
        })
    }

    /// Parses the body of the request as Serde JSON Value. Returns an error if the body is not valid JSON.  
    /// This method is useful for parsing JSON payloads in requests.  
    #[cfg(feature = "json")]
    pub fn json(&self) -> Result<serde_json::Value, Error> {
        serde_json::from_slice(&self.body).map_err(|e| -> Error { Box::new(io::Error::new(io::ErrorKind::InvalidData, format!("Failed to parse JSON body: {}", e))) })
    }
    /// Returns a Hashmap of the query parameters of the Request.  
    /// Returns a Error if parsing fails
    pub fn query(&self) -> Result<HashMap<String, String>, Error> {
        if let Some(query) = self.uri.query() {
            serde_urlencoded::from_str(query).map_err(|e| -> Error { Box::new(io::Error::new(io::ErrorKind::InvalidData, format!("Failed to Parse Query parameters {}", e))) })
        } else {
            Ok(HashMap::new())
        }
    }

    pub fn set_params(&mut self, params: HashMap<String, String>) {
        self.params = params;
    }

    pub fn param(&self, key: &str) -> Option<&str> {
        self.params.get(key).map(|v| &**v)
    }

    /// Returns the path of the Request
    pub fn path(&self) -> Cow<'_, str> {
        decode(self.uri.path()).unwrap()
    }
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} for {}: Body Data: {} ", self.method, self.uri.path(), String::from_utf8_lossy(&self.body))
    }
}
