use crate::frame::FixedHeader;
use crate::{FromToBuf, Error};
use bytes::{BufMut, BytesMut};
use crate::publish::Qos;
use crate::packet::PacketType;

#[derive(Debug, Clone, PartialEq)]
pub struct PingResp {
    fixed_header: FixedHeader
}


impl FromToBuf<PingResp> for PingResp {
    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let len = self.fixed_header.to_buf(buf)?;
        Ok(len)
    }

    fn from_buf(buf: &mut BytesMut) -> Result<PingResp, Error> {
        let fixed_header = FixedHeader::from_buf(buf)
            .expect("Failed to parse PingResp Fixed Header");
        assert_eq!(fixed_header.packet_type, PacketType::PINGRESP);
        assert_eq!(fixed_header.dup, false, "The dup of PingResp Fixed Header must be set to false");
        assert_eq!(fixed_header.qos, Qos::AtMostOnce, "The qos of PingResp Fixed Header must be set to be AtMostOnce");
        assert_eq!(fixed_header.retain, false, "The retain of PingResp Fixed Header must be set to false");
        assert_eq!(fixed_header.remaining_length, 0);
        Ok(PingResp {
            fixed_header
        })
    }
}