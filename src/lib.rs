mod packet;
mod protocol;
mod connect;
mod error;
pub use error::Error;

/// Packet delivery Qos [Quality of Service] level
///
/// [Qos]: https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901103
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Qos {
    /// Qos value: 0
    AtMostOnce,
    /// Qos value: 1
    AtLeastOnce,
    /// Qos value: 2
    ExactlyOnce,
}

impl Qos {

    pub(crate) fn to_u8(&self) -> u8 {
        match *self {
            Qos::AtMostOnce => 0,
            Qos::AtLeastOnce => 1,
            Qos::ExactlyOnce => 2
        }
    }

    pub(crate) fn from_u8(byte: u8) -> Result<Qos, Error> {
        match byte {
            0 => Ok(Qos::AtMostOnce),
            1 => Ok(Qos::AtLeastOnce),
            2 => Ok(Qos::ExactlyOnce),
            n => Err(Error::InvalidQos(n)),
        }
    }
}