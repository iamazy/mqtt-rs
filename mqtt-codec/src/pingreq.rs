use crate::fixed_header::FixedHeader;
use crate::{Frame, Error};
use bytes::{BytesMut, BufMut};
use crate::publish::Qos;
use crate::packet::{PacketType, PacketCodec};

#[derive(Debug, Clone, PartialEq)]
pub struct PingReq {
    pub fixed_header: FixedHeader,
}

impl Default for PingReq {
    fn default() -> Self {
        PingReq {
            fixed_header: FixedHeader {
                packet_type: PacketType::PINGREQ,
                dup: false,
                qos: Qos::AtMostOnce,
                retain: false,
                remaining_length: 0
            }
        }
    }
}

impl PacketCodec<PingReq> for PingReq {
    fn from_buf_extra(_buf: &mut BytesMut, fixed_header: FixedHeader) -> Result<PingReq, Error> {
        Ok(PingReq {
            fixed_header
        })
    }
}

impl Frame<PingReq> for PingReq {
    fn to_buf(&self, buf: &mut impl BufMut) -> usize {
        self.fixed_header.to_buf(buf)
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

    fn length(&self) -> usize {
        self.fixed_header.length() + self.fixed_header.remaining_length
    }
}

#[cfg(test)]
mod test {
    use bytes::{BytesMut, BufMut};
    use crate::pingreq::PingReq;
    use crate::Frame;

    #[test]
    fn test_pingreq() {
        let pingreq_bytes = &[
            0b1100_0000u8, 0  // fixed header
        ];
        let mut buf = BytesMut::with_capacity(64);
        buf.put_slice(pingreq_bytes);
        let pingreq = PingReq::from_buf(&mut buf)
            .expect("Failed to parse PingReq Packet");

        let mut buf = BytesMut::with_capacity(64);
        pingreq.to_buf(&mut buf);
        assert_eq!(pingreq, PingReq::from_buf(&mut buf).unwrap());
    }
}