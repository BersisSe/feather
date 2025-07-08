use super::ConnectionState;
use crate::utils::error::Error;
use bytes::Bytes;
use http::{Extensions, HeaderMap, Method, Uri, Version};
use std::str::FromStr;
use std::{borrow::Cow, collections::HashMap, fmt};
use std::net::TcpStream;
use urlencoding::decode;

/// Contains a incoming Http Request
#[derive(Debug)]
pub struct Request {
    stream: Option<TcpStream>,
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
        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut request = httparse::Request::new(&mut headers);
        let mut connection = None;
        request.parse(raw).map_err(|e| Error::ParseError(format!("Failed to parse request: {}", e)))?;
        let method = match Method::from_str(request.method.unwrap_or("nil")) {
            Ok(m) => m,
            Err(e) => return Err(Error::ParseError(format!("Failed to parse method: {}", e))),
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

            if name.as_str().eq_ignore_ascii_case("connection") {
                connection = ConnectionState::parse(value.to_str().unwrap_or(""));
            }

            header_map.insert(name, value);
        }
        let body_start = raw.windows(4).position(|w| w == b"\r\n\r\n").map(|pos| pos + 4).unwrap_or(raw.len());
        let body = Bytes::copy_from_slice(&raw[body_start..]);
        let extensions = Extensions::new();
        Ok(Request {
            stream: None,
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

    pub(crate) fn set_stream(&mut self, stream: TcpStream) {
        self.stream = Some(stream);
    }
    /// Takes the stream of the request
    /// This medhod takes ownership of the Request!  
    /// And also gives you to responsiblity give answer to the socket connection
    /// This method is only intented to used by Power Users use **it at your own risk.**  
    /// ## Usage:
    /// Takes the stream out but you have to clone the request to do so.  
    /// ```rust,ignore
    /// if let Some(stream) = request.clone().take_stream(){
    ///     
    /// }
    /// ```
    /// The Reason of this is in the middlewares don't allow you to have ownership of the request and response objects so to get ownership you have to clone it
    pub fn take_stream(mut self) -> Option<TcpStream> {
        self.stream.take()
    }
    /// Returns true if the Request has a Stream false if stream is taken
    pub fn has_stream(&self) -> bool {
        self.stream.is_some()
    }

    /// Parses the body of the request as Serde JSON Value. Returns an error if the body is not valid JSON.  
    /// This method is useful for parsing JSON payloads in requests.  
    #[cfg(feature = "json")]
    pub fn json(&self) -> Result<serde_json::Value, Error> {
        serde_json::from_slice(&self.body).map_err(|e| Error::ParseError(format!("Failed to parse JSON body: {}", e)))
    }
    /// Returns a Hashmap of the query parameters of the Request.  
    /// Returns a Error if parsing fails
    pub fn query(&self) -> Result<HashMap<String, String>, Error> {
        if let Some(query) = self.uri.query() {
            serde_urlencoded::from_str(query).map_err(|e| Error::ParseError(format!("Failed to Parse Query parameters {}", e)))
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
