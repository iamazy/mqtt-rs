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
            /// Protocol Level: https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901033
            ("MQTT", 5u8) => Ok(Protocol::MQTT5),
            _ => Err(Error::InvalidProtocol(name.into(), level))
        }
    }

    pub(crate) fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        match self {
            Protocol::MQTT5 => {
                /// offset: 0, length: 4, body: MQTT, level: 5
                let slice = &[0u8, 4u8, 'M' as u8, 'Q' as u8, 'T' as u8, 'T' as u8, 5u8];
                buf.put_slice(slice);
                Ok(slice.len())
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::protocol::Protocol;

    #[test]
    fn test_protocol() {
        use bytes::BufMut;
        let mut buf = &mut Vec::<u8>::with_capacity(1024);
        let protocol = Protocol::new("MQTT", 5).unwrap();
        let len = protocol.to_buf(buf).unwrap();
        println!("len: {}, buf: {:?}",len, buf);
    }
}