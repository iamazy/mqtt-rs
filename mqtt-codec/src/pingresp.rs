use crate::fixed_header::FixedHeader;
use crate::{Frame, Error};
use bytes::{BufMut, BytesMut};
use crate::publish::Qos;
use crate::packet::{PacketType, PacketCodec};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PingResp {
    fixed_header: FixedHeader
}

impl PacketCodec<PingResp> for PingResp {
    fn from_buf_extra(_buf: &mut BytesMut, fixed_header: FixedHeader) -> Result<PingResp, Error> {
        Ok(PingResp {
            fixed_header
        })
    }
}


impl Frame<PingResp> for PingResp {
    fn to_buf(&self, buf: &mut impl BufMut) -> usize {
        self.fixed_header.to_buf(buf)
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

    fn length(&self) -> usize {
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    use bytes::{BytesMut, BufMut};
    use crate::Frame;
    use crate::pingresp::PingResp;

    #[test]
    fn test_pingresp() {
        let pingresp_bytes = &[
            0b1101_0000u8, 0  // fixed header
        ];
        let mut buf = BytesMut::with_capacity(64);
        buf.put_slice(pingresp_bytes);
        let pingresp = PingResp::from_buf(&mut buf)
            .expect("Failed to parse PingResp Packet");

        let mut buf = BytesMut::with_capacity(64);
        pingresp.to_buf(&mut buf);
        assert_eq!(pingresp, PingResp::from_buf(&mut buf).unwrap());
    }
}