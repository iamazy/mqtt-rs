use crate::error::Error;
use bytes::BufMut;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    /// [MQTT 5]: https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html
    MQTT5
}

impl Protocol {

    pub(crate) fn new(name: &str, level: u8) -> Result<Protocol, Error> {
        match (name, level) {
            /// I dont know the protocol level of `MQTT5`, set level to 5 temporarily
            ("MQTT5", 5) => Ok(Protocol::MQTT5),
            _ => Err(Error::InvalidProtocol(name.into(), level))
        }
    }

    pub(crate) fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        match self {
            Protocol::MQTT5 => {
                let slice = &[0u8, 4, 'M' as u8, 'Q' as u8, 'T' as u8, 'T' as u8, 4];
                buf.put_slice(slice);
                Ok(slice.len())
            }
        }
    }
}