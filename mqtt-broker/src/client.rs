use crate::connection::Connection;
use tokio::net::{ToSocketAddrs, TcpStream};
use mqtt_codec::packet::Packet;
use tracing::instrument;

pub struct Client {
    connection: Connection
}

pub async fn connect<T: ToSocketAddrs>(addr: T) -> crate::Result<Client> {
    let socket = TcpStream::connect(addr).await?;
    let connection = Connection::new(socket);
    Ok(Client { connection })
}

impl Client {

    #[instrument(skip(self))]
    pub async fn print(&mut self) -> crate::Result<()>{
        Ok(())
    }
}