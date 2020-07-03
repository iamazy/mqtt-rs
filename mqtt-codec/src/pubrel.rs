use crate::fixed_header::FixedHeader;
use crate::packet::{PacketId, PacketType, PacketCodec};
use crate::{FromToU8, Error, Mqtt5Property, Frame};
use bytes::{BytesMut, BufMut, Buf};
use crate::publish::Qos;

#[derive(Debug, Clone, PartialEq)]
pub struct PubRel {
    fixed_header: FixedHeader,
    variable_header: PubRelVariableHeader
}

impl PacketCodec<PubRel> for PubRel {
    fn from_buf_extra(buf: &mut BytesMut, fixed_header: FixedHeader) -> Result<PubRel, Error> {
        let variable_header = PubRelVariableHeader::from_buf(buf)
            .expect("Failed to parse PubRel Variable Header");
        Ok(PubRel {
            fixed_header,
            variable_header
        })
    }
}

impl Frame<PubRel> for PubRel {
    fn to_buf(&self, buf: &mut impl BufMut) -> usize {
        let mut len = self.fixed_header.to_buf(buf);
        len += self.variable_header.to_buf(buf);
        len
    }

    fn from_buf(buf: &mut BytesMut) -> Result<PubRel, Error> {
        let fixed_header = PubRel::decode_fixed_header(buf);
        assert_eq!(fixed_header.packet_type, PacketType::PUBREL);
        assert_eq!(fixed_header.dup, false, "The dup of PubRel Fixed Header must be set to false");
        assert_eq!(fixed_header.qos, Qos::AtLeastOnce, "The qos of PubRel Fixed Header must be set to be AtLeastOnce");
        assert_eq!(fixed_header.retain, false, "The retain of PubRel Fixed Header must be set to false");
        PubRel::from_buf_extra(buf, fixed_header)
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

impl Frame<PubRelVariableHeader> for PubRelVariableHeader {
    fn to_buf(&self, buf: &mut impl BufMut) -> usize {
        let mut len = self.packet_id.to_buf(buf);
        buf.put_u8(self.pubrel_reason_code.to_u8());
        len += 1;
        len += self.pubrel_property.to_buf(buf);
        len
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


#[cfg(test)]
mod test {
    use bytes::{BytesMut, BufMut};
    use crate::Frame;
    use crate::pubrel::PubRel;

    #[test]
    fn test_pubrec() {
        let pubrel_bytes = &[
            0b0110_0010u8, 9,  // fixed header
            0x00, 0x10, // packet identifier
            0x00, // pubrec reason code
            5, // properties length
            0x1F, // property id
            0x00, 0x02, 'I' as u8, 'a' as u8, // reason string
        ];
        let mut buf = BytesMut::with_capacity(64);
        buf.put_slice(pubrel_bytes);
        let pubrel = PubRel::from_buf(&mut buf)
            .expect("Failed to parse PubRel Packet");

        let mut buf = BytesMut::with_capacity(64);
        pubrel.to_buf(&mut buf);
        assert_eq!(pubrel, PubRel::from_buf(&mut buf).unwrap());
    }
}
