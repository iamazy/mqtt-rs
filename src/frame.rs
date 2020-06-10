use crate::packet::PacketType;
use crate::publish::Qos;

pub struct Header {
    packet_type: PacketType,
    dup: bool,
    qos: Qos,
    retain: bool,
    remaining_length: usize
}