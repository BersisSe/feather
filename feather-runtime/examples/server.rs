use feather_runtime::{http::HttpResponse, server::tcp::Server};

fn main(){
    let mut server = Server::new("127.0.0.1:3500".to_string(), 8);
    println!("Starting http://127.0.0.1:3500");
    server.incoming().for_each(|rq|{
        return HttpResponse::ok("aaaa");
    });
}