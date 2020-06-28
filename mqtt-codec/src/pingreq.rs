use crate::fixed_header::FixedHeader;
use crate::{FromToBuf, Error};
use bytes::{BytesMut, BufMut};
use crate::publish::Qos;
use crate::packet::{PacketType, Packet};

#[derive(Debug, Clone, PartialEq)]
pub struct PingReq {
    fixed_header: FixedHeader
}

impl Packet<PingReq> for PingReq {
    fn from_buf_extra(_buf: &mut BytesMut, fixed_header: FixedHeader) -> Result<PingReq, Error> {
        Ok(PingReq {
            fixed_header
        })
    }
}

impl FromToBuf<PingReq> for PingReq {
    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let len = self.fixed_header.to_buf(buf)?;
        Ok(len)
    }

    fn from_buf(buf: &mut BytesMut) -> Result<PingReq, Error> {
        let fixed_header = PingReq::decode_fixed_header(buf);
        assert_eq!(fixed_header.packet_type, PacketType::PINGREQ);
        assert_eq!(fixed_header.dup, false, "The dup of PingReq Fixed Header must be set to false");
        assert_eq!(fixed_header.qos, Qos::AtMostOnce, "The qos of PingReq Fixed Header must be set to be AtMostOnce");
        assert_eq!(fixed_header.retain, false, "The retain of PingReq Fixed Header must be set to false");
        assert_eq!(fixed_header.remaining_length, 0);
        PingReq::from_buf_extra(buf, fixed_header)
    }
}