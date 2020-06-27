use crate::fixed_header::FixedHeader;
use crate::packet::{PacketId, PacketType};
use crate::{FromToU8, Error, Mqtt5Property, FromToBuf};
use bytes::{BytesMut, BufMut, Buf};
use crate::publish::Qos;

#[derive(Debug, Clone, PartialEq)]
pub struct PubRel {
    fixed_header: FixedHeader,
    variable_header: PubRelVariableHeader
}

impl FromToBuf<PubRel> for PubRel {
    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let mut len = self.fixed_header.to_buf(buf)?;
        len += self.variable_header.to_buf(buf)?;
        Ok(len)
    }

    fn from_buf(buf: &mut BytesMut) -> Result<PubRel, Error> {
        let fixed_header = FixedHeader::from_buf(buf)
            .expect("Failed to parse PubRel Fixed Header");
        assert_eq!(fixed_header.packet_type, PacketType::PUBREL);
        assert_eq!(fixed_header.dup, false, "The dup of PubRel Fixed Header must be set to false");
        assert_eq!(fixed_header.qos, Qos::AtLeastOnce, "The qos of PubRel Fixed Header must be set to be AtLeastOnce");
        assert_eq!(fixed_header.retain, false, "The retain of PubRel Fixed Header must be set to false");
        let pubrel_variable_header = PubRelVariableHeader::from_buf(buf)
            .expect("Failed to parse PubRel Variable Header");
        Ok(PubRel {
            fixed_header,
            variable_header: pubrel_variable_header
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PubRelVariableHeader {
    packet_id: PacketId,
    pubrel_reason_code: PubRelReasonCode,
    pubrel_property: Mqtt5Property,
}

impl PubRelVariableHeader {

    fn check_pubrel_property(pubrel_property: &mut Mqtt5Property) -> Result<(), Error> {

        for key in pubrel_property.properties.keys() {
            let key = *key;
            match key {
                0x1F | 0x26 => {},
                _ => return Err(Error::InvalidPropertyType("PubRel Properties contains a invalid property".to_string()))
            }
        }
        Ok(())
    }
}

impl FromToBuf<PubRelVariableHeader> for PubRelVariableHeader {
    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let mut len = self.packet_id.to_buf(buf)?;
        buf.put_u8(self.pubrel_reason_code.to_u8());
        len += 1;
        len += self.pubrel_property.to_buf(buf)?;
        Ok(len)
    }

    fn from_buf(buf: &mut BytesMut) -> Result<PubRelVariableHeader, Error> {
        let packet_id = PacketId::new(buf.get_u16());
        let pubrel_reason_code = PubRelReasonCode::from_u8(buf.get_u8())
            .expect("Failed to parse PubRel Reason Code");
        let mut pubrel_property = Mqtt5Property::from_buf(buf)
            .expect("Failed to parse PubRel Properties");
        PubRelVariableHeader::check_pubrel_property(&mut pubrel_property)?;
        Ok(PubRelVariableHeader {
            packet_id,
            pubrel_reason_code,
            pubrel_property
        })

    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PubRelReasonCode {
    /// 0[0x00], The message is accepted. Publication of the QoS 2 message proceeds
    Success,
    /// 146[0x92], The Packet Identifier is not known. This is not an error during recovery,
    /// but at other times indicates a mismatch between the Session State on the Client and Server.
    PacketIdentifierNotFound,
}

impl FromToU8<PubRelReasonCode> for PubRelReasonCode {
    fn to_u8(&self) -> u8 {
        match *self {
            PubRelReasonCode::Success => 0,
            PubRelReasonCode::PacketIdentifierNotFound => 146,
        }
    }

    fn from_u8(byte: u8) -> Result<PubRelReasonCode, Error> {
        match byte {
            0 => Ok(PubRelReasonCode::Success),
            146 => Ok(PubRelReasonCode::PacketIdentifierNotFound),
            n => Err(Error::InvalidReasonCode(n))
        }
    }
}
