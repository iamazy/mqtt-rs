use crate::error::Error;
use bytes::{BufMut, BytesMut, Buf};
use crate::{Frame, read_string};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    MQTT5
}

impl Frame<Protocol> for Protocol {
    fn to_buf(&self, buf: &mut impl BufMut) -> usize {
        match self {
            Protocol::MQTT5 => {
                // offset: 0, length: 4, body: MQTT, level: 5
                let slice = &[0u8, 4u8, 'M' as u8, 'Q' as u8, 'T' as u8, 'T' as u8, 5u8];
                buf.put_slice(slice);
                slice.len()
            }
        }
    }

    fn from_buf(buf: &mut BytesMut) -> Result<Protocol, Error> {
        let name = read_string(buf)
            .expect("Failed to parse Protocol Name");
        let name = name.as_ref();
        let level = buf.get_u8();
        match (name, level) {
            ("MQTT", 5u8) => Ok(Protocol::MQTT5),
            _ => Err(Error::InvalidProtocol(name.into(), level))
        }
    }

    fn length(&self) -> usize {
        7
    }
}

impl Default for Protocol {
    fn default() -> Self {
        Protocol::MQTT5
    }
}

#[cfg(test)]
mod test {
    use crate::protocol::Protocol;
    use crate::Frame;

    #[test]
    fn test_protocol() {
        let buf = &mut Vec::<u8>::with_capacity(1024);
        let protocol = Protocol::MQTT5;
        let len = protocol.to_buf(buf);
        println!("len: {}, buf: {:?}",len, buf);
    }
}