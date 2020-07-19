mod connection;
pub mod broker;
mod shutdown;

pub const DEFAULT_PORT: &str = "8888";

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;