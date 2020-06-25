use crate::{FromToU8, Error, Mqtt5Property, FromToBuf, write_string, read_string};
use crate::frame::FixedHeader;
use crate::packet::PacketId;
use bytes::{Bytes, BytesMut, BufMut, Buf};

#[derive(Debug, Clone, PartialEq)]
pub struct Publish {
    fixed_header: FixedHeader,
    publish_variable_header: PublishVariableHeader,
    payload: Bytes,
}

impl FromToBuf<Publish> for Publish {
    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let mut len = self.fixed_header.to_buf(buf)?;
        len += self.publish_variable_header.to_buf(buf)?;
        buf.put_slice(self.payload.as_ref());
        len += self.payload.len();
        Ok(len)
    }

    fn from_buf(buf: &mut BytesMut) -> Result<Publish, Error> {
        let fixed_header = FixedHeader::new(buf, true, Qos::ExactlyOnce, true)
            .expect("Failed to parse Publish Fixed Header");
        let publish_variable_header = PublishVariableHeader::from_buf(buf)
            .expect("Failed to parse Publish Variable Header");
        let payload_len = fixed_header.remaining_length
            - publish_variable_header.topic_name.len()
            - 2
            - publish_variable_header.publish_property.property_length;
        assert!(payload_len >= 0, "Publish Payload length must greater than 0");
        let payload = buf.split_to(payload_len).to_bytes();
        Ok(Publish {
            fixed_header,
            publish_variable_header,
            payload,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PublishVariableHeader {
    topic_name: String,
    packet_id: PacketId,
    publish_property: Mqtt5Property,
}

impl FromToBuf<PublishVariableHeader> for PublishVariableHeader {
    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let mut len = write_string(self.topic_name.clone(), buf);
        len += self.packet_id.to_buf(buf)?;
        len += self.publish_property.to_buf(buf)?;
        Ok(len)
    }

    fn from_buf(buf: &mut BytesMut) -> Result<PublishVariableHeader, Error> {
        let topic_name = read_string(buf).expect("Failed to parse Topic Name");
        let packet_id = PacketId::new(buf.get_u16());
        let publish_property = Mqtt5Property::from_buf(buf).expect("Failed to parse Publish Properties");
        Ok(PublishVariableHeader {
            topic_name,
            packet_id,
            publish_property,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Qos {
    /// Qos value: 0
    AtMostOnce,
    /// Qos value: 1
    AtLeastOnce,
    /// Qos value: 2
    ExactlyOnce,
}

impl FromToU8<Qos> for Qos {
    fn to_u8(&self) -> u8 {
        match *self {
            Qos::AtMostOnce => 0,
            Qos::AtLeastOnce => 1,
            Qos::ExactlyOnce => 2
        }
    }

    fn from_u8(byte: u8) -> Result<Qos, Error> {
        match byte {
            0 => Ok(Qos::AtMostOnce),
            1 => Ok(Qos::AtLeastOnce),
            2 => Ok(Qos::ExactlyOnce),
            _ => Err(Error::MalformedPacket),
        }
    }
}