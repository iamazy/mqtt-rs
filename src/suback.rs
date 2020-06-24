use crate::packet::PacketId;
use crate::{Mqtt5Property, Error, FromToBuf, FromToU8};
use crate::frame::FixedHeader;
use crate::subscribe::SubscribeReasonCode;
use bytes::{BytesMut, BufMut, Buf};
use crate::publish::Qos;

#[derive(Debug, Clone, PartialEq)]
pub struct SubAck {
    fixed_header: FixedHeader,
    suback_variable_header: SubAckVariableHeader,
    payload: Vec<SubscribeReasonCode>
}

impl FromToBuf<SubAck> for SubAck {

    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let mut len = self.fixed_header.to_buf(buf)?;
        len += self.suback_variable_header.to_buf(buf)?;
        let mut payload = self.payload.clone();
        for reason_code in payload {
            buf.put_u8(reason_code.to_u8());
            len += 1;
        }
        Ok(len)
    }

    fn from_buf(buf: &mut BytesMut) -> Result<SubAck, Error> {
        let fixed_header = FixedHeader::new(buf, false, Qos::AtMostOnce, false)
            .expect("Failed to parse SubAck Fixed Header");
        let suback_variable_header = SubAckVariableHeader::from_buf(buf)
            .expect("Failed to parse SubAck Variable Header");
        let mut payload_len = fixed_header.remaining_length - 2 - suback_variable_header.suback_property.property_length;
        let mut payload = Vec::<SubscribeReasonCode>::new();
        while payload_len > 0 {
            payload.push(SubscribeReasonCode::from_u8(buf.get_u8())?);
            payload_len -= 1;
        }
        Ok(SubAck {
            fixed_header,
            suback_variable_header,
            payload
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubAckVariableHeader {
    packet_id: PacketId,
    suback_property: Mqtt5Property,
}


impl FromToBuf<SubAckVariableHeader> for SubAckVariableHeader {
    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let mut len = self.packet_id.to_buf(buf)?;
        len += self.suback_property.to_buf(buf)?;
        Ok(len)
    }

    fn from_buf(buf: &mut BytesMut) -> Result<SubAckVariableHeader, Error> {
        let packet_id = PacketId::new(buf.get_u16());
        let suback_property = Mqtt5Property::from_buf(buf)
            .expect("Failed to parse SubAck Properties");
        Ok(SubAckVariableHeader {
            packet_id,
            suback_property
        })
    }
}

#[cfg(test)]
mod test {
    use bytes::{BytesMut, BufMut};
    use crate::frame::FixedHeader;
    use crate::packet::{PacketType, PacketId};
    use crate::publish::Qos;
    use crate::suback::{SubAckVariableHeader, SubAck};
    use crate::{Mqtt5Property, PropertyValue, FromToBuf};
    use std::collections::HashMap;
    use crate::subscribe::SubscribeReasonCode;
    use crate::decoder::write_variable_bytes;

    #[test]
    fn test_suback() {
        let mut buf = BytesMut::with_capacity(64);

        let fixed_header = FixedHeader {
            packet_type: PacketType::SUBACK,
            dup: false,
            qos: Qos::AtMostOnce,
            retain: false,
            remaining_length: 38
        };

        let mut properties = HashMap::<u32, PropertyValue>::new();
        properties.insert(31, PropertyValue::String("default".to_string()));
        let mut user_property = Vec::<(String, String)>::new();
        user_property.push(("name".to_string(),"iamazy".to_string()));
        user_property.push(("age".to_string(), "13".to_string()));
        properties.insert(38, PropertyValue::StringPair(user_property));

        let len = 35;
        let property = Mqtt5Property {
            properties,
            property_length: len
        };
        let suback_variable_header = SubAckVariableHeader {
            packet_id: PacketId::new(19),
            suback_property: property
        };
        let mut payload = Vec::<SubscribeReasonCode>::new();
        payload.push(SubscribeReasonCode::GrantedQos2);

        let suback = SubAck {
            fixed_header,
            suback_variable_header,
            payload
        };

        let len = suback.to_buf(&mut buf).expect("Failed to turn suback to buf");

        println!("{:?}",buf.to_vec());

        let suback = SubAck::from_buf(&mut buf).expect("Failed to parse SubAck");

        println!("{:?}", suback);

    }
}