use crate::connect::ConnectReasonCode;
use crate::{Mqtt5Property, FromToBuf, Error, FromToU8, PropertyValue, write_variable_bytes};
use crate::fixed_header::FixedHeader;
use bytes::{BytesMut, BufMut, Buf};
use crate::publish::Qos;
use crate::packet::{PacketType, Packet};

#[derive(Debug, Clone, PartialEq)]
pub struct ConnAck {
    fixed_header: FixedHeader,
    variable_header: ConnAckVariableHeader,
}

impl Packet<ConnAck> for ConnAck {
    fn from_buf_extra(buf: &mut BytesMut, mut fixed_header: FixedHeader) -> Result<ConnAck, Error> {
        let variable_header = ConnAckVariableHeader::from_buf(buf).expect("Failed to parse Connack Variable Header");
        Ok(ConnAck {
            fixed_header,
            variable_header
        })
    }
}

impl FromToBuf<ConnAck> for ConnAck {
    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let mut len = self.fixed_header.to_buf(buf)?;
        len += self.variable_header.to_buf(buf)?;
        Ok(len)
    }

    fn from_buf(buf: &mut BytesMut) -> Result<ConnAck, Error> {
        let fixed_header = FixedHeader::from_buf(buf).expect("Failed to parse Connack Fixed Header");
        assert_eq!(fixed_header.packet_type, PacketType::CONNACK);
        assert_eq!(fixed_header.dup, false, "The dup of ConnAck Fixed Header must be set to false");
        assert_eq!(fixed_header.qos, Qos::AtMostOnce, "The qos of ConnAck Fixed Header must be set to be AtMostOnce");
        assert_eq!(fixed_header.retain, false, "The retain of ConnAck Fixed Header must be set to false");
        ConnAck::from_buf_extra(buf, fixed_header)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConnAckVariableHeader {
    connack_flags: ConnAckFlags,
    connect_reason_code: ConnectReasonCode,
    connack_property: Mqtt5Property
}

impl ConnAckVariableHeader {

    fn check_connack_property(connack_property: &mut Mqtt5Property) -> Result<(), Error> {
        for key in connack_property.properties.keys() {
            let key = *key;
            match key {
                0x11 | 0x12 | 0x13 | 0x15 | 0x16 | 0x1A | 0x1C | 0x1F | 0x21 | 0x22 | 0x24 | 0x25 | 0x26 | 0x27 | 0x28 | 0x29| 0x2A => {},
                _ => {
                    return Err(Error::InvalidPropertyType("Connack Properties contains a invalid property".to_string()))
                }
            }
        }
        Ok(())
    }
}

impl FromToBuf<ConnAckVariableHeader> for ConnAckVariableHeader {

    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let mut len = self.connack_flags.to_buf(buf)?;
        buf.put_u8(self.connect_reason_code.to_u8());
        len += 1;
        len += self.connack_property.to_buf(buf)?;
        Ok(len)
    }

    fn from_buf(buf: &mut BytesMut) -> Result<ConnAckVariableHeader, Error> {
        let connack_flags = ConnAckFlags::from_buf(buf).expect("Failed to parse ConnAck");
        let connect_reason_code = ConnectReasonCode::from_u8(buf.get_u8())
            .expect("Not a valid Connect Reason Code");
        let mut connack_property = Mqtt5Property::from_buf(buf)
            .expect("Failed to parse ConnAck Properties");
        ConnAckVariableHeader::check_connack_property(&mut connack_property)?;
        Ok(ConnAckVariableHeader {
            connack_flags,
            connect_reason_code,
            connack_property
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConnAckFlags {
    session_present: bool,
}


impl FromToBuf<ConnAckFlags> for ConnAckFlags {
    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        if self.session_present {
            buf.put_u8(1);
        } else {
            buf.put_u8(0);
        }
        Ok(1)
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
        Ok(ConnAckFlags {
            session_present
        })
    }
}

#[cfg(test)]
mod test {
    use crate::connack::ConnAck;
    use bytes::{BytesMut, BufMut};
    use crate::FromToBuf;

    #[test]
    fn test_connack() {
        let connack_bytes = &[
            0b0010_0000, 2,  // fixed header
            0b0000_0000, // connack flag
            0x00, // conack reason code
            0x05, 0x11, 0x00, 0x00, 0x00, 0x10 // connack properties
        ];

        let mut buf = BytesMut::with_capacity(64);
        buf.put_slice(connack_bytes);
        let connack = ConnAck::from_buf(&mut buf)
            .expect("Failed to parse Connect Packet");

        let mut buf = BytesMut::with_capacity(64);
        connack.to_buf(&mut buf);
        println!("{:?}", connack);
        assert_eq!(connack, ConnAck::from_buf(&mut buf).unwrap());
    }
}