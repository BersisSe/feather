use feather::middleware::MiddlewareResult;
use feather::{App, AppConfig};
use feather::{Request, Response};
use reqwest::blocking::get;
use std::thread;

#[test]
fn test_app() {
    let config = AppConfig { threads: 4 };
    let mut app = App::with_config(config);

    app.get("/", |_request: &mut Request, response: &mut Response| {
        response.send_text("Hello from Feather!");
        MiddlewareResult::Next
    });
    thread::spawn(move || {
        app.listen("127.0.0.1:3000");
    });

    let response = get("http://127.0.0.1:3000").unwrap();
    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(response.text().unwrap(), "Hello from Feather!");
}
