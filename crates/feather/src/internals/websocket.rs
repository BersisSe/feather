use std::{collections::HashMap, error::Error, io, net::TcpStream};
use uuid::Uuid;
use feather_runtime::{Message, TungsteniteErr, WebSocket};// Just Tungtinite WebSocket

pub struct Socket {
    pub(crate )path: &'static str,
    clients: Vec<WsClient>,
    handlers: Vec<Box<dyn Fn(&mut WsClient, String) + Send + Sync>>,
    on_open: Option<Box<dyn Fn(&mut WsClient) + Send + Sync>>,
    on_close: Option<Box<dyn Fn(&mut WsClient) + Send + Sync>>,
}
impl Socket {
    /// Creates a new empty socket object
    pub(crate) fn new(path:&'static str,) -> Self{
        Self { 
            path,
            clients: Vec::new(),
            handlers: Vec::new(),
            on_open: None,
            on_close: None 
        }
    }
    pub(crate) fn add_client(&mut self, client: WsClient){
        self.clients.push(client);
    } 
    /// Removes a client by its Uuid
    pub(crate) fn remove_client(&mut self, id: &Uuid) {
        self.clients.retain(|c| c.id() != id);
    }
    pub fn run(&mut self) {
       
            let mut disconnected = Vec::new();
            for client in &mut self.clients {
                match client.read() {
                    Ok(msg) => {
                        let msg_str = format!("{:?}", msg); // Or extract text if it's a text message
                        for handler in &self.handlers {
                            handler(client, msg_str.clone());
                        }
                    }
                    Err(_) => {
                        disconnected.push(client.id().clone());
                    }
                }
            }
            for id in disconnected {
                self.remove_client(&id);
                if let Some(ref on_close) = self.on_close {
                    // Find the client if you want to pass it, or just call the handler
                }
            }
            // Sleep or yield to avoid busy loop
            std::thread::sleep(std::time::Duration::from_millis(10));
        
    }
    /// Returns the number of clients are currently connected
    pub fn clients(&self) -> usize{
        self.clients.len()
    }
    pub fn on_message<H>(&mut self,handler:H)
    where H: Fn(&mut WsClient, String) + Send + Sync + 'static
    {
        self.handlers.push(Box::new(handler));
    }
    pub fn on_init<H>(&mut self, handler:H)
    where H: Fn(&mut WsClient) + Send + Sync +  'static
    {
        self.on_open = Some(Box::new(handler))
    }
    pub fn on_close<H>(&mut self, handler:H)
    where H: Fn(&mut WsClient) + Send + Sync +  'static
    {
        self.on_close = Some(Box::new(handler))
    }
}

pub struct WsClient {
    id: Uuid,
    inner: WebSocket<TcpStream>
}
impl WsClient{
    pub(crate) fn new(stream: WebSocket<TcpStream>) -> Self{
        WsClient { 
            id: Uuid::new_v4(),
            inner: stream 
        }
    }
    pub fn id(&self) -> &Uuid{
        &self.id
    }
    pub fn read(&mut self) -> Result<Message,TungsteniteErr>{
        self.inner.read()
    }
    pub fn send(&mut self, msg: &str) -> io::Result<()>{
        if self.inner.can_write(){
            match self.inner.send(msg.into()) {
                Ok(_) =>{
                    return Ok(());
                }
                Err(e) =>{
                    return Err(io::Error::new(io::ErrorKind::Other, e));
                }
            }
        }
        else{
            return Err(io::Error::new(io::ErrorKind::Other, "Could not Write to the Socket"));
        }
    }
}

