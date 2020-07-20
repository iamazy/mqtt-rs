use serde_derive::Deserialize;

mod connection;
mod shutdown;

pub mod broker;
pub mod client;

pub const DEFAULT_PORT: &str = "8888";

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;



#[derive(Debug, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16
}

impl Config {

    pub fn new(file: &str) -> crate::Result<Self> {
        let mut c = config::Config::new();
        c.set_default("host", "127.0.0.1")?;
        c.set_default("port", DEFAULT_PORT)?;

        c.merge(config::File::with_name(file))?;
        c.merge(config::Environment::with_prefix("MQTT"))?;
        Ok(c.try_into()?)
    }
}