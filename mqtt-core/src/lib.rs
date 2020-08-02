mod connection;
pub use connection::Connection;
mod shutdown;
use mqtt_codec::Packet;
pub use shutdown::Shutdown;
use tokio::sync::mpsc;

pub mod codec {
    pub use mqtt_codec::*;
}

/// Shorthand for the transmit half of the message channel
type Tx = mpsc::UnboundedSender<Packet>;
/// Shorthand for the receive half of the message channel
type Rx = mpsc::UnboundedReceiver<Packet>;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub type Result<T> = std::result::Result<T, Error>;
