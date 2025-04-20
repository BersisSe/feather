use feather_runtime::{http::HttpResponse, server::server::Server};
fn main() {
    let config = feather_runtime::server::server::ServerConfig{
        address: "127.0.0.1:5000".to_string(),
    };
    let server = Server::new(config);
    
    server.incoming().for_each(|_r|{
        let mut resp = HttpResponse::default();
        resp.send_text("Hello from Feather!");
        return resp;
    }).unwrap();
}
