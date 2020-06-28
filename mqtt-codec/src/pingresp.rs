use crate::fixed_header::FixedHeader;
use crate::{FromToBuf, Error};
use bytes::{BufMut, BytesMut};
use crate::publish::Qos;
use crate::packet::{PacketType, Packet};

#[derive(Debug, Clone, PartialEq)]
pub struct PingResp {
    fixed_header: FixedHeader
}

impl Packet<PingResp> for PingResp {
    fn from_buf_extra(buf: &mut BytesMut, fixed_header: FixedHeader) -> Result<PingResp, Error> {
        Ok(PingResp {
            fixed_header
        })
    }
}


impl FromToBuf<PingResp> for PingResp {
    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let len = self.fixed_header.to_buf(buf)?;
        Ok(len)
    }

    fn from_buf(buf: &mut BytesMut) -> Result<PingResp, Error> {
        let fixed_header = PingResp::decode_fixed_header(buf);
        assert_eq!(fixed_header.packet_type, PacketType::PINGRESP);
        assert_eq!(fixed_header.dup, false, "The dup of PingResp Fixed Header must be set to false");
        assert_eq!(fixed_header.qos, Qos::AtMostOnce, "The qos of PingResp Fixed Header must be set to be AtMostOnce");
        assert_eq!(fixed_header.retain, false, "The retain of PingResp Fixed Header must be set to false");
        assert_eq!(fixed_header.remaining_length, 0);
        PingResp::from_buf_extra(buf, fixed_header)
    }
}