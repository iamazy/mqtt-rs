use serde_derive::Deserialize;

mod connection;
mod shutdown;
pub mod client;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;
