use std::num::NonZeroU16;
use bytes::BufMut;
use crate::Error;

/// Packet Identifier
///
/// when `Qos == 1 || Qos == 2`, `Packet Identifier` should be present in `PUBLISH` Packet
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
pub enum PacketType {
    /// 0, Client to server, Connection request
    CONNECT,
    /// 1, Server to client, Connect acknowledgement
    CONNACK,
    /// 2, Two-way, Publish message
    PUBLISH,
    /// 3, Two-way, Publish acknowledgement (QoS 1)
    PUBACK,
    /// 4, Two-way, Publish received (QoS 2 delivery part 1)
    PUBREC,
    /// 5, Two-way, Publish release (QoS 2 delivery part 2)
    PUBREL,
    /// 6, Two-way, Publish complete (Qos 2 delivery part 3)
    PUBCOMP,
    /// 7, Client to server, Subscribe request
    SUBSCRIBE,
    /// 8, Server to client, Subscribe acknowledgement
    SUBACK,
    /// 9, Client to server, Unsubscribe request
    UNSUBSCRIBE,
    /// 10, Server to client, Unsubscribe acknowledgement
    UNSUBACK,
    /// 11, Client to server, Ping request
    PINGREQ,
    /// 12, Server to client, Ping response
    PINGRESP,
    /// 13, Two-way, Disconnect notification
    DISCONNECT,
    /// 14, Two-way, Authentication exchange
    AUTH
}