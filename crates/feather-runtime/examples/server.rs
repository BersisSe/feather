use feather_runtime::{http::Response, server::server::Server};

fn main() {
    simple_logger::init_with_level(log::Level::Debug).unwrap();
    
    let mut server = Server::new();
    server.start("127.0.0.1:6060");
   
    server.attach_websocket("/chat", |mut ws|{
        if ws.can_write(){
            ws.send(tungstenite::Message::text("Selam")).unwrap();
        }
    });

    server.incoming().for_each(|_rq| {
        let mut res = Response::default();
        res.send_text("Hi");
        return res;
    }).unwrap();
}