mod request;
mod response;
use std::ops::Deref;

pub use request::Request;
pub use response::Response;

#[derive(Debug, Clone)]
pub enum ConnectionState {
    KeepAlive,
    Close,
}

impl ConnectionState {
    pub fn parse(string: &str) -> Option<ConnectionState> {
        let string = string.to_lowercase();
        match string.as_str() {
            "close" => Some(ConnectionState::Close),
            "keep-alive" => Some(ConnectionState::KeepAlive),
            _ => None,
        }
    }
}

impl Deref for ConnectionState {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::KeepAlive => return "keep-alive",
            Self::Close => return "close",
        }
    }
}
