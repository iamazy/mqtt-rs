use crate::packet::{PacketId, PacketType, PacketCodec};
use crate::{Mqtt5Property, Error, Frame, FromToU8, write_variable_bytes};
use crate::fixed_header::FixedHeader;
use crate::subscribe::SubscribeReasonCode;
use bytes::{BytesMut, BufMut, Buf};
use crate::publish::Qos;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SubAck {
    fixed_header: FixedHeader,
    variable_header: SubAckVariableHeader,
    payload: Vec<SubscribeReasonCode>
}

impl PacketCodec<SubAck> for SubAck {
    fn from_buf_extra(buf: &mut BytesMut, fixed_header: FixedHeader) -> Result<SubAck, Error> {
        let variable_header = SubAckVariableHeader::from_buf(buf)
            .expect("Failed to parse SubAck Variable Header");
        let mut payload_len = fixed_header.remaining_length - 2
            - write_variable_bytes(variable_header.suback_property.property_length, |_|{})
            - variable_header.suback_property.property_length;
        let mut payload = Vec::<SubscribeReasonCode>::new();
        while payload_len > 0 {
            payload.push(SubscribeReasonCode::from_u8(buf.get_u8())?);
            payload_len -= 1;
        }
        Ok(SubAck {
            fixed_header,
            variable_header,
            payload
        })
    }
}

impl Frame<SubAck> for SubAck {

    fn to_buf(&self, buf: &mut impl BufMut) -> usize {
        let mut len = self.fixed_header.to_buf(buf);
        len += self.variable_header.to_buf(buf);
        for reason_code in self.payload.clone() {
            buf.put_u8(reason_code.to_u8());
            len += 1;
        }
        len
    }

    fn from_buf(buf: &mut BytesMut) -> Result<SubAck, Error> {
        let fixed_header = SubAck::decode_fixed_header(buf);
        assert_eq!(fixed_header.packet_type, PacketType::SUBACK);
        assert_eq!(fixed_header.dup, false, "The dup of SubAck Fixed Header must be set to false");
        assert_eq!(fixed_header.qos, Qos::AtMostOnce, "The qos of SubAck Fixed Header must be set to be AtMostOnce");
        assert_eq!(fixed_header.retain, false, "The retain of SubAck Fixed Header must be set to false");
        SubAck::from_buf_extra(buf, fixed_header)
    }

    fn length(&self) -> usize {
        self.fixed_header.length() + self.fixed_header.remaining_length
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SubAckVariableHeader {
    packet_id: PacketId,
    suback_property: Mqtt5Property,
}

impl SubAckVariableHeader {

    fn check_suback_property(suback_property: &mut Mqtt5Property) -> Result<(), Error> {

        for key in suback_property.properties.keys() {
            let key = *key;
            match key {
                0x1F | 0x26 => {},
                _ => return Err(Error::InvalidPropertyType("SubAck Properties contains a invalid property".to_string()))
            }
        }
        Ok(())
    }

}

impl Frame<SubAckVariableHeader> for SubAckVariableHeader {
    fn to_buf(&self, buf: &mut impl BufMut) -> usize {
        let mut len = self.packet_id.to_buf(buf);
        len += self.suback_property.to_buf(buf);
        len
    }

    fn from_buf(buf: &mut BytesMut) -> Result<SubAckVariableHeader, Error> {
        let packet_id = PacketId::new(buf.get_u16());
        let mut suback_property = Mqtt5Property::from_buf(buf)
            .expect("Failed to parse SubAck Properties");
        SubAckVariableHeader::check_suback_property(&mut suback_property)?;
        Ok(SubAckVariableHeader {
            packet_id,
            suback_property
        })
    }

    fn length(&self) -> usize {
        self.packet_id.length() + self.suback_property.length()
    }
}

#[cfg(test)]
mod test {
    use crate::suback::SubAck;
    use bytes::{BytesMut, BufMut};
    use crate::Frame;

    #[test]
    fn test_suback() {
        let suback_bytes = &[
            0b1001_0000u8, 11,  // fixed header
            0x00, 0x10, // packet identifier
            5, // properties length
            0x1F, // property id
            0x00, 0x02, 'I' as u8, 'a' as u8, // reason string
            0x00, 0x01, 0x02
        ];
        let mut buf = BytesMut::with_capacity(64);
        buf.put_slice(suback_bytes);
        let suback = SubAck::from_buf(&mut buf)
            .expect("Failed to parse SubAck Packet");

        let mut buf = BytesMut::with_capacity(64);
        suback.to_buf(&mut buf);
        assert_eq!(suback, SubAck::from_buf(&mut buf).unwrap());
    }
}