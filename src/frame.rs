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
        let fixed_header_byte = buf.get_u8();
        let packet_type = PacketType::from_u8(fixed_header_byte >> 4)
            .expect("Failed to parse Packet Type in Fixed Header");
        if dup != ((fixed_header_byte >> 3) & 0x01 == 1) {
            return Err(Error::MalformedFixedHeader("It's a Malformed FixedHeader, Please check it Dup again".to_string()))
        }
        if qos != (Qos::from_u8((fixed_header_byte >> 1) & 0x03)?) {
            return Err(Error::MalformedFixedHeader("It's a Malformed FixedHeader, Please check it Qos again".to_string()))
        }
        if retain != (fixed_header_byte & 0x01 == 1) {
            return Err(Error::MalformedFixedHeader("It's a Malformed FixedHeader, Please check it Retain again".to_string()))
        }
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
        len += write_variable_bytes(self.remaining_length, |byte| buf.put_u8(byte))?;
        Ok(len)
    }
}