use crate::{FromToU8, Error, Mqtt5Property, FromToBuf, write_string, read_string};
use crate::fixed_header::FixedHeader;
use crate::packet::{PacketId, PacketType, Packet};
use bytes::{Bytes, BytesMut, BufMut, Buf};

#[derive(Debug, Clone, PartialEq)]
pub struct Publish {
    fixed_header: FixedHeader,
    variable_header: PublishVariableHeader,
    payload: Bytes,
}

impl Packet<Publish> for Publish {
    fn from_buf_extra(buf: &mut BytesMut, fixed_header: FixedHeader) -> Result<Publish, Error> {
        let variable_header = PublishVariableHeader::from_buf(buf)
            .expect("Failed to parse Publish Variable Header");
        let payload_len = fixed_header.remaining_length
            - variable_header.topic_name.len()
            - 2
            - variable_header.publish_property.property_length;
        let payload = buf.split_to(payload_len).to_bytes();
        Ok(Publish {
            fixed_header,
            variable_header,
            payload,
        })
    }
}

impl FromToBuf<Publish> for Publish {
    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let mut len = self.fixed_header.to_buf(buf)?;
        len += self.variable_header.to_buf(buf)?;
        buf.put_slice(self.payload.as_ref());
        len += self.payload.len();
        Ok(len)
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
}

#[derive(Debug, Clone, PartialEq)]
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
        let mut publish_property = Mqtt5Property::from_buf(buf).expect("Failed to parse Publish Properties");
        PublishVariableHeader::check_publish_property(&mut publish_property)?;
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
            n => Err(Error::InvalidQos(n)),
        }
    }
}