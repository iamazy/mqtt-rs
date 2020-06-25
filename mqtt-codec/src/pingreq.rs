use crate::frame::FixedHeader;
use crate::{FromToBuf, Error};
use bytes::{BytesMut, BufMut};
use crate::publish::Qos;

#[derive(Debug, Clone, PartialEq)]
pub struct PingReq {
    fixed_header: FixedHeader
}


impl FromToBuf<PingReq> for PingReq {
    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let len = self.fixed_header.to_buf(buf)?;
        Ok(len)
    }

    fn from_buf(buf: &mut BytesMut) -> Result<PingReq, Error> {
        let fixed_header = FixedHeader::new(buf, false, Qos::AtMostOnce, false)
            .expect("Failed to parse PingReq Fixed Header");
        Ok(PingReq {
            fixed_header
        })
    }
}