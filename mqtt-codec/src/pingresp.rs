use crate::frame::FixedHeader;
use crate::{FromToBuf, Error};
use bytes::{BufMut, BytesMut};
use crate::publish::Qos;

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
        let fixed_header = FixedHeader::new(buf, false, Qos::AtMostOnce, false)
            .expect("Failed to parse PingResp Fixed Header");
        Ok(PingResp {
            fixed_header
        })
    }
}