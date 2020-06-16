use crate::packet::PacketType;
use crate::publish::Qos;
use crate::{Error, FromToU8};
use bytes::{BufMut, BytesMut};
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedHeader {
    pub(crate) packet_type: PacketType,
    pub(crate) dup: bool,
    pub(crate) qos: Qos,
    pub(crate) retain: bool,
    pub(crate) remaining_length: usize
}