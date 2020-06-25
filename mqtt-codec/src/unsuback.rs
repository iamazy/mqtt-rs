use crate::frame::FixedHeader;
use crate::unsubscribe::UnSubscribeReasonCode;
use crate::packet::PacketId;
use crate::{Mqtt5Property, FromToBuf, Error, FromToU8};
use bytes::{BytesMut, BufMut, Buf};
use crate::publish::Qos;

#[derive(Debug, Clone, PartialEq)]
pub struct UnSubAck {
    fixed_header: FixedHeader,
    unsuback_variable_header: UnSubAckVariableHeader,
    payload: Vec<UnSubscribeReasonCode>
}

impl FromToBuf<UnSubAck> for UnSubAck {
    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let mut len = self.fixed_header.to_buf(buf)?;
        len += self.unsuback_variable_header.to_buf(buf)?;
        for reason_code in self.payload.clone() {
            buf.put_u8(reason_code.to_u8());
            len += 1;
        }
        Ok(len)
    }

    fn from_buf(buf: &mut BytesMut) -> Result<UnSubAck, Error> {
        let fixed_header = FixedHeader::new(buf, false, Qos::AtMostOnce, false)
            .expect("Failed to parse UnSubAck Fixed Header");
        let unsuback_variable_header = UnSubAckVariableHeader::from_buf(buf)
            .expect("Failed to parse UnSubAck Variable Header");
        let mut payload_len = fixed_header.remaining_length - 2 - unsuback_variable_header.unsuback_property.property_length;
        let mut payload = Vec::<UnSubscribeReasonCode>::new();
        while payload_len > 0 {
            payload.push(UnSubscribeReasonCode::from_u8(buf.get_u8())?);
            payload_len -= 1;
        }
        Ok(UnSubAck {
            fixed_header,
            unsuback_variable_header,
            payload
        })

    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnSubAckVariableHeader {
    packet_id: PacketId,
    unsuback_property: Mqtt5Property
}

impl FromToBuf<UnSubAckVariableHeader> for UnSubAckVariableHeader {
    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let mut len = self.packet_id.to_buf(buf)?;
        len += self.unsuback_property.to_buf(buf)?;
        Ok(len)
    }

    fn from_buf(buf: &mut BytesMut) -> Result<UnSubAckVariableHeader, Error> {
        let packet_id = PacketId::new(buf.get_u16());
        let unsuback_property = Mqtt5Property::from_buf(buf)
            .expect("Failed to parse UnSubAck Properties");
        Ok(UnSubAckVariableHeader {
            packet_id,
            unsuback_property
        })
    }
}