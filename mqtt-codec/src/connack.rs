use crate::connect::ConnectReasonCode;
use crate::{Mqtt5Property, FromToBuf, Error, FromToU8, PropertyValue, write_variable_bytes};
use crate::frame::FixedHeader;
use bytes::{BytesMut, BufMut, Buf};
use crate::publish::Qos;
use crate::packet::PacketType;

#[derive(Debug, Clone, PartialEq)]
pub struct ConnAck {
    fixed_header: FixedHeader,
    connack_variable_header: ConnAckVariableHeader,
}

impl FromToBuf<ConnAck> for ConnAck {
    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let mut len = self.fixed_header.to_buf(buf)?;
        len += self.connack_variable_header.to_buf(buf)?;
        Ok(len)
    }

    fn from_buf(buf: &mut BytesMut) -> Result<ConnAck, Error> {
        let fixed_header = FixedHeader::from_buf(buf).expect("Failed to parse Connack Fixed Header");
        assert_eq!(fixed_header.packet_type, PacketType::CONNACK);
        assert_eq!(fixed_header.dup, false, "The dup of ConnAck Fixed Header must be set to false");
        assert_eq!(fixed_header.qos, Qos::AtMostOnce, "The qos of ConnAck Fixed Header must be set to be AtMostOnce");
        assert_eq!(fixed_header.retain, false, "The retain of ConnAck Fixed Header must be set to false");
        let connack_variable_header = ConnAckVariableHeader::from_buf(buf).expect("Failed to parse Connack Variable Header");
        Ok(ConnAck {
            fixed_header,
            connack_variable_header
        })
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
        // If the Session Expiry Interval is absent the value in the CONNECT Packet used.

        if !connack_property.properties.contains_key(&0x21) {
            connack_property.properties.insert(0x21, PropertyValue::TwoByteInteger(65535));
            connack_property.property_length += write_variable_bytes(0x21, |_| {})?;
            connack_property.property_length += 2;
        }

        if !connack_property.properties.contains_key(&0x24) {
            connack_property.properties.insert(0x24, PropertyValue::Byte(2));
            connack_property.property_length += write_variable_bytes(0x24, |_| {})?;
            connack_property.property_length += 1;
        }

        // If not present, then retained messages are supported
        if !connack_property.properties.contains_key(&0x25) {
            connack_property.properties.insert(0x25, PropertyValue::Byte(1));
            connack_property.property_length += write_variable_bytes(0x25, |_| {})?;
            connack_property.property_length += 1;
        }

        // If the Maximum Packet Size is not present, there is no limit on the packet size imposed beyond the limitations
        // in the protocol as a result of the remaining length encoding and the protocol header sizes.

        // If the Client connects using a zero length Client Identifier, the Server MUST respond with a CONNACK containing
        // an Assigned Client Identifier. The Assigned Client Identifier MUST be a new Client Identifier not used by any other Session currently in the Server

        if !connack_property.properties.contains_key(&0x22) {
            connack_property.properties.insert(0x22, PropertyValue::TwoByteInteger(0));
            connack_property.property_length += write_variable_bytes(0x22, |_| {})?;
            connack_property.property_length += 2;
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