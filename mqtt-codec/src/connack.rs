use crate::connect::ConnectReasonCode;
use crate::fixed_header::FixedHeader;
use crate::packet::{PacketCodec, PacketType};
use crate::publish::Qos;
use crate::{Error, Frame, FromToU8, Mqtt5Property};
use bytes::{Buf, BufMut, BytesMut};

#[derive(Debug, Clone, PartialEq)]
pub struct ConnAck {
    pub fixed_header: FixedHeader,
    pub variable_header: ConnAckVariableHeader,
}

impl Default for ConnAck {
    fn default() -> Self {
        let variable_header = ConnAckVariableHeader::default();
        let fixed_header = FixedHeader {
            packet_type: PacketType::CONNACK,
            dup: false,
            qos: Qos::AtMostOnce,
            retain: false,
            remaining_length: variable_header.length(),
        };
        ConnAck {
            fixed_header,
            variable_header,
        }
    }
}

impl PacketCodec<ConnAck> for ConnAck {
    fn from_buf_extra(buf: &mut BytesMut, fixed_header: FixedHeader) -> Result<ConnAck, Error> {
        let variable_header =
            ConnAckVariableHeader::from_buf(buf).expect("Failed to parse Connack Variable Header");
        Ok(ConnAck {
            fixed_header,
            variable_header,
        })
    }
}

impl Frame<ConnAck> for ConnAck {
    fn to_buf(&self, buf: &mut impl BufMut) -> usize {
        let mut len = self.fixed_header.to_buf(buf);
        len += self.variable_header.to_buf(buf);
        len
    }

    fn from_buf(buf: &mut BytesMut) -> Result<ConnAck, Error> {
        let fixed_header =
            FixedHeader::from_buf(buf).expect("Failed to parse Connack Fixed Header");
        assert_eq!(fixed_header.packet_type, PacketType::CONNACK);
        assert_eq!(
            fixed_header.dup, false,
            "The dup of ConnAck Fixed Header must be set to false"
        );
        assert_eq!(
            fixed_header.qos,
            Qos::AtMostOnce,
            "The qos of ConnAck Fixed Header must be set to be AtMostOnce"
        );
        assert_eq!(
            fixed_header.retain, false,
            "The retain of ConnAck Fixed Header must be set to false"
        );
        ConnAck::from_buf_extra(buf, fixed_header)
    }

    fn length(&self) -> usize {
        self.fixed_header.length() + self.fixed_header.remaining_length
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConnAckVariableHeader {
    pub connack_flags: ConnAckFlags,
    pub connect_reason_code: ConnectReasonCode,
    pub connack_property: Mqtt5Property,
}

impl ConnAckVariableHeader {
    fn check_connack_property(connack_property: &mut Mqtt5Property) -> Result<(), Error> {
        for key in connack_property.properties.keys() {
            let key = *key;
            match key {
                0x11 | 0x12 | 0x13 | 0x15 | 0x16 | 0x1A | 0x1C | 0x1F | 0x21 | 0x22 | 0x24
                | 0x25 | 0x26 | 0x27 | 0x28 | 0x29 | 0x2A => {}
                _ => {
                    return Err(Error::InvalidPropertyType(
                        "ConnAck Properties contains a invalid property".to_string(),
                    ))
                }
            }
        }
        Ok(())
    }
}

impl Frame<ConnAckVariableHeader> for ConnAckVariableHeader {
    fn to_buf(&self, buf: &mut impl BufMut) -> usize {
        let mut len = self.connack_flags.to_buf(buf);
        buf.put_u8(self.connect_reason_code.to_u8());
        len += 1;
        len += self.connack_property.to_buf(buf);
        len
    }

    fn from_buf(buf: &mut BytesMut) -> Result<ConnAckVariableHeader, Error> {
        let connack_flags = ConnAckFlags::from_buf(buf).expect("Failed to parse ConnAck");
        let connect_reason_code =
            ConnectReasonCode::from_u8(buf.get_u8()).expect("Not a valid Connect Reason Code");
        let mut connack_property =
            Mqtt5Property::from_buf(buf).expect("Failed to parse ConnAck Properties");
        ConnAckVariableHeader::check_connack_property(&mut connack_property)?;
        Ok(ConnAckVariableHeader {
            connack_flags,
            connect_reason_code,
            connack_property,
        })
    }

    fn length(&self) -> usize {
        2 + self.connack_property.length()
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ConnAckFlags {
    pub session_present: bool,
}

impl Frame<ConnAckFlags> for ConnAckFlags {
    fn to_buf(&self, buf: &mut impl BufMut) -> usize {
        if self.session_present {
            buf.put_u8(1);
        } else {
            buf.put_u8(0);
        }
        1
    }

    fn from_buf(buf: &mut BytesMut) -> Result<ConnAckFlags, Error> {
        let conn_ack_flags = buf.get_u8() & 0b1111_1111;
        if conn_ack_flags > 1 {
            return Err(Error::MalformedPacket);
        }
        let mut session_present = false;
        if conn_ack_flags == 1 {
            session_present = true;
        }
        Ok(ConnAckFlags { session_present })
    }

    fn length(&self) -> usize {
        1
    }
}

#[cfg(test)]
mod test {
    use crate::connack::ConnAck;
    use crate::Frame;
    use bytes::{BufMut, BytesMut};

    #[test]
    #[rustfmt::skip]
    fn test_connack() {
        let connack_bytes = &[
            0b0010_0000, 8,           // fixed header
            0b0000_0000, // connack flag
            0x00,        // conack reason code
            0x05, 0x11, 0x00, 0x00, 0x00, 0x10, // connack properties
        ];

        let mut buf = BytesMut::with_capacity(64);
        buf.put_slice(connack_bytes);
        let connack = ConnAck::from_buf(&mut buf).expect("Failed to parse ConnAck Packet");

        let mut buf = BytesMut::with_capacity(64);
        connack.to_buf(&mut buf);
        println!("{:?}", connack);
        assert_eq!(connack, ConnAck::from_buf(&mut buf).unwrap());
    }
}
