use bytes::Bytes;
use feather_runtime::http::Request;
mod common;
use common::ADDR;

pub fn split_request(buf: &[u8]) -> (&[u8], Bytes) {
    let header_end = buf.windows(4).position(|w| w == b"\r\n\r\n").map(|p| p + 4).unwrap_or(buf.len());

    let headers = &buf[..header_end];
    let body = Bytes::copy_from_slice(&buf[header_end..]);

    (headers, body)
}

#[test]
fn test_parse_simple_get_request() {
    let raw = b"GET /test HTTP/1.1\r\nHost: example.com\r\n\r\n";
    let request = Request::parse(raw, Bytes::new(), ADDR).unwrap();
    assert_eq!(request.method.as_str(), "GET");
    assert_eq!(request.path().as_ref(), "/test");
    assert_eq!(request.headers.len(), 1);
}

#[test]
fn test_parse_request_with_body() {
    let headers_raw = b"POST /submit HTTP/1.1\r\nHost: example.com\r\nContent-Length: 11\r\n\r\n";
    let body = b"Hello World";
    let request = Request::parse(headers_raw, Bytes::from_static(body), ADDR).unwrap();
    assert_eq!(request.method.as_str(), "POST");
    assert_eq!(request.path().as_ref(), "/submit");
    assert_eq!(*request.body, *b"Hello World");
}

#[test]
fn test_parse_request_with_query_params() {
    let raw = b"GET /search?q=test&page=1 HTTP/1.1\r\nHost: example.com\r\n\r\n";
    let request = Request::parse(raw, Bytes::new(), ADDR).unwrap();
    let params = request.query().unwrap();
    assert_eq!(params.get("q").unwrap(), "test");
    assert_eq!(params.get("page").unwrap(), "1");
}

#[test]
fn test_parse_request_with_headers() {
    let raw = b"GET / HTTP/1.1\r\nHost: example.com\r\nUser-Agent: test\r\nAccept: */*\r\n\r\n";
    let request = Request::parse(raw, Bytes::new(), ADDR).unwrap();
    assert_eq!(request.headers.len(), 3);
    assert_eq!(request.headers.get("user-agent").unwrap(), "test");
    assert_eq!(request.headers.get("accept").unwrap(), "*/*");
}

#[test]
fn test_parse_invalid_method() {
    // HTTP allows extension/custom methods. Ensure parser accepts token-like methods.
    let raw = b"INVALID / HTTP/1.1\r\nHost: example.com\r\n\r\n";
    let req = Request::parse(raw, Bytes::new(), ADDR).expect("should parse custom method");
    assert_eq!(req.method.as_str(), "INVALID");
}

#[test]
fn test_parse_empty_method() {
    let raw = b" / HTTP/1.1\r\nHost: example.com\r\n\r\n";
    assert!(Request::parse(raw, Bytes::new(), ADDR).is_err());
}

#[test]
fn test_parse_missing_method() {
    let raw = b"/test HTTP/1.1\r\nHost: example.com\r\n\r\n";
    assert!(Request::parse(raw, Bytes::new(), ADDR).is_err());
}

#[test]
fn test_valid_http_methods() {
    let valid_methods = ["GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS", "PATCH"];

    for method in valid_methods.iter() {
        let raw = format!("{} /test HTTP/1.1\r\nHost: example.com\r\n\r\n", method);
        let request = Request::parse(raw.as_bytes(), Bytes::new(), ADDR).unwrap();
        assert_eq!(request.method.as_str(), *method);
    }
}
