use feather_runtime::{http, server::server::Server};
fn main() {
    env_logger::init();
    let config = feather_runtime::server::server::ServerConfig{
        address: "127.0.0.1:5000".to_string(),
    };
    let server = Server::new(config);
    println!("Server started on {}", server.address());
    server.incoming().for_each(|_r|{
        
        return http::HttpResponse::default()
    }).unwrap();
}
