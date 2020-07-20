use mqtt_broker::client;


#[tokio::main(basic_scheduler)]
async fn main() -> mqtt_broker::Result<()> {
    let mut client = client::connect(&"127.0.0.1:8888").await?;
    client.print().await;
    Ok(())
}