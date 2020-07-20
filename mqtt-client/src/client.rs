use crate::connection::Connection;
use tokio::net::{ToSocketAddrs, TcpStream};
use mqtt_codec::packet::Packet;
use tracing::instrument;
use bytes::{BytesMut, BufMut};
use mqtt_codec::connect::Connect;
use mqtt_codec::Frame;

pub struct Client {
    connection: Connection
}

pub async fn connect<T: ToSocketAddrs>(addr: T) -> crate::Result<Client> {
    let socket = TcpStream::connect(addr).await?;
    let mut connection = Connection::new(socket);
    Ok(Client { connection })
}

impl Client {

    #[instrument(skip(self))]
    pub async fn connect(&mut self) -> crate::Result<()> {
        // Send Connect Packet to Broker
        let connect_bytes = &[
            0b0001_0000u8, 52,  // fixed header
            0x00, 0x04, 'M' as u8, 'Q' as u8, 'T' as u8, 'T' as u8, // protocol name
            0x05, // protocol version
            0b1100_1110u8, // connect flag
            0x00, 0x10, // keep alive
            0x05, 0x11, 0x00, 0x00, 0x00, 0x10, // connect properties
            0x00, 0x03, 'c' as u8, 'i' as u8, 'd' as u8, // client id
            0x05, 0x02, 0x00, 0x00, 0x00, 0x10, // will properties
            0x00, 0x04, 'w' as u8, 'i' as u8, 'l' as u8, 'l' as u8, // will topic
            0x00, 0x01, 'p' as u8, // will payload
            0x00, 0x06, 'i' as u8, 'a' as u8, 'm' as u8, 'a' as u8, 'z' as u8, 'y' as u8, // username
            0x00, 0x06, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06
        ];
        let mut buf = BytesMut::with_capacity(64);
        buf.put_slice(connect_bytes);
        let connect = Connect::from_buf(&mut buf)
            .expect("Failed to parse Connect Packet");
        self.connection.write_packet(&Packet::Connect(connect)).await?;
        Ok(())
    }
}