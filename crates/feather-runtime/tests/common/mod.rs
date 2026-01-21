use bytes::Bytes;
use feather_runtime::http::{Request, Response};
use feather_runtime::runtime::service::{Service, ServiceResult};
use may::net::TcpStream;
use std::io;

/// A simple echo service for testing
pub struct EchoService;

impl Service for EchoService {
    fn handle(&self, req: Request, _stream: Option<TcpStream>) -> io::Result<ServiceResult> {
        let mut response = Response::default();
        response.set_status(200);
        response.send_text(format!("Echo: {}", String::from_utf8_lossy(&req.body)));
        Ok(ServiceResult::Response(response))
    }
}

/// Helper to create a test HTTP request
pub fn create_test_request(method: &str, path: &str, body: &[u8]) -> Vec<u8> {
    let mut request = Vec::new();
    request.extend_from_slice(format!("{} {} HTTP/1.1\r\n", method, path).as_bytes());
    request.extend_from_slice(b"Host: localhost\r\n");
    if !body.is_empty() {
        request.extend_from_slice(format!("Content-Length: {}\r\n", body.len()).as_bytes());
    }
    request.extend_from_slice(b"\r\n");
    request.extend_from_slice(body);
    request
}


pub fn split_request(buf: &[u8]) -> (&[u8], Bytes) {
    let header_end = buf
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|p| p + 4)
        .unwrap_or(buf.len());

    let headers = &buf[..header_end];
    let body = Bytes::copy_from_slice(&buf[header_end..]);

    (headers, body)
}
