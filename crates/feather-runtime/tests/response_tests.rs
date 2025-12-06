use feather_runtime::http::Response;
use serde::Serialize;

#[test]
fn test_response_creation() {
    let mut response = Response::default();
    response.set_status(200);
    response.send_text("Hello World");

    let raw = response.to_raw();
    let raw_str = String::from_utf8_lossy(&raw);
    let raw_lower = raw_str.to_lowercase();

    assert!(raw_str.contains("HTTP/1.1 200 OK"));
    assert!(raw_lower.contains("content-type: text/plain"));
    assert!(raw_lower.contains("content-length: 11"));
    assert!(raw_str.contains("Hello World"));
}

#[test]
fn test_response_with_custom_headers() {
    let mut response = Response::default();
    response.set_status(201);
    response.add_header("X-Custom", "test").unwrap();
    response.send_text("Created");

    let raw = response.to_raw();
    let raw_str = String::from_utf8_lossy(&raw);
    let raw_lower = raw_str.to_lowercase();

    assert!(raw_str.contains("HTTP/1.1 201 Created"));
    assert!(raw_lower.contains("x-custom: test"));
}

#[test]
fn test_json_response() {
    let mut response = Response::default();
    response.set_status(200);

    #[derive(Serialize)]
    struct TestData {
        message: String,
    }

    let data = TestData {
        message: "test".to_string(),
    };

    response.send_json(data);

    let raw = response.to_raw();
    let raw_str = String::from_utf8_lossy(&raw);
    let raw_lower = raw_str.to_lowercase();

    assert!(raw_lower.contains("content-type: application/json"));
    assert!(raw_str.contains(r#"{"message":"test"}"#));
}

#[test]
fn test_error_response() {
    let mut response = Response::default();
    response.set_status(404);
    response.send_text("Not Found");

    let raw = response.to_raw();
    let raw_str = String::from_utf8_lossy(&raw);
    assert!(raw_str.contains("HTTP/1.1 404 Not Found"));
}

#[test]
fn test_response_headers_case_insensitivity() {
    let mut response = Response::default();
    response.add_header("Content-Type", "text/plain").unwrap();
    response.add_header("CONTENT-LENGTH", "5").unwrap();

    let raw = response.to_raw();
    let raw_str = String::from_utf8_lossy(&raw);
    let raw_lower = raw_str.to_lowercase();

    assert!(raw_lower.contains("content-type: text/plain"));
    assert!(raw_lower.contains("content-length: 5"));
}
