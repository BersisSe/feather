use crate::http::HttpRequest;
use std::io::Error as IoError;


/// A lightweight message type for the server.
pub enum Message{
    Error(IoError),
    Request(Connection)
}

/// A struct to encapsulate a connection to the server.
/// It contains the stream and the request.

pub struct Connection{
    pub(crate) stream: std::net::TcpStream,
    pub(crate) request: HttpRequest,
}


impl From<IoError> for Message {
    fn from(value: IoError) -> Self {
        Message::Error(IoError::from(value))
    }
    
}