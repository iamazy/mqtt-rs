use bytes::{BufMut, BytesMut};
use futures::Future;
use mqtt_core::{
    codec::{ConnAck, Frame, Packet, PingResp},
    Connection, Result, Shutdown,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, Semaphore};
use tokio::time::{self, Duration};
use tracing::{debug, error, info, instrument};
use std::borrow::Borrow;

#[derive(Debug)]
struct Listener {
    listener: TcpListener,
    limit_connections: Arc<Semaphore>,
    notify_shutdown: broadcast::Sender<()>,
    shutdown_complete_rx: mpsc::UnboundedReceiver<()>,
    shutdown_complete_tx: mpsc::UnboundedSender<()>,
}

#[derive(Debug)]
struct Handler {
    connection: Connection,
    limit_connections: Arc<Semaphore>,
    shutdown: Shutdown,
    _shutdown_complete: mpsc::UnboundedSender<()>,
}

const MAX_CONNECTIONS: usize = 250;

pub async fn run(listener: TcpListener, shutdown: impl Future) -> Result<()> {
    let (notify_shutdown, _) = broadcast::channel(1);
    let (shutdown_complete_tx, shutdown_complete_rx) = mpsc::unbounded_channel();

    let mut server = Listener {
        listener,
        limit_connections: Arc::new(Semaphore::new(MAX_CONNECTIONS)),
        notify_shutdown,
        shutdown_complete_tx,
        shutdown_complete_rx,
    };

    tokio::select! {
        res = server.run() => {
            if let Err(err) = res {
                error!(cause = %err, "failed to accept");
            }
        }
        _ = shutdown => {
            info!("shutting down");
        }
    }

    let Listener {
        mut shutdown_complete_rx,
        shutdown_complete_tx,
        ..
    } = server;

    drop(shutdown_complete_tx);
    let _ = shutdown_complete_rx.recv().await;
    Ok(())
}

impl Listener {
    async fn run(&mut self) -> Result<()> {
        info!("accepting unbound connections");

        loop {
            self.limit_connections.acquire().await.forget();
            let (socket, addr) = self.accept().await?;
            let mut handler = Handler {
                connection: Connection::new(socket),
                limit_connections: self.limit_connections.clone(),
                shutdown: Shutdown::new(self.notify_shutdown.subscribe()),
                _shutdown_complete: self.shutdown_complete_tx.clone(),
            };

            tokio::spawn(async move {
                if let Err(err) = handler.run().await {
                    error!(cause = ?err, "connection error, address is {}:{}", addr.ip(), addr.port());
                }
            });
        }
    }

    async fn accept(&mut self) -> Result<(TcpStream, SocketAddr)> {
        let mut backoff = 1;
        loop {
            match self.listener.accept().await {
                Ok((socket, addr)) => return Ok((socket, addr)),
                Err(err) => {
                    if backoff > 64 {
                        return Err(err.into());
                    }
                }
            }

            time::delay_for(Duration::from_secs(backoff)).await;
            backoff *= 2;
        }
    }
}

impl Handler {
    #[instrument(skip(self))]
    async fn run(&mut self) -> Result<()> {
        while !self.shutdown.is_shutdown() {
            let maybe_packet = tokio::select! {
                res = self.connection.read_packet() => res?,
                _ = self.shutdown.recv() => {
                    return Ok(())
                }
            };
            let packet = match maybe_packet {
                Some(packet) => packet,
                None => return Ok(()),
            };
            self.process(&packet).await?;
        }
        Ok(())
    }


    async fn process(&mut self, packet: &Packet) -> Result<()> {
        debug!("received packet {:?}", packet);
        match packet.borrow() {
            Packet::Connect(connect) => {
                self.connection
                    .write_packet(&Packet::ConnAck(ConnAck::default()))
                    .await?;
            }
            Packet::PingReq(pingreq) => {
                self.connection
                    .write_packet(&Packet::PingResp(PingResp::default()))
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }
}

impl Drop for Handler {
    fn drop(&mut self) {
        self.limit_connections.add_permits(1);
    }
}
