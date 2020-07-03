use crate::fixed_header::FixedHeader;
use crate::packet::{PacketId, PacketType, PacketCodec};
use crate::{FromToU8, Error, Mqtt5Property, Frame};
use bytes::{BytesMut, BufMut, Buf};
use crate::publish::Qos;

#[derive(Debug, Clone, PartialEq)]
pub struct PubComp {
    fixed_header: FixedHeader,
    variable_header: PubCompVariableHeader
}

impl PacketCodec<PubComp> for PubComp {
    fn from_buf_extra(buf: &mut BytesMut, fixed_header: FixedHeader) -> Result<PubComp, Error> {
        let variable_header = PubCompVariableHeader::from_buf(buf)
            .expect("Failed to parse PubComp Variable Header");
        Ok(PubComp {
            fixed_header,
            variable_header
        })
    }
}

impl Frame<PubComp> for PubComp {
    fn to_buf(&self, buf: &mut impl BufMut) -> usize {
        let mut len = self.fixed_header.to_buf(buf);
        len += self.variable_header.to_buf(buf);
        len
    }

    fn from_buf(buf: &mut BytesMut) -> Result<PubComp, Error> {
        let fixed_header = PubComp::decode_fixed_header(buf);
        assert_eq!(fixed_header.packet_type, PacketType::PUBCOMP);
        assert_eq!(fixed_header.dup, false, "The dup of PubComp Fixed Header must be set to false");
        assert_eq!(fixed_header.qos, Qos::AtMostOnce, "The qos of PubComp Fixed Header must be set to be AtMostOnce");
        assert_eq!(fixed_header.retain, false, "The retain of PubComp Fixed Header must be set to false");
        PubComp::from_buf_extra(buf, fixed_header)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PubCompVariableHeader {
    packet_id: PacketId,
    pubcomp_reason_code: PubCompReasonCode,
    pubcomp_property: Mqtt5Property,
}

impl PubCompVariableHeader {

    fn check_pubcomp_property(pubcomp_property: &mut Mqtt5Property) -> Result<(), Error> {

        for key in pubcomp_property.properties.keys() {
            let key = *key;
            match key {
                0x1F | 0x26 => {},
                _ => return Err(Error::InvalidPropertyType("PubComp Properties contains a invalid property".to_string()))
            }
        }
        Ok(())
    }
}

impl Frame<PubCompVariableHeader> for PubCompVariableHeader {
    fn to_buf(&self, buf: &mut impl BufMut) -> usize {
        let mut len = self.packet_id.to_buf(buf);
        buf.put_u8(self.pubcomp_reason_code.to_u8());
        len += 1;
        len += self.pubcomp_property.to_buf(buf);
        len
    }

    fn from_buf(buf: &mut BytesMut) -> Result<PubCompVariableHeader, Error> {
        let packet_id = PacketId::new(buf.get_u16());
        let pubcomp_reason_code = PubCompReasonCode::from_u8(buf.get_u8())
            .expect("Failed to parse PubComp Reason Code");
        let mut pubcomp_property = Mqtt5Property::from_buf(buf)
            .expect("Failed to parse PubComp Properties");
        PubCompVariableHeader::check_pubcomp_property(&mut pubcomp_property)?;
        Ok(PubCompVariableHeader {
            packet_id,
            pubcomp_reason_code,
            pubcomp_property
        })

    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PubCompReasonCode {
    /// 0[0x00], The message is accepted. Publication of the QoS 2 message proceeds
    Success,
    /// 146[0x92], The Packet Identifier is not known. This is not an error during recovery,
    /// but at other times indicates a mismatch between the Session State on the Client and Server.
    PacketIdentifierNotFound,
}

impl FromToU8<PubCompReasonCode> for PubCompReasonCode {
    fn to_u8(&self) -> u8 {
        match *self {
            PubCompReasonCode::Success => 0,
            PubCompReasonCode::PacketIdentifierNotFound => 146,
        }
    }

    fn from_u8(byte: u8) -> Result<PubCompReasonCode, Error> {
        match byte {
            0 => Ok(PubCompReasonCode::Success),
            146 => Ok(PubCompReasonCode::PacketIdentifierNotFound),
            n => Err(Error::InvalidReasonCode(n))
        }
    }
}

#[cfg(test)]
mod test {
    use bytes::{BytesMut, BufMut};
    use crate::Frame;
    use crate::pubcomp::PubComp;

    #[test]
    fn test_pubcomp() {
        let pubcomp_bytes = &[
            0b0111_0000u8, 10, // fixed header,
            0x00, 0x10, // packet identifier
            0x00, // pubcomp reason code
            6,
            0x1F,
            0x00, 0x03, 'h' as u8, 'e' as u8, 'l' as u8, // reason string
        ];
        let mut buf = BytesMut::with_capacity(64);
        buf.put_slice(pubcomp_bytes);
        let pubcomp = PubComp::from_buf(&mut buf)
            .expect("Failed to parse PubComp Packet");

        let mut buf = BytesMut::with_capacity(64);
        pubcomp.to_buf(&mut buf);
        assert_eq!(pubcomp, PubComp::from_buf(&mut buf).unwrap());
    }
}