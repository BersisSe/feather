use crate::http::Request;
use std::io::Error as IoError;

/// A lightweight message type for the server.
pub enum Message {
    Error(IoError),
    Request(Request),
}

impl From<IoError> for Message {
    fn from(value: IoError) -> Self {
        Message::Error(IoError::from(value))
    }
}
