use crate::{FromToU8, Error, Mqtt5Property, Frame, write_string, read_string, write_variable_bytes};
use crate::fixed_header::FixedHeader;
use crate::packet::{PacketId, PacketType, PacketCodec};
use bytes::{Bytes, BytesMut, BufMut, Buf};

#[derive(Debug, Clone, PartialEq)]
pub struct Publish {
    fixed_header: FixedHeader,
    variable_header: PublishVariableHeader,
    payload: Bytes,
}

impl Default for Publish {
    fn default() -> Self {
        let variable_header = PublishVariableHeader::default();
        let payload = Bytes::default();
        let fixed_header = FixedHeader {
            packet_type: PacketType::PUBLISH,
            dup: false,
            qos: Qos::AtMostOnce,
            retain: false,
            remaining_length: variable_header.length() + payload.len()
        };
        Publish {
            fixed_header,
            variable_header,
            payload
        }
    }
}

impl PacketCodec<Publish> for Publish {
    fn from_buf_extra(buf: &mut BytesMut, fixed_header: FixedHeader) -> Result<Publish, Error> {
        let variable_header = PublishVariableHeader::from_buf(buf)
            .expect("Failed to parse Publish Variable Header");
        let payload_len = fixed_header.remaining_length
            - variable_header.topic_name.as_bytes().len()
            - 4
            - variable_header.publish_property.property_length
            - write_variable_bytes(variable_header.publish_property.property_length, |_| {});
        let payload = buf.split_to(payload_len).to_bytes();
        Ok(Publish {
            fixed_header,
            variable_header,
            payload,
        })
    }
}

impl Frame<Publish> for Publish {
    fn to_buf(&self, buf: &mut impl BufMut) -> usize {
        let mut len = self.fixed_header.to_buf(buf);
        len += self.variable_header.to_buf(buf);
        buf.put_slice(self.payload.as_ref());
        len += self.payload.len();
        len
    }

    fn from_buf(buf: &mut BytesMut) -> Result<Publish, Error> {
        // If the DUP flag is set to 0, it indicates that this is the first occasion that the Client or Server has attempted to send this PUBLISH packet.
        // If the DUP flag is set to 1, it indicates that this might be re-delivery of an earlier attempt to send the packet
        // The DUP flag MUST be set to 1 by the Client or Server when it attempts to re-deliver a PUBLISH packet.
        // The DUP flag MUST be set to 0 for all QoS 0 messages
        let fixed_header = Publish::decode_fixed_header(buf);
        assert_eq!(fixed_header.packet_type, PacketType::PUBLISH);
        Publish::from_buf_extra(buf, fixed_header)
    }

    fn length(&self) -> usize {
        self.fixed_header.length() + self.fixed_header.remaining_length
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PublishVariableHeader {
    topic_name: String,
    packet_id: PacketId,
    publish_property: Mqtt5Property,
}

impl PublishVariableHeader {

    fn check_publish_property(publish_property: &mut Mqtt5Property) -> Result<(), Error> {

        for key in publish_property.properties.keys() {
            let key = *key;
            match key {
                0x01 | 0x02 | 0x03 | 0x08 | 0x09 | 0x0B | 0x23 | 0x26 => {},
                _ => return Err(Error::InvalidPropertyType("Publish Properties contains a invalid property".to_string()))
            }
        }
        Ok(())
    }
}

impl Frame<PublishVariableHeader> for PublishVariableHeader {
    fn to_buf(&self, buf: &mut impl BufMut) -> usize {
        let mut len = write_string(self.topic_name.clone(), buf);
        len += self.packet_id.to_buf(buf);
        len += self.publish_property.to_buf(buf);
        len
    }

    fn from_buf(buf: &mut BytesMut) -> Result<PublishVariableHeader, Error> {
        let topic_name = read_string(buf).expect("Failed to parse Topic Name");
        let packet_id = PacketId::new(buf.get_u16());
        let mut publish_property = Mqtt5Property::from_buf(buf).expect("Failed to parse Publish Properties");
        PublishVariableHeader::check_publish_property(&mut publish_property)?;
        Ok(PublishVariableHeader {
            topic_name,
            packet_id,
            publish_property,
        })
    }

    fn length(&self) -> usize {
        self.packet_id.length() + self.topic_name.as_bytes().len() + self.publish_property.length()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Qos {
    /// Qos value: 0
    AtMostOnce = 0,
    /// Qos value: 1
    AtLeastOnce = 1,
    /// Qos value: 2
    ExactlyOnce = 2,
    /// Reserved, must not be used
    Reserved = 3
}

impl Default for Qos {
    fn default() -> Self {
        Qos::AtMostOnce
    }
}

impl FromToU8<Qos> for Qos {
    fn to_u8(&self) -> u8 {
        match *self {
            Qos::AtMostOnce => 0,
            Qos::AtLeastOnce => 1,
            Qos::ExactlyOnce => 2,
            _ => 3
        }
    }

    fn from_u8(byte: u8) -> Result<Qos, Error> {
        match byte {
            0 => Ok(Qos::AtMostOnce),
            1 => Ok(Qos::AtLeastOnce),
            2 => Ok(Qos::ExactlyOnce),
            n => Err(Error::InvalidQos(n)),
        }
    }
}

#[cfg(test)]
mod test {
    use bytes::{BytesMut, BufMut};
    use crate::Frame;
    use crate::publish::Publish;

    #[test]
    fn test_publish() {
        let publish_bytes = &[
            0b0011_1101u8, 14,  // fixed header,
            0x00, 0x03, 'c' as u8, 'a' as u8, 't' as u8, // topic name
            0x00, 0x10, // packet identifier
            6,
            0x08,
            0x00, 0x03, 'c' as u8, 'a' as u8, 't' as u8, // reponse topic
        ];
        let mut buf = BytesMut::with_capacity(64);
        buf.put_slice(publish_bytes);
        let publish = Publish::from_buf(&mut buf)
            .expect("Failed to parse Publish Packet");

        let mut buf = BytesMut::with_capacity(64);
        publish.to_buf(&mut buf);
        assert_eq!(publish, Publish::from_buf(&mut buf).unwrap());
    }
}