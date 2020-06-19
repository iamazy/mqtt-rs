use crate::packet::PacketType;
use crate::publish::Qos;
use crate::{Error, FromToU8};
use bytes::{BytesMut, Buf};
use crate::decoder::read_variable_byte;

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
        let remaining_length = read_variable_byte(buf)
            .expect("Failed to parse Fixed Header Remaining Length");
        Ok(FixedHeader {
            packet_type,
            dup,
            qos,
            retain,
            remaining_length,
        })
    }
}