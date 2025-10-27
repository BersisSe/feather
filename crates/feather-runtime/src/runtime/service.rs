use crate::http::{Request, Response};
use std::io;
use may::net::TcpStream;

/// The outcome of the application's request handling.
pub enum ServiceResult {
    /// A standard HTTP response. The Connection Handler will serialize and write this.
    Response(Response),
    /// The Service has taken ownership of the `TcpStream` (e.g., for WebSockets).
    /// The Connection Handler must terminate its loop immediately.
    Consumed,
}

/// The trait representing the user's core application logic.
pub trait Service: Send + Sync + 'static {
    /// Handles an incoming request, receiving the Request and the underlying stream.
    /// The stream is passed as an `Option` to allow the service to consume it for upgrades.
    fn handle(&self, req: Request, stream: Option<TcpStream>) -> io::Result<ServiceResult>;
}

pub type BoxService = Box<dyn Service>;