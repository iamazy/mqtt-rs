use serde_derive::Deserialize;
pub mod broker;

pub const DEFAULT_PORT: &str = "8888";

#[derive(Debug, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn new(file: &str) -> mqtt_core::Result<Self> {
        let mut c = config::Config::new();
        c.set_default("host", "127.0.0.1")?;
        c.set_default("port", DEFAULT_PORT)?;

        c.merge(config::File::with_name(file))?;
        c.merge(config::Environment::with_prefix("MQTT"))?;
        Ok(c.try_into()?)
    }
}
