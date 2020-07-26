use mqtt_core::Connection;
use mqtt_core::{
    codec::{Connect, Packet, PingReq},
    Result,
};
use tokio::net::{TcpStream, ToSocketAddrs};
use tracing::{instrument, debug, error};
use tokio::time::{Duration, delay_for};
use tokio::runtime::Runtime;
use std::borrow::Borrow;


pub struct Client {
    pub shutdown: bool,
    pub connection: Connection,
}

pub async fn connect<T: ToSocketAddrs>(addr: T) -> Result<Client> {
    let socket = TcpStream::connect(addr).await?;
    let connection = Connection::new(socket);
    Ok(Client {
        shutdown: false,
        connection
    })
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

    pub async fn run(&mut self) -> Result<()> {
        while !self.shutdown {
            let packet = match self.connection.read_packet().await? {
                Some(packet) => packet,
                None => return Ok(()),
            };
            debug!("Received packet {:?}", packet);
        }
        Ok(())
    }

    pub async fn ping(&mut self) -> Result<()> {
        loop {
            delay_for(Duration::from_secs(3)).await;
            self.connection.write_packet(&Packet::PingReq(PingReq::default()))
                .await;
        }
    }
}
