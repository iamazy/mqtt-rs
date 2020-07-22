mod connection;
pub use connection::Connection;
mod shutdown;
pub use shutdown::Shutdown;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;