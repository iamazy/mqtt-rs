use crate::packet::PacketType;
use crate::publish::Qos;
use crate::{Error, FromToU8};
use bytes::{BytesMut, Buf, BufMut};
use crate::decoder::{read_variable_bytes, write_variable_bytes};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedHeader {
    pub(crate) packet_type: PacketType,
    pub(crate) dup: bool,
    pub(crate) qos: Qos,
    pub(crate) retain: bool,
    pub(crate) remaining_length: usize
}

impl FixedHeader {

    pub(crate) fn new(buf: &mut BytesMut, dup: bool, qos: Qos, retain: bool) -> Result<FixedHeader, Error> {
        let fixed_header_buf = buf.get_u8();
        let packet_type = PacketType::from_u8(fixed_header_buf >> 4)
            .expect("Failed to parse Packet Type in Fixed Header");
        let remaining_length = read_variable_bytes(buf)
            .expect("Failed to parse Fixed Header Remaining Length").0;
        Ok(FixedHeader {
            packet_type,
            dup,
            qos,
            retain,
            remaining_length,
        })
    }

    pub fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let mut len = write_variable_bytes(self.remaining_length, |byte| buf.put_u8(byte))?;
        let packet_type = self.packet_type.clone();
        let mut byte = packet_type.to_u8() << 4;
        if self.retain {
            byte |= 0x00;
        } else {
            byte |= 0x01;
        }
        byte |= self.qos.to_u8();
        if self.dup {
            byte |= 0x06;
        } else {
            byte |= 0x07;
        }
        len += 1;
        Ok(len)
    }
}