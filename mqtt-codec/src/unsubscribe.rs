use crate::packet::{PacketId, PacketType};
use crate::{Mqtt5Property, FromToBuf, Error, FromToU8, write_string, read_string};
use crate::frame::FixedHeader;
use bytes::{BytesMut, BufMut, Buf};
use crate::publish::Qos;

#[derive(Debug, Clone, PartialEq)]
pub struct UnSubscribe {
    fixed_header: FixedHeader,
    unsubscribe_variable_header: UnSubscribeVariableHeader,
    payload: Vec<String>
}

impl FromToBuf<UnSubscribe> for UnSubscribe {

    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let mut len = self.fixed_header.to_buf(buf)?;
        len += self.unsubscribe_variable_header.to_buf(buf)?;
        for topic_filter in self.payload.clone() {
            len += write_string(topic_filter, buf);
        }
        Ok(len)
    }

    fn from_buf(buf: &mut BytesMut) -> Result<UnSubscribe, Error> {
        let fixed_header = FixedHeader::from_buf(buf)
            .expect("Failed to parse Unsubscribe Fixed Header");
        assert_eq!(fixed_header.packet_type, PacketType::UNSUBSCRIBE);
        assert_eq!(fixed_header.dup, false, "The dup of Unsubscribe Fixed Header must be set to false");
        assert_eq!(fixed_header.qos, Qos::AtLeastOnce, "The qos of Unsubscribe Fixed Header must be set to be AtLeastOnce");
        assert_eq!(fixed_header.retain, false, "The retain of Unsubscribe Fixed Header must be set to false");
        let unsubscribe_variable_header = UnSubscribeVariableHeader::from_buf(buf)
            .expect("Failed to parse Unsubscribe Variable Header");
        let mut payload_len = fixed_header.remaining_length - 2 - unsubscribe_variable_header.unsubscribe_property.property_length;
        let mut payload = Vec::<String>::new();
        while payload_len > 0 {
            let topic_filter = read_string(buf)
                .expect("Failed to parse Topic Filter");
            payload_len -= 1;
            payload.push(topic_filter);
        }
        Ok(UnSubscribe {
            fixed_header,
            unsubscribe_variable_header,
            payload
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnSubscribeVariableHeader {
    packet_id: PacketId,
    unsubscribe_property: Mqtt5Property
}

impl UnSubscribeVariableHeader {

    fn check_unsubscribe_property(unsubscribe_property: &mut Mqtt5Property) -> Result<(), Error> {

        for key in unsubscribe_property.properties.keys() {
            let key = *key;
            match key {
                0x26 => {},
                _ => return Err(Error::InvalidPropertyType("UnSubscribe Properties contains a invalid property".to_string()))
            }
        }
        Ok(())
    }
}

impl FromToBuf<UnSubscribeVariableHeader> for UnSubscribeVariableHeader {
    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let mut len = self.packet_id.to_buf(buf)?;
        len += self.unsubscribe_property.to_buf(buf)?;
        Ok(len)
    }

    fn from_buf(buf: &mut BytesMut) -> Result<UnSubscribeVariableHeader, Error> {
        let packet_id = PacketId::new(buf.get_u16());
        let mut unsubscribe_property = Mqtt5Property::from_buf(buf)
            .expect("Failed to parse Unsubscribe Properties");
        UnSubscribeVariableHeader::check_unsubscribe_property(&mut unsubscribe_property)?;
        Ok(UnSubscribeVariableHeader {
            packet_id,
            unsubscribe_property
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnSubscribeReasonCode {
    /// 0[0x00], The subscription is deleted.
    Success,
    /// 17[0x11], No matching Topic Filter is being used by the Client.
    NoSubscriptionFound,
    /// 128[0x80], The unsubscribe could not be completed and the Server
    /// either does not wish to reveal the reason or none of the other Reason Codes apply.
    UnspecifiedError,
    /// 131[0x87], The UNSUBSCRIBE is valid but the Server does not accept it.
    ImplementationSpecificError,
    /// 135[0x87], The Client is not authorized to unsubscribe.
    NotAuthorized,
    /// 143[0X8F], The Topic Filter is correctly formed but is not allowed for this Client.
    TopicFilterInValid,
    /// 145[0X91], The specified Packet Identifier is already in use.
    PacketIdentifierInUse
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
            UnSubscribeReasonCode::PacketIdentifierInUse => 145
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
            n => Err(Error::InvalidReasonCode(n))
        }

    }
}