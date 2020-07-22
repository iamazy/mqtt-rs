use crate::packet::PacketType;
use crate::publish::Qos;
use crate::{Frame, Error, FromToU8, write_variable_bytes, read_variable_bytes};
use bytes::{BufMut, BytesMut, Buf};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FixedHeader {
    pub packet_type: PacketType,
    pub dup: bool,
    pub qos: Qos,
    pub retain: bool,
    pub remaining_length: usize,
}

impl Frame<FixedHeader> for FixedHeader {
    fn to_buf(&self, buf: &mut impl BufMut) -> usize {
        let packet_type = self.packet_type.clone();
        let mut byte = packet_type.to_u8() << 4;
        if self.retain {
            byte |= 0x01;
        }
        byte |= self.qos.to_u8() << 1;
        if self.dup {
            byte |= 0b0000_1000;
        }
        let mut len = 1;
        buf.put_u8(byte);
        len += write_variable_bytes(self.remaining_length, |byte| buf.put_u8(byte));
        len
    }

    fn from_buf(buf: &mut BytesMut) -> Result<FixedHeader, Error> {
        let fixed_header_byte = buf.get_u8();
        let packet_type = PacketType::from_u8(fixed_header_byte >> 4)
            .expect("Failed to parse Packet Type in Fixed Header");
        let dup = (fixed_header_byte >> 3) & 0x01 == 1;
        let qos = Qos::from_u8((fixed_header_byte >> 1) & 0x03)?;
        let retain = fixed_header_byte & 0x01 == 1;
        let remaining_length = read_variable_bytes(buf)
            .expect("Failed to parse Fixed Header Remaining Length").0;
        assert_eq!(remaining_length, buf.len());
        Ok(FixedHeader {
            packet_type,
            dup,
            qos,
            retain,
            remaining_length,
        })
    }

    fn length(&self) -> usize {
        unimplemented!()
    }
}