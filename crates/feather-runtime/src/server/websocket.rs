use std::net::TcpStream;
use tungstenite::WebSocket;


pub struct WSRoute{
    pub path: &'static str,
    pub handler: Box<dyn FnMut(WebSocket<TcpStream>) + Send + Sync>
}
