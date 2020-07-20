use mqtt_client::client;

#[tokio::main(basic_scheduler)]
async fn main() -> mqtt_client::Result<()> {
    let mut client = client::connect(&"127.0.0.1:8888").await?;
    client.connect().await?;
    Ok(())
}