use crate::channel::Channel;
use crate::handler::Handler;
use futures::Future;
use mqtt_core::{Connection, Result, Shutdown};
use std::borrow::Borrow;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, Mutex, Semaphore};
use tokio::time::{self, delay_for, Duration, Instant};
use tracing::{error, info};

#[derive(Debug)]
struct Listener {
    listener: TcpListener,
    limit_connections: Arc<Semaphore>,
    notify_shutdown: broadcast::Sender<()>,
    shutdown_complete_rx: mpsc::UnboundedReceiver<()>,
    shutdown_complete_tx: mpsc::UnboundedSender<()>,
}

const MAX_CONNECTIONS: usize = 2;

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
            let handler = Handler {
                shutdown: Shutdown::new(self.notify_shutdown.subscribe()),
                _shutdown_complete: self.shutdown_complete_tx.clone(),
                limit_connections: self.limit_connections.clone(),
                channel: Channel::new(
                    String::default(),
                    addr,
                    Arc::new(Mutex::new(Connection::new(socket))),
                ),
            };
            let handler = Arc::new(Mutex::new(handler));
            let handler2 = handler.clone();
            let run_handler = tokio::spawn(async move {
                if let Err(err) = handler.lock().await.run().await {
                    error!(cause = ?err, "connection error, address is {}:{}", addr.ip(), addr.port());
                }
            });
            let idle_handler = tokio::spawn(async move {
                handler2.lock().await.handle_idle().await;
            });
            futures::join!(run_handler, idle_handler);
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
