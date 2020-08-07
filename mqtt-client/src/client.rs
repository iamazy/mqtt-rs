use mqtt_core::Connection;
use mqtt_core::{
    codec::{Connect, Packet, PingReq},
    Result,
};
use std::sync::Arc;
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::sync::Mutex;
use tokio::time::{delay_for, Duration};
use tracing::{debug, error, instrument};

pub struct Client {
    pub shutdown: bool,
    pub connection: Arc<Mutex<Connection>>,
}

pub async fn connect<T: ToSocketAddrs>(addr: T) -> Result<Client> {
    let socket = TcpStream::connect(addr).await?;
    let connection = Connection::new(socket);
    Ok(Client {
        shutdown: false,
        connection: Arc::new(Mutex::new(connection)),
    })
}

pub async fn run(connection: &mut Arc<Mutex<Connection>>) -> Result<()> {
    for i in 0..10 {
        let connection = connection.clone();
        tokio::spawn(async move {
            loop {
                match connection.lock().await.read_packet().await {
                    Ok(Some(packet)) => {
                        debug!("Received packet {:?}", packet);
                    }
                    _ => {}
                };
                delay_for(Duration::from_secs(3)).await;
                #[rustfmt::skip]
                connection.lock().await.write_packet(&Packet::PingReq(PingReq::default())).await;
            }
        })
        .await;
    }
    Ok(())
}

impl Client {
    #[instrument(skip(self))]
    pub async fn connect(&mut self) -> Result<()> {
        // Send Connect Packet to Broker
        #[rustfmt::skip]
        self.connection.lock().await.write_packet(&Packet::Connect(Connect::default())).await?;
        Ok(())
    }

    pub async fn ping(&mut self) -> Result<()> {
        loop {
            delay_for(Duration::from_secs(3)).await;
            #[rustfmt::skip]
            self.connection.lock().await.write_packet(&Packet::PingReq(PingReq::default())).await;
        }
    }
}
