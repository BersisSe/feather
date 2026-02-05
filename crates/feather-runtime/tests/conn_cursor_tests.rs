use std::io::{Cursor, Read};

use std::thread;
mod common;

use common::{ADDR, EchoService, create_test_request, split_request};
use feather_runtime::runtime::service::Service;

#[test]
fn test_conn_sim_single_request_with_cursor() {
    let request = create_test_request("GET", "/cursor", b"HelloCursor");

    // Simulate an incoming stream using Cursor
    let mut in_cursor = Cursor::new(request);

    let mut buf = Vec::new();
    in_cursor.read_to_end(&mut buf).unwrap();
    let (headers, body) = split_request(&buf);
    // Parse request from buffer
    let req = feather_runtime::http::Request::parse(headers, body, ADDR).unwrap();

    // Dispatch to service
    let service = EchoService;
    let res = service.handle(req, None).unwrap();
    if let feather_runtime::runtime::service::ServiceResult::Response(response) = res {
        let raw = response.to_raw();
        let raw_str = String::from_utf8_lossy(&raw);
        assert!(raw_str.contains("HTTP/1.1 200"));
        assert!(raw_str.contains("Echo: HelloCursor"));
    } else {
        panic!("Expected Response variant");
    }
}

#[test]
fn test_conn_sim_concurrent_cursors() {
    let mut handles = Vec::new();

    for i in 0..5 {
        handles.push(thread::spawn(move || {
            let body = format!("Payload {}", i);
            let request = create_test_request("POST", "/cursor", body.as_bytes());
            let mut in_cursor = Cursor::new(request);
            let mut buf = Vec::new();
            in_cursor.read_to_end(&mut buf).unwrap();
            let (headers, body_bytes) = split_request(&buf);
            let req = feather_runtime::http::Request::parse(headers, body_bytes, ADDR).unwrap();
            let service = EchoService;
            let res = service.handle(req, None).unwrap();
            if let feather_runtime::runtime::service::ServiceResult::Response(response) = res {
                let raw = response.to_raw();
                String::from_utf8_lossy(&raw).to_string()
            } else {
                panic!("Expected Response variant");
            }
        }));
    }

    for (i, h) in handles.into_iter().enumerate() {
        let resp = h.join().unwrap();
        assert!(resp.contains(&format!("Echo: Payload {}", i)));
    }
}
