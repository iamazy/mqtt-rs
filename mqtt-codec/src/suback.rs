use crate::packet::{PacketId, PacketType, Packet};
use crate::{Mqtt5Property, Error, FromToBuf, FromToU8};
use crate::fixed_header::FixedHeader;
use crate::subscribe::SubscribeReasonCode;
use bytes::{BytesMut, BufMut, Buf};
use crate::publish::Qos;

#[derive(Debug, Clone, PartialEq)]
pub struct SubAck {
    fixed_header: FixedHeader,
    variable_header: SubAckVariableHeader,
    payload: Vec<SubscribeReasonCode>
}

impl Packet<SubAck> for SubAck {
    fn from_buf_extra(buf: &mut BytesMut, mut fixed_header: FixedHeader) -> Result<SubAck, Error> {
        let variable_header = SubAckVariableHeader::from_buf(buf)
            .expect("Failed to parse SubAck Variable Header");
        let mut payload_len = fixed_header.remaining_length - 2 - variable_header.suback_property.property_length;
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

impl FromToBuf<SubAck> for SubAck {

    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let mut len = self.fixed_header.to_buf(buf)?;
        len += self.variable_header.to_buf(buf)?;
        for reason_code in self.payload.clone() {
            buf.put_u8(reason_code.to_u8());
            len += 1;
        }
        Ok(len)
    }

    fn from_buf(buf: &mut BytesMut) -> Result<SubAck, Error> {
        let fixed_header = SubAck::decode_fixed_header(buf);
        assert_eq!(fixed_header.packet_type, PacketType::SUBACK);
        assert_eq!(fixed_header.dup, false, "The dup of SubAck Fixed Header must be set to false");
        assert_eq!(fixed_header.qos, Qos::AtMostOnce, "The qos of SubAck Fixed Header must be set to be AtMostOnce");
        assert_eq!(fixed_header.retain, false, "The retain of SubAck Fixed Header must be set to false");
        SubAck::from_buf_extra(buf, fixed_header)
    }
}

#[derive(Debug, Clone, PartialEq)]
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

impl FromToBuf<SubAckVariableHeader> for SubAckVariableHeader {
    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let mut len = self.packet_id.to_buf(buf)?;
        len += self.suback_property.to_buf(buf)?;
        Ok(len)
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
}

#[cfg(test)]
mod test {
    use bytes::{BytesMut};
    use crate::fixed_header::FixedHeader;
    use crate::packet::{PacketType, PacketId};
    use crate::publish::Qos;
    use crate::suback::{SubAckVariableHeader, SubAck};
    use crate::{Mqtt5Property, PropertyValue, FromToBuf};
    use std::collections::HashMap;
    use crate::subscribe::SubscribeReasonCode;

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
            variable_header: suback_variable_header,
            payload
        };

        let len = suback.to_buf(&mut buf).expect("Failed to turn suback to buf");

        println!("{:?}",buf.to_vec());

        let suback = SubAck::from_buf(&mut buf).expect("Failed to parse SubAck");

        println!("{:?}", suback);

    }
}