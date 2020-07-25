use bytes::{BufMut, BytesMut};
use mqtt_core::Connection;
use mqtt_core::{
    codec::{Connect, Frame, Packet},
    Result,
};
use tokio::net::{TcpStream, ToSocketAddrs};
use tracing::instrument;

pub struct Client {
    pub connection: Connection,
}

pub async fn connect<T: ToSocketAddrs>(addr: T) -> Result<Client> {
    let socket = TcpStream::connect(addr).await?;
    let connection = Connection::new(socket);
    Ok(Client { connection })
}

impl Client {
    #[instrument(skip(self))]
    pub async fn connect(&mut self) -> Result<()> {
        // Send Connect Packet to Broker
        self.connection
            .write_packet(&Packet::Connect(Connect::default()))
            .await?;
        Ok(())
    }
}
