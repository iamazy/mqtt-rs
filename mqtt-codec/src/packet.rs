use std::num::NonZeroU16;
use bytes::{BufMut, BytesMut};
use crate::{Error, FromToU8, FromToBuf};
use crate::fixed_header::FixedHeader;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PacketId(NonZeroU16);

impl PacketId {

    pub fn new(value: u16) -> Self {
        PacketId(NonZeroU16::new(value).unwrap())
    }

    pub fn get(self) -> u16 {
        self.0.get()
    }

    pub(crate) fn to_buf(self, buf: &mut impl BufMut) -> Result<usize, Error> {
        buf.put_u16(self.get());
        Ok(2)
    }
}

impl Default for PacketId {
    fn default() -> Self {
        PacketId::new(1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PacketType {
    /// 1, Client to server, Connection request
    CONNECT,
    /// 2, Server to client, Connect acknowledgement
    CONNACK,
    /// 3, Two-way, Publish message
    PUBLISH,
    /// 4, Two-way, Publish acknowledgement (QoS 1)
    PUBACK,
    /// 5, Two-way, Publish received (QoS 2 delivery part 1)
    PUBREC,
    /// 6, Two-way, Publish release (QoS 2 delivery part 2)
    PUBREL,
    /// 7, Two-way, Publish complete (Qos 2 delivery part 3)
    PUBCOMP,
    /// 8, Client to server, Subscribe request
    SUBSCRIBE,
    /// 9, Server to client, Subscribe acknowledgement
    SUBACK,
    /// 10, Client to server, Unsubscribe request
    UNSUBSCRIBE,
    /// 11, Server to client, Unsubscribe acknowledgement
    UNSUBACK,
    /// 12, Client to server, Ping request
    PINGREQ,
    /// 13, Server to client, Ping response
    PINGRESP,
    /// 14, Two-way, Disconnect notification
    DISCONNECT,
    /// 15, Two-way, Authentication exchange
    AUTH
}

impl FromToU8<PacketType> for PacketType {

    fn to_u8(&self) -> u8 {
        match *self {
            PacketType::CONNECT => 1,
            PacketType::CONNACK => 2,
            PacketType::PUBLISH => 3,
            PacketType::PUBACK => 4,
            PacketType::PUBREC => 5,
            PacketType::PUBREL => 6,
            PacketType::PUBCOMP => 7,
            PacketType::SUBSCRIBE => 8,
            PacketType::SUBACK => 9,
            PacketType::UNSUBSCRIBE => 10,
            PacketType::UNSUBACK => 11,
            PacketType::PINGREQ => 12,
            PacketType::PINGRESP => 13,
            PacketType::DISCONNECT => 14,
            PacketType::AUTH => 15,
        }
    }

    fn from_u8(byte: u8) -> Result<PacketType, Error> {
        match byte {
            1 => Ok(PacketType::CONNECT),
            2 => Ok(PacketType::CONNACK),
            3 => Ok(PacketType::PUBLISH),
            4 => Ok(PacketType::PUBACK),
            5 => Ok(PacketType::PUBREC),
            6 => Ok(PacketType::PUBREL),
            7 => Ok(PacketType::PUBCOMP),
            8 => Ok(PacketType::SUBSCRIBE),
            9 => Ok(PacketType::SUBACK),
            10 => Ok(PacketType::UNSUBSCRIBE),
            11 => Ok(PacketType::UNSUBACK),
            12 => Ok(PacketType::PINGREQ),
            13 => Ok(PacketType::PINGRESP),
            14 => Ok(PacketType::DISCONNECT),
            15 => Ok(PacketType::AUTH),
            n => Err(Error::InvalidPacketType(n))
        }
    }
}


pub trait Packet<R> {
    fn decode_fixed_header(buf: &mut BytesMut) -> FixedHeader {
        FixedHeader::from_buf(buf).expect("Failed to parse Fixed Header")
    }

    fn from_buf_extra(buf: &mut BytesMut, fixed_header: FixedHeader) -> Result<R, Error>;
}