use mqtt_client::client;
use mqtt_core::codec::Error;
use tracing::{debug, error, Level};

#[tokio::main(basic_scheduler)]
async fn main() -> mqtt_core::Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("no global subscriber has been set");

    let mut client = client::connect(&"127.0.0.1:8888").await?;
    tokio::select! {
        res = client.run() => {
            if let Err(err) = res {
                error!(cause = %err, "failed to accept");
            }
        }
    }
    Ok(())
}
