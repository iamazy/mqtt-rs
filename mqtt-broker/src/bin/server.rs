use tokio::net::TcpListener;
use std::panic;
use mqtt_broker::{broker, DEFAULT_PORT};

#[tokio::main]
pub async fn main() -> mqtt_broker::Result<()> {
    tracing_subscriber::fmt::try_init()?;
    let listener = TcpListener::bind(&format!("127.0.0.1:{}", DEFAULT_PORT)).await?;
    panic::set_hook(Box::new(|panic_info| {
        println!("{:?}", panic_info);
    }));
    broker::run(listener, tokio::signal::ctrl_c()).await
}