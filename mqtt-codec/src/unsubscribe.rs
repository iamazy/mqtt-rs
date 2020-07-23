use crate::fixed_header::FixedHeader;
use crate::packet::{PacketCodec, PacketId, PacketType};
use crate::publish::Qos;
use crate::{
    read_string, write_string, write_variable_bytes, Error, Frame, FromToU8, Mqtt5Property,
};
use bytes::{Buf, BufMut, BytesMut};

#[derive(Debug, Clone, PartialEq)]
pub struct UnSubscribe {
    fixed_header: FixedHeader,
    variable_header: UnSubscribeVariableHeader,
    payload: Vec<String>,
}

impl Default for UnSubscribe {
    fn default() -> Self {
        let variable_header = UnSubscribeVariableHeader::default();
        let payload = Vec::<String>::default();
        let mut payload_bytes_len = 0 as usize;
        for item in payload.clone() {
            payload_bytes_len += item.as_bytes().len() + 2
        }
        let fixed_header = FixedHeader {
            packet_type: PacketType::UNSUBACK,
            dup: false,
            qos: Qos::AtMostOnce,
            retain: false,
            remaining_length: variable_header.length() + payload_bytes_len,
        };
        UnSubscribe {
            fixed_header,
            variable_header,
            payload,
        }
    }
}

impl PacketCodec<UnSubscribe> for UnSubscribe {
    fn from_buf_extra(buf: &mut BytesMut, fixed_header: FixedHeader) -> Result<UnSubscribe, Error> {
        let variable_header = UnSubscribeVariableHeader::from_buf(buf)
            .expect("Failed to parse Unsubscribe Variable Header");
        let mut payload_len = fixed_header.remaining_length
            - 2
            - variable_header.unsubscribe_property.property_length
            - write_variable_bytes(variable_header.unsubscribe_property.property_length, |_| {});
        let mut payload = Vec::<String>::new();
        while payload_len > 0 {
            let topic_filter = read_string(buf).expect("Failed to parse Topic Filter");
            payload_len -= topic_filter.as_bytes().len() + 2;
            payload.push(topic_filter);
        }
        Ok(UnSubscribe {
            fixed_header,
            variable_header,
            payload,
        })
    }
}

impl Frame<UnSubscribe> for UnSubscribe {
    fn to_buf(&self, buf: &mut impl BufMut) -> usize {
        let mut len = self.fixed_header.to_buf(buf);
        len += self.variable_header.to_buf(buf);
        for topic_filter in self.payload.clone() {
            len += write_string(topic_filter, buf);
        }
        len
    }

    fn from_buf(buf: &mut BytesMut) -> Result<UnSubscribe, Error> {
        let fixed_header = UnSubscribe::decode_fixed_header(buf);
        assert_eq!(fixed_header.packet_type, PacketType::UNSUBSCRIBE);
        assert_eq!(
            fixed_header.dup, false,
            "The dup of Unsubscribe Fixed Header must be set to false"
        );
        assert_eq!(
            fixed_header.qos,
            Qos::AtLeastOnce,
            "The qos of Unsubscribe Fixed Header must be set to be AtLeastOnce"
        );
        assert_eq!(
            fixed_header.retain, false,
            "The retain of Unsubscribe Fixed Header must be set to false"
        );
        UnSubscribe::from_buf_extra(buf, fixed_header)
    }

