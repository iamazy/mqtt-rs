mod connection;
pub use connection::Connection;
mod shutdown;
pub use shutdown::Shutdown;

pub mod codec {
    pub use mqtt_codec::*;
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;