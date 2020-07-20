use mqtt_client::client;
use mqtt_codec::Error;
use tracing::{debug, Level, error};

#[tokio::main(basic_scheduler)]
async fn main() -> mqtt_client::Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("no global subscriber has been set");

    let mut client = client::connect(&"127.0.0.1:8888").await?;
    client.connect().await?;

    match client.connection.read_packet().await {
        Ok(None) | Err(Error::Incomplete) => {}
        Ok(Some(packet)) => debug!("received packet: {:?}", packet),
        Err(e) => error!("cause error: {:?}", e)
    }
    Ok(())
}