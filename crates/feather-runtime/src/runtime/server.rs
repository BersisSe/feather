use log::info;
use may::net::TcpListener;
use num_cpus;
use std::net::SocketAddr;
use std::io;

use crate::runtime::service::BoxService;
struct Server{
    /// The address the server is bound to
    addr : SocketAddr,
    /// The user's application logic
    service: BoxService,
}

impl Server {
    /// Create a new Server instance with the given Service
    pub fn new(addr : SocketAddr, service: BoxService) -> Self {
        Self { addr, service }
    }

    pub fn run(&self) -> io::Result<()> {
        // Setting worker count equal to CPU cores for maximum parallel utilization.
        may::config().set_workers(num_cpus::get());

        let listener = TcpListener::bind(self.addr)?;
        info!("Feather Runtime Started!");

        loop {
            let (stm, host) = listener.accept()?;
        }

        Ok(())
    }
}