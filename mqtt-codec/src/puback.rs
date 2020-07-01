use crate::fixed_header::FixedHeader;
use crate::{Mqtt5Property, FromToU8, Error, FromToBuf};
use bytes::{BytesMut, BufMut, Buf};
use crate::packet::{PacketId, PacketType, Packet};
use crate::publish::Qos;

#[derive(Debug, Clone, PartialEq)]
pub struct PubAck {
    fixed_header: FixedHeader,
    variable_header: PubAckVariableHeader,
}

impl Packet<PubAck> for PubAck {
    fn from_buf_extra(buf: &mut BytesMut, mut fixed_header: FixedHeader) -> Result<PubAck, Error> {
        let variable_header = PubAckVariableHeader::from_buf(buf)
            .expect("Failed to parse PubAck Variable Header");
        Ok(PubAck {
            fixed_header,
            variable_header
        })
    }
}

impl FromToBuf<PubAck> for PubAck {
    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let mut len = self.fixed_header.to_buf(buf)?;
        len += self.variable_header.to_buf(buf)?;
        Ok(len)
    }

    fn from_buf(buf: &mut BytesMut) -> Result<PubAck, Error> {
        let fixed_header = PubAck::decode_fixed_header(buf);
        assert_eq!(fixed_header.packet_type, PacketType::PUBACK);
        assert_eq!(fixed_header.dup, false, "The dup of PubAck Fixed Header must be set to false");
        assert_eq!(fixed_header.qos, Qos::AtMostOnce, "The qos of PubAck Fixed Header must be set to be AtMostOnce");
        assert_eq!(fixed_header.retain, false, "The retain of PubAck Fixed Header must be set to false");
        PubAck::from_buf_extra(buf, fixed_header)
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct PubAckVariableHeader {
    packet_id: PacketId,
    puback_reason_code: PubAckReasonCode,
    puback_property: Mqtt5Property
}

impl PubAckVariableHeader {

    fn check_puback_property(puback_property: &mut Mqtt5Property) -> Result<(), Error> {

        for key in puback_property.properties.keys() {
            let key = *key;
            match key {
                0x1F | 0x26 => {},
                _ => return Err(Error::InvalidPropertyType("PubAck Properties contains a invalid property".to_string()))
            }
        }
        Ok(())
    }

}

impl FromToBuf<PubAckVariableHeader> for PubAckVariableHeader {
    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let mut len = self.packet_id.to_buf(buf)?;
        buf.put_u8(self.puback_reason_code.to_u8());
        len += 1;
        len += self.puback_property.to_buf(buf)?;
        Ok(len)
    }

    fn from_buf(buf: &mut BytesMut) -> Result<PubAckVariableHeader, Error> {
        let packet_id = PacketId::new(buf.get_u16());
        let puback_reason_code = PubAckReasonCode::from_u8(buf.get_u8()).expect("Failed to parse PubAck Reason Code");
        let mut puback_property = Mqtt5Property::from_buf(buf).expect("Failed to parse PubAck Property");
        PubAckVariableHeader::check_puback_property(&mut puback_property)?;
        Ok(PubAckVariableHeader {
            packet_id,
            puback_reason_code,
            puback_property
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PubAckReasonCode {
    /// 0[0x00], The message is accepted. Publication of the QoS 1 message proceeds
    Success,
    /// 16[0x10], The message is accepted but there are no subscribers. This is sent only by the Server.
    /// If the Server knows that there are no matching subscribers, it MAY use this Reason Code instead of 0x00 (Success)
    NoMatchingSubscribers,
    /// 128[0x80], The receiver does not accept the publish but either does not want to reveal the reason,
    /// or it does not match one of the other values
    UnspecifiedError,
    /// 131[0x83], The `PUBLISH` is valid but the receiver is not willing to accept it.
    ImplementationSpecificError,
    /// 135[0x87], The `PUBLISH` is not authorized
    NotAuthorized,
    /// 144[0x90], The Topic Name is not malformed, but is not accepted by this Client or Server
    TopicNameInvalid,
    /// 145[0x91], The Packet Identifier is already in use. This might indicate a mismatch in the Session State between
    /// the Client and Server
    PacketIdentifierInUse,
    /// 151[0x97], An implementation or administrative imposed limit has been exceeded
    QuotaExceeded,
    /// 153[0x99], The payload format does not match the specified Payload Format Indicator
    PayloadFormatInvalid,
}


impl FromToU8<PubAckReasonCode> for PubAckReasonCode {
    fn to_u8(&self) -> u8 {
        match *self {
            PubAckReasonCode::Success => 0,
            PubAckReasonCode::NoMatchingSubscribers => 16,
            PubAckReasonCode::UnspecifiedError => 128,
            PubAckReasonCode::ImplementationSpecificError => 131,
            PubAckReasonCode::NotAuthorized => 135,
            PubAckReasonCode::TopicNameInvalid => 144,
            PubAckReasonCode::PacketIdentifierInUse => 145,
            PubAckReasonCode::QuotaExceeded => 151,
            PubAckReasonCode::PayloadFormatInvalid => 153,
        }
    }

    fn from_u8(byte: u8) -> Result<PubAckReasonCode, Error> {
        match byte {
            0 => Ok(PubAckReasonCode::Success),
            16 => Ok(PubAckReasonCode::NoMatchingSubscribers),
            128 => Ok(PubAckReasonCode::UnspecifiedError),
            131 => Ok(PubAckReasonCode::ImplementationSpecificError),
            135 => Ok(PubAckReasonCode::NotAuthorized),
            144 => Ok(PubAckReasonCode::TopicNameInvalid),
            145 => Ok(PubAckReasonCode::PacketIdentifierInUse),
            151 => Ok(PubAckReasonCode::QuotaExceeded),
            153 => Ok(PubAckReasonCode::PayloadFormatInvalid),
            n => Err(Error::InvalidReasonCode(n))
        }
    }
}
