use tokio::net::TcpListener;
use std::panic;
use mqtt_broker::{broker, Config};
use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version};
use tracing::{info, error, Level};

#[tokio::main]
pub async fn main() -> mqtt_core::Result<()> {
    let opts = app_from_crate!()
        .arg(
            clap::Arg::with_name("config")
                .short("c")
                .long("config")
                .help("Configuration file path")
                .takes_value(true)
                .default_value("mqtt-broker/conf/mqtt.yml")
        ).get_matches();

    let cfg = Config::new(opts.value_of("config").unwrap())?;
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("no global subscriber has been set");
    let listener = TcpListener::bind(&format!("{}:{}",cfg.host, cfg.port)).await?;
    panic::set_hook(Box::new(|panic_info| {
        error!("{:?}", panic_info);
    }));
    info!("mqtt broker started at {}:{}", cfg.host, cfg.port);
    broker::run(listener, tokio::signal::ctrl_c()).await
}