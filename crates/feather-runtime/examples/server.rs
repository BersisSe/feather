use feather_runtime::{http, server::server::Server};

fn main() {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
    let config = feather_runtime::server::server::ServerConfig {
        address: "127.0.0.1:5000".to_string(),
    };
    let server = Server::new(config);
    println!("Server started on {}", server.address());
    server
        .incoming()
        .for_each(|_r| return http::Response::default())
        .unwrap();
}
