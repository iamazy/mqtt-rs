use crate::fixed_header::FixedHeader;
use crate::packet::{PacketId, PacketType, PacketCodec};
use crate::{FromToU8, Error, Mqtt5Property, Frame};
use bytes::{BytesMut, BufMut, Buf};
use crate::publish::Qos;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PubRec {
    fixed_header: FixedHeader,
    variable_header: PubRecVariableHeader
}

impl PacketCodec<PubRec> for PubRec {
    fn from_buf_extra(buf: &mut BytesMut, fixed_header: FixedHeader) -> Result<PubRec, Error> {
        let variable_header = PubRecVariableHeader::from_buf(buf)
            .expect("Failed to parse PubRec Variable Header");
        Ok(PubRec {
            fixed_header,
            variable_header
        })
    }
}

impl Frame<PubRec> for PubRec {
    fn to_buf(&self, buf: &mut impl BufMut) -> usize {
        let mut len = self.fixed_header.to_buf(buf);
        len += self.variable_header.to_buf(buf);
        len
    }

    fn from_buf(buf: &mut BytesMut) -> Result<PubRec, Error> {
        let fixed_header =PubRec::decode_fixed_header(buf);
        assert_eq!(fixed_header.packet_type, PacketType::PUBREC);
        assert_eq!(fixed_header.dup, false, "The dup of PubRec Fixed Header must be set to false");
        assert_eq!(fixed_header.qos, Qos::AtMostOnce, "The qos of PubRec Fixed Header must be set to be AtMostOnce");
        assert_eq!(fixed_header.retain, false, "The retain of PubRec Fixed Header must be set to false");
        PubRec::from_buf_extra(buf, fixed_header)
    }

    fn length(&self) -> usize {
        unimplemented!()
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PubRecVariableHeader {
    packet_id: PacketId,
    pubrec_reason_code: PubRecReasonCode,
    pubrec_property: Mqtt5Property,
}

impl PubRecVariableHeader {

    fn check_pubrec_property(pubrec_property: &mut Mqtt5Property) -> Result<(), Error> {

        for key in pubrec_property.properties.keys() {
            let key = *key;
            match key {
                0x1F | 0x26 => {},
                _ => return Err(Error::InvalidPropertyType("PubRec Properties contains a invalid property".to_string()))
            }
        }
        Ok(())
    }

}

impl Frame<PubRecVariableHeader> for PubRecVariableHeader {
    fn to_buf(&self, buf: &mut impl BufMut) -> usize {
        let mut len = self.packet_id.to_buf(buf);
        buf.put_u8(self.pubrec_reason_code.to_u8());
        len += 1;
        len += self.pubrec_property.to_buf(buf);
        len
    }

    fn from_buf(buf: &mut BytesMut) -> Result<PubRecVariableHeader, Error> {
        let packet_id = PacketId::new(buf.get_u16());
        let pubrec_reason_code = PubRecReasonCode::from_u8(buf.get_u8())
            .expect("Failed to parse PubRec Reason Code");
        let mut pubrec_property = Mqtt5Property::from_buf(buf)
            .expect("Failed to parse PubRec Properties");
        PubRecVariableHeader::check_pubrec_property(&mut pubrec_property)?;
        Ok(PubRecVariableHeader {
            packet_id,
            pubrec_reason_code,
            pubrec_property
        })

    }

    fn length(&self) -> usize {
        unimplemented!()
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PubRecReasonCode {
    /// 0[0x00], The message is accepted. Publication of the QoS 2 message proceeds
    Success = 0x00,
    /// 16[0x10], The message is accepted but there are no subscribers. This is sent only by the Server.
    /// If the Server knows that there are no matching subscribers, it MAY use this Reason Code instead of 0x00 (Success)
    NoMatchingSubscribers = 0x10,
    /// 128[0x80], The receiver does not accept the publish but either does not want to reveal the reason,
    /// or it does not match one of the other values
    UnspecifiedError = 0x80,
    /// 131[0x83], The PUBLISH is valid but the receiver is not willing to accept it
    ImplementationSpecificError = 0x83,
    /// 135[0x87], The PUBLISH is not authorized
    NotAuthorized = 0x87,
    /// 144[0x90], The Topic Name is not malformed, but is not accepted by this Client or Server
    TopicNameInvalid = 0x90,
    /// 145[0x91], The Packet Identifier is already in use. This might indicate a mismatch in the
    /// Session State between the Client and Server.
    PacketIdentifierInUse = 0x91,
    /// 151[0x97], An implementation or administrative imposed limit has been exceeded
    QuotaExceeded = 0x97,
    /// 153[0x99], The payload format does not match the one specified in the Payload Format Indicator
    PayloadFormatInvalid = 0x99,
}

impl Default for PubRecReasonCode {
    fn default() -> Self {
        PubRecReasonCode::UnspecifiedError
    }
}

impl FromToU8<PubRecReasonCode> for PubRecReasonCode {
    fn to_u8(&self) -> u8 {
        match *self {
            PubRecReasonCode::Success => 0,
            PubRecReasonCode::NoMatchingSubscribers => 16,
            PubRecReasonCode::UnspecifiedError => 128,
            PubRecReasonCode::ImplementationSpecificError => 131,
            PubRecReasonCode::NotAuthorized => 135,
            PubRecReasonCode::TopicNameInvalid => 144,
            PubRecReasonCode::PacketIdentifierInUse => 145,
            PubRecReasonCode::QuotaExceeded => 151,
            PubRecReasonCode::PayloadFormatInvalid => 153
        }
    }

    fn from_u8(byte: u8) -> Result<PubRecReasonCode, Error> {
        match byte {
            0 => Ok(PubRecReasonCode::Success),
            16 => Ok(PubRecReasonCode::NoMatchingSubscribers),
            128 => Ok(PubRecReasonCode::UnspecifiedError),
            131 => Ok(PubRecReasonCode::ImplementationSpecificError),
            135 => Ok(PubRecReasonCode::NotAuthorized),
            144 => Ok(PubRecReasonCode::TopicNameInvalid),
            145 => Ok(PubRecReasonCode::PacketIdentifierInUse),
            151 => Ok(PubRecReasonCode::QuotaExceeded),
            153 => Ok(PubRecReasonCode::PayloadFormatInvalid),
            n => Err(Error::InvalidReasonCode(n))
        }
    }
}

#[cfg(test)]
mod test {
    use bytes::{BytesMut, BufMut};
    use crate::Frame;
    use crate::pubrec::PubRec;

    #[test]
    fn test_pubrec() {
        let pubrec_bytes = &[
            0b0101_0000u8, 9,  // fixed header
            0x00, 0x10, // packet identifier
            0x00, // pubrec reason code
            5, // properties length
            0x1F, // property id
            0x00, 0x02, 'I' as u8, 'a' as u8, // reason string
        ];
        let mut buf = BytesMut::with_capacity(64);
        buf.put_slice(pubrec_bytes);
        let pubrec = PubRec::from_buf(&mut buf)
            .expect("Failed to parse PubRec Packet");

        let mut buf = BytesMut::with_capacity(64);
        pubrec.to_buf(&mut buf);
        assert_eq!(pubrec, PubRec::from_buf(&mut buf).unwrap());
    }
}