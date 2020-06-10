use crate::packet::PacketType;
use crate::publish::Qos;
use crate::{Error, FromToU8};

pub struct Header {
    packet_type: PacketType,
    dup: bool,
    qos: Qos,
    retain: bool,
    remaining_length: usize
}

impl Header {

    /// Packet Type
    ///
    /// https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901023
    pub fn new(head: u8, remaining_length: u8) -> Result<Header, Error> {
        let (packet_type, valid_flag) = match head >> 4 {
            1 => (PacketType::CONNECT, head & 0b1111 == 0),
            2 => (PacketType::CONNACK, head & 0b1111 == 0),
            3 => (PacketType::PUBLISH, true),
            4 => (PacketType::PUBACK, head & 0b1111 == 0),
            5 => (PacketType::PUBREC, head & 0b1111 == 0),
            6 => (PacketType::PUBREL, head & 0b1111 == 0b0010),
            7 => (PacketType::PUBCOMP, head & 0b1111 == 0),
            8 => (PacketType::SUBSCRIBE, head & 0b1111 == 0b0010),
            9 => (PacketType::SUBACK, head & 0b1111 == 0),
            10 => (PacketType::UNSUBSCRIBE, head & 0b1111 == 0b0010),
            11 => (PacketType::UNSUBACK, head & 0b1111 == 0),
            12 => (PacketType::PINGREQ, head & 0b1111 == 0),
            13 => (PacketType::PINGRESP, head & 0b1111 == 0),
            14 => (PacketType::DISCONNECT, head & 0b1111 == 0),
            15 => (PacketType::AUTH, head & 0b1111 == 0),
            _ => (PacketType::CONNECT, false)
        };
        if !valid_flag {
            return Err(Error::InvalidHeader);
        }
        Ok(Header {
            packet_type,
            // https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901102
            dup: head & 0b1000 != 0,
            // https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901103
            qos: Qos::from_u8((head & 0b0110) >> 1)?,
            // https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901104
            retain: head & 0b0001 == 1,
            /// Remaining Length
            ///
            /// This is the length of Variable Header plus the length of the Payload, encoded as a Variable Byte Integer
            remaining_length: remaining_length as usize
        })
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn test_header() {
        println!("{:?}", 0b1000_0000 >> 4);

    }
}