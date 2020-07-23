use crate::fixed_header::FixedHeader;
use crate::unsubscribe::UnSubscribeReasonCode;
use crate::packet::{PacketId, PacketType, PacketCodec};
use crate::{Mqtt5Property, Frame, Error, FromToU8, write_variable_bytes};
use bytes::{BytesMut, BufMut, Buf};
use crate::publish::Qos;

#[derive(Debug, Clone, PartialEq)]
pub struct UnSubAck {
    fixed_header: FixedHeader,
    variable_header: UnSubAckVariableHeader,
    payload: Vec<UnSubscribeReasonCode>
}

impl Default for UnSubAck {
    fn default() -> Self {
        let variable_header = UnSubAckVariableHeader::default();
        let payload = Vec::default();
        let fixed_header = FixedHeader {
            packet_type: PacketType::UNSUBACK,
            dup: false,
            qos: Qos::AtMostOnce,
            retain: false,
            remaining_length: variable_header.length() + payload.len()
        };
        UnSubAck {
            fixed_header,
            variable_header,
            payload
        }
    }
}

impl PacketCodec<UnSubAck> for UnSubAck {
    fn from_buf_extra(buf: &mut BytesMut, fixed_header: FixedHeader) -> Result<UnSubAck, Error> {
        let variable_header = UnSubAckVariableHeader::from_buf(buf)
            .expect("Failed to parse UnSubAck Variable Header");
        let mut payload_len = fixed_header.remaining_length
            - 2
            - variable_header.unsuback_property.property_length
            - write_variable_bytes(variable_header.unsuback_property.property_length, |_| {});
        let mut payload = Vec::<UnSubscribeReasonCode>::new();
        while payload_len > 0 {
            payload.push(UnSubscribeReasonCode::from_u8(buf.get_u8())?);
            payload_len -= 1;
        }
        Ok(UnSubAck {
            fixed_header,
            variable_header,
            payload
        })
    }
}

impl Frame<UnSubAck> for UnSubAck {
    fn to_buf(&self, buf: &mut impl BufMut) -> usize {
        let mut len = self.fixed_header.to_buf(buf);
        len += self.variable_header.to_buf(buf);
        for reason_code in self.payload.clone() {
            buf.put_u8(reason_code.to_u8());
            len += 1;
        }
        len
    }

    fn from_buf(buf: &mut BytesMut) -> Result<UnSubAck, Error> {
        let fixed_header = UnSubAck::decode_fixed_header(buf);
        assert_eq!(fixed_header.packet_type, PacketType::UNSUBACK);
        assert_eq!(fixed_header.dup, false, "The dup of UnSubAck Fixed Header must be set to false");
        assert_eq!(fixed_header.qos, Qos::AtMostOnce, "The qos of UnSubAck Fixed Header must be set to be AtMostOnce");
        assert_eq!(fixed_header.retain, false, "The retain of UnSubAck Fixed Header must be set to false");
        UnSubAck::from_buf_extra(buf, fixed_header)
    }

    fn length(&self) -> usize {
        self.fixed_header.length() + self.fixed_header.remaining_length
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct UnSubAckVariableHeader {
    packet_id: PacketId,
    unsuback_property: Mqtt5Property
}

impl UnSubAckVariableHeader {

    fn check_unsuback_property(unsuback_property: &mut Mqtt5Property) -> Result<(), Error> {

        for key in unsuback_property.properties.keys() {
            let key = *key;
            match key {
                0x1F | 0x26 => {},
                _ => return Err(Error::InvalidPropertyType("UnSubAck Properties contains a invalid property".to_string()))
            }
        }
        Ok(())
    }
}

impl Frame<UnSubAckVariableHeader> for UnSubAckVariableHeader {
    fn to_buf(&self, buf: &mut impl BufMut) -> usize {
        let mut len = self.packet_id.to_buf(buf);
        len += self.unsuback_property.to_buf(buf);
        len
    }

    fn from_buf(buf: &mut BytesMut) -> Result<UnSubAckVariableHeader, Error> {
        let packet_id = PacketId::new(buf.get_u16());
        let mut unsuback_property = Mqtt5Property::from_buf(buf)
            .expect("Failed to parse UnSubAck Properties");
        UnSubAckVariableHeader::check_unsuback_property(&mut unsuback_property)?;
        Ok(UnSubAckVariableHeader {
            packet_id,
            unsuback_property
        })
    }

    fn length(&self) -> usize {
        self.packet_id.length() + self.unsuback_property.length()
    }
}


#[cfg(test)]
mod test {
    use bytes::{BytesMut, BufMut};
    use crate::Frame;
    use crate::unsuback::UnSubAck;

    #[test]
    fn test_unsuback() {
        let unsuback_bytes = &[
            0b1011_0000u8, 11,  // fixed header
            0x00, 0x10, // packet identifier
            5, // properties length
            0x1F, // property id
            0x00, 0x02, 'I' as u8, 'a' as u8, // reason string
            0x00, 0x11, 0x80
        ];
        let mut buf = BytesMut::with_capacity(64);
        buf.put_slice(unsuback_bytes);
        let unsuback = UnSubAck::from_buf(&mut buf)
            .expect("Failed to parse UnSubAck Packet");

        let mut buf = BytesMut::with_capacity(64);
        unsuback.to_buf(&mut buf);
        assert_eq!(unsuback, UnSubAck::from_buf(&mut buf).unwrap());
    }
}