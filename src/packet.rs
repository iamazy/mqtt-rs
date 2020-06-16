use std::num::NonZeroU16;
use bytes::BufMut;
use crate::{Error, FromToU8};

/// Packet Identifier
///
/// when `Qos == 1 || Qos == 2`, `Packet Identifier` should be present in `PUBLISH` Packet
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PacketId(NonZeroU16);

impl PacketId {

    pub fn new() -> Self {
        PacketId(NonZeroU16::new(1).unwrap())
    }

    /// Get `Packet Identifier` as a raw `u16`
    pub fn get(self) -> u16 {
        self.0.get()
    }

    pub(crate) fn to_buf(self, buf: &mut impl BufMut) -> Result<(), Error> {
        Ok(buf.put_u16(self.get()))
    }
}

impl Default for PacketId {
    fn default() -> Self {
        PacketId::new()
    }
}

/// MQTT Control Packet type
/// Position: 1byte, bits 7-4
///
/// https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901022
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

#[cfg(test)]
mod test {

    #[test]
    fn parse_connect_fixed_header() {

    }
}