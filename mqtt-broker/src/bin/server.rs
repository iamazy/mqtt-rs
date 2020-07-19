use tokio::net::TcpListener;
use mqtt_broker::{broker, DEFAULT_PORT};

#[tokio::main]
pub async fn main() -> mqtt_broker::Result<()> {
    tracing_subscriber::fmt::try_init()?;

    let listener = TcpListener::bind(&format!("127.0.0.1:{}", DEFAULT_PORT)).await?;
    broker::run(listener, tokio::signal::ctrl_c()).await
}