    fn length(&self) -> usize {
        self.fixed_header.length() + self.fixed_header.remaining_length
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct UnSubscribeVariableHeader {
    packet_id: PacketId,
    unsubscribe_property: Mqtt5Property,
}

impl UnSubscribeVariableHeader {
    fn check_unsubscribe_property(unsubscribe_property: &mut Mqtt5Property) -> Result<(), Error> {
        for key in unsubscribe_property.properties.keys() {
            let key = *key;
            match key {
                0x26 => {}
                _ => {
                    return Err(Error::InvalidPropertyType(
                        "UnSubscribe Properties contains a invalid property".to_string(),
                    ))
                }
            }
        }
        Ok(())
    }
}

impl Frame<UnSubscribeVariableHeader> for UnSubscribeVariableHeader {
    fn to_buf(&self, buf: &mut impl BufMut) -> usize {
        let mut len = self.packet_id.to_buf(buf);
        len += self.unsubscribe_property.to_buf(buf);
        len
    }

    fn from_buf(buf: &mut BytesMut) -> Result<UnSubscribeVariableHeader, Error> {
        let packet_id = PacketId::new(buf.get_u16());
        let mut unsubscribe_property =
            Mqtt5Property::from_buf(buf).expect("Failed to parse Unsubscribe Properties");
        UnSubscribeVariableHeader::check_unsubscribe_property(&mut unsubscribe_property)?;
        Ok(UnSubscribeVariableHeader {
            packet_id,
            unsubscribe_property,
        })
    }

    fn length(&self) -> usize {
        self.packet_id.length() + self.unsubscribe_property.length()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnSubscribeReasonCode {
    /// 0[0x00], The subscription is deleted.
    Success = 0x00,
    /// 17[0x11], No matching Topic Filter is being used by the Client.
    NoSubscriptionFound = 0x11,
    /// 128[0x80], The unsubscribe could not be completed and the Server
    /// either does not wish to reveal the reason or none of the other Reason Codes apply.
    UnspecifiedError = 0x80,
    /// 131[0x83], The UNSUBSCRIBE is valid but the Server does not accept it.
    ImplementationSpecificError = 0x83,
    /// 135[0x87], The Client is not authorized to unsubscribe.
    NotAuthorized = 0x87,
    /// 143[0x8F], The Topic Filter is correctly formed but is not allowed for this Client.
    TopicFilterInValid = 0x8F,
    /// 145[0x91], The specified Packet Identifier is already in use.
    PacketIdentifierInUse = 0x91,
}

impl Default for UnSubscribeReasonCode {
    fn default() -> Self {
        UnSubscribeReasonCode::UnspecifiedError
    }
}

impl FromToU8<UnSubscribeReasonCode> for UnSubscribeReasonCode {
    fn to_u8(&self) -> u8 {
        match *self {
            UnSubscribeReasonCode::Success => 0,
            UnSubscribeReasonCode::NoSubscriptionFound => 17,
            UnSubscribeReasonCode::UnspecifiedError => 128,
            UnSubscribeReasonCode::ImplementationSpecificError => 131,
            UnSubscribeReasonCode::NotAuthorized => 135,
            UnSubscribeReasonCode::TopicFilterInValid => 143,
            UnSubscribeReasonCode::PacketIdentifierInUse => 145,
        }
    }

    fn from_u8(byte: u8) -> Result<UnSubscribeReasonCode, Error> {
        match byte {
            0 => Ok(UnSubscribeReasonCode::Success),
            17 => Ok(UnSubscribeReasonCode::NoSubscriptionFound),
            128 => Ok(UnSubscribeReasonCode::UnspecifiedError),
            131 => Ok(UnSubscribeReasonCode::ImplementationSpecificError),
            135 => Ok(UnSubscribeReasonCode::NotAuthorized),
            143 => Ok(UnSubscribeReasonCode::TopicFilterInValid),
            145 => Ok(UnSubscribeReasonCode::PacketIdentifierInUse),
            n => Err(Error::InvalidReasonCode(n)),
        }
    }
}

#[test]
fn test_subscribe() {
    let unsubscribe_bytes = &[
        0b1010_0010u8,
        38, // fixed header
        0x00,
        0x10, // packet identifier
        25,   // properties length
        0x26, // property id
        0x00,
        0x04,
        'n' as u8,
        'a' as u8,
        'm' as u8,
        'e' as u8, // user property key1
        0x00,
        0x06,
        'i' as u8,
        'a' as u8,
        'm' as u8,
        'a' as u8,
        'z' as u8,
        'y' as u8, // user property value1
        0x26,
        0x00,
        0x03,
        'a' as u8,
        'g' as u8,
        'e' as u8, // user property key2
        0x00,
        0x02,
        '2' as u8,
        '4' as u8, // user property value2
        0x00,
        0x03,
        'a' as u8,
        '/' as u8,
        'b' as u8, // topic filter
        0x00,
        0x03,
        'c' as u8,
        '/' as u8,
        'd' as u8, // topic filter
    ];
    let mut buf = BytesMut::with_capacity(64);
    buf.put_slice(unsubscribe_bytes);
    let unsubscribe = UnSubscribe::from_buf(&mut buf).expect("Failed to parse UnSubscribe Packet");

    let mut buf = BytesMut::with_capacity(64);
    unsubscribe.to_buf(&mut buf);
    assert_eq!(unsubscribe, UnSubscribe::from_buf(&mut buf).unwrap());
}
