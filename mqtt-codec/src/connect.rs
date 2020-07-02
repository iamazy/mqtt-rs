use crate::{Error, FromToU8, FromToBuf, PropertyValue, Mqtt5Property, write_variable_bytes, read_string, write_string, read_bytes, write_bytes};
use crate::protocol::Protocol;
use crate::publish::Qos;
use bytes::{BytesMut, BufMut, Buf, Bytes};
use crate::fixed_header::FixedHeader;
use crate::packet::{PacketType, Packet};
use std::env::var;

#[derive(Debug, Clone, PartialEq)]
pub struct Connect {
    fixed_header: FixedHeader,
    variable_header: ConnectVariableHeader,
    payload: ConnectPayload,
}

impl Packet<Connect> for Connect {

    fn from_buf_extra(buf: &mut BytesMut, mut fixed_header: FixedHeader) -> Result<Connect, Error> {
        // parse variable header
        let variable_header = ConnectVariableHeader::from_buf(buf)
            .expect("Failed to parse Connect Variable Header");
        // parse connect payload
        let payload = ConnectPayload::from_buf(buf, &variable_header.connect_flags)
            .expect("Failed to parse Connect Payload");
        Ok(Connect {
            fixed_header,
            variable_header,
            payload,
        })
    }
}

impl FromToBuf<Connect> for Connect {
    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let mut len = self.fixed_header.to_buf(buf).expect("Failed to parse Fixed Header to Byte Buf");
        len += self.variable_header.to_buf(buf).expect("Failed to parse Variable Header to Byte Buf");
        len += self.payload.to_buf(buf, &self.variable_header.connect_flags.clone())?;
        Ok(len)
    }

    fn from_buf(buf: &mut BytesMut) -> Result<Connect, Error> {
        // parse fixed header
        let fixed_header = Connect::decode_fixed_header(buf);
        assert_eq!(fixed_header.packet_type, PacketType::CONNECT);
        assert_eq!(fixed_header.dup, false, "The dup of Connect Fixed Header must be set to false");
        assert_eq!(fixed_header.qos, Qos::AtMostOnce, "The qos of Connect Fixed Header must be set to be AtMostOnce");
        assert_eq!(fixed_header.retain, false, "The retain of Connect Fixed Header must be set to false");
        Connect::from_buf_extra(buf, fixed_header)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConnectVariableHeader {
    pub protocol: Protocol,
    pub connect_flags: ConnectFlags,
    pub keep_alive: u16,
    pub connect_property: Mqtt5Property,
}

impl ConnectVariableHeader {

    fn check_connect_property(connect_property: &mut Mqtt5Property) -> Result<(), Error> {

        for key in connect_property.properties.keys() {
            let key = *key;
            match key {
                0x11 | 0x15 | 0x16 | 0x17 | 0x19 | 0x21 | 0x22 | 0x26 | 0x27 => {},
                _ => {
                    return Err(Error::InvalidPropertyType("Connect Properties contains a invalid property".to_string()))
                }
            }
        }
        Ok(())
    }
}

impl FromToBuf<ConnectVariableHeader> for ConnectVariableHeader {

    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let mut len: usize = 0;
        len += self.protocol.to_buf(buf).expect("Failed to parse Protocol to Byte Buf");
        len += self.connect_flags.to_buf(buf).expect("Failed to parse Connect Flags to Byte Buf");
        buf.put_u16(self.keep_alive);
        len += 2;
        len += self.connect_property.to_buf(buf).expect("Failed to parse Connect Property to Byte Buf");
        Ok(len)
    }

    fn from_buf(buf: &mut BytesMut) -> Result<ConnectVariableHeader, Error> {
        let protocol = Protocol::from_buf(buf).expect("Failed to parse Protocol");
        let connect_flags = ConnectFlags::from_buf(buf).expect("Failed to parse Connect Flag");
        let keep_alive = buf.get_u16();
        let mut connect_property = Mqtt5Property::from_buf(buf).expect("Failed to parse Connect Properties");
        ConnectVariableHeader::check_connect_property(&mut connect_property)?;
        Ok(ConnectVariableHeader {
            protocol,
            connect_flags,
            keep_alive,
            connect_property,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConnectFlags {
    clean_start: bool,
    will_flag: bool,
    will_qos: Qos,
    will_retain: bool,
    username_flag: bool,
    password_flag: bool,
}

impl FromToBuf<ConnectFlags> for ConnectFlags {
    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let mut connect_flags = 0b0000_0000;
        if self.clean_start {
            connect_flags |= 0b0000_0010;
        }
        if self.will_flag {
            connect_flags |= 0b0000_0100;
        }
        connect_flags |= self.will_qos.to_u8() << 3;
        if self.will_retain {
            connect_flags |= 0b0010_0000;
        }
        if self.password_flag {
            connect_flags |= 0b0100_0000;
        }
        if self.username_flag {
            connect_flags |= 0b1000_0000;
        }
        connect_flags &= 0b1111_1110;
        buf.put_u8(connect_flags);
        Ok(connect_flags as usize)
    }

    fn from_buf(buf: &mut BytesMut) -> Result<ConnectFlags, Error> {
        let connect_flags = buf.get_u8();
        let reserved_flag = connect_flags & 0x01 != 0;
        if reserved_flag {
            return Err(Error::MalformedPacket);
        }
        // If a CONNECT packet is received with Clean Start is set to 1, the Client and Server MUST discard any existing Session and start a new Session
        // If a CONNECT packet is received with Clean Start set to 0 and there is a Session associated with the Client Identifier, the Server MUST resume communications with the Client based on state from the existing Session [MQTT-3.1.2-5].
        // If a CONNECT packet is received with Clean Start set to 0 and there is no Session associated with the Client Identifier, the Server MUST create a new Session [MQTT-3.1.2-6].
        let clean_start = (connect_flags >> 1) & 0x01 > 0;
        let will_flag = (connect_flags >> 2) & 0x01 > 0;
        let will_qos = Qos::from_u8((connect_flags >> 3) & 0x03).expect("Expected [Will Qos] value is 0(0x00), 1(0x01), 2(0x02)");
        // If the Will Flag is set to 0, then the Will QoS MUST be set to 0 (0x00)
        // If the Will Flag is set to 1, the value of Will QoS can be 0 (0x00), 1 (0x01), or 2 (0x02), A value of 3 (0x03) is a Malformed Packet.
        if !will_flag && will_qos != Qos::AtMostOnce {
            return Err(Error::MalformedPacket);
        }
        let will_retain = (connect_flags >> 5) & 0x01 > 0;
        // If the Will Flag is set to 0, then Will Retain MUST be set to 0.
        // If the Will Flag is set to 1 and Will Retain is set to 0, the Server MUST publish the Will Message as a non-retained message.
        // If the Will Flag is set to 1 and Will Retain is set to 1, the Server MUST publish the Will Message as a retained message
        if !will_flag && will_retain {
            return Err(Error::MalformedPacket);
        }
        let password_flag = (connect_flags >> 6) & 0x01 > 0;
        // MQTT5 allows the sending of a Password with no User Name, where MQTT v3.1.1 did not.
        // This reflects the common use of Password for credentials other than a password.
        let username_flag = (connect_flags >> 7) & 0x01 > 0;
        Ok(ConnectFlags {
            clean_start,
            will_flag,
            will_qos,
            will_retain,
            password_flag,
            username_flag,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConnectPayload {
    client_id: String,
    will_property: Option<Mqtt5Property>,
    will_topic: Option<String>,
    will_payload: Option<Bytes>,
    username: Option<String>,
    password: Option<String>,
}

impl ConnectPayload {

    fn new() -> ConnectPayload {
        ConnectPayload {
            client_id: String::default(),
            will_property: None,
            will_topic: None,
            will_payload: None,
            username: None,
            password: None,
        }
    }

    fn check_will_property(will_property: &mut Mqtt5Property) -> Result<(), Error> {

        for key in will_property.properties.keys() {
            let key = *key;
            match key {
                0x01 | 0x02 | 0x03 | 0x08 | 0x09 | 0x18 | 0x26 => {},
                _ => {
                    return Err(Error::InvalidPropertyType("Will Properties contains a invalid property".to_string()))
                }
            }
        }
        Ok(())
    }

    fn to_buf(&self, buf: &mut impl BufMut, connect_flags: &ConnectFlags) -> Result<usize, Error> {
        let mut len:usize = 0;
        len += write_string(self.client_id.clone(), buf);
        if connect_flags.will_flag {
            let will_property = self.will_property.clone();
            len += will_property.expect("Failed to get Will Property from Connect Payload").to_buf(buf)?;
            let will_topic = self.will_topic.as_ref().expect("Failed to get Will Topic from Connect Payload");
            len += write_string(will_topic.clone(), buf);
            let will_payload = self.will_payload.as_ref().expect("Failed to get Will Payload from Connect Payload");
            len += write_bytes(will_payload.clone(), buf);
        }
        if connect_flags.username_flag {
            let username = self.username.as_ref().expect("Failed to get Username from Connect Payload");
            len += write_string(username.clone(), buf)
        }
        if connect_flags.password_flag {
            let password = self.password.as_ref().expect("Failed to get Password from Connect Payload");
            len += write_string(password.clone(), buf)
        }
        Ok(len)
    }

    fn from_buf(buf: &mut BytesMut, connect_flags: &ConnectFlags) -> Result<ConnectPayload, Error> {
        let mut connect_payload = ConnectPayload::new();
        let client_id = read_string(buf).expect("Failed to parse Client Id in CONNECT Payload");
        connect_payload.client_id = client_id;

        if connect_flags.will_flag {
            let mut will_property = Mqtt5Property::from_buf(buf).expect("Failed to parse Will Property");
            ConnectPayload::check_will_property(&mut will_property)?;
            connect_payload.will_property = Some(will_property);

            let will_topic = read_string(buf).expect("Failed to parse Will Topic in Will Properties");
            connect_payload.will_topic = Some(will_topic);
            let will_payload = read_bytes(buf).expect("Failed to parse Will Payload in Will Properties");
            connect_payload.will_payload = Some(will_payload);
        }
        if connect_flags.username_flag {
            let username = read_string(buf).expect("Failed to parse username in Will Properties");
            connect_payload.username = Some(username);
        }
        if connect_flags.password_flag {
            let password = read_string(buf).expect("Failed to parse password in Will Properties");
            connect_payload.password = Some(password);
        }

        Ok(connect_payload)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectReasonCode {
    /// 0[0x00], Connection accepted
    Success,
    /// 128[0x80], The Server does not wish to reveal the reason for the failure,
    /// or none of the other Reason Codes apply.
    UnspecifiedError,
    /// 129[0x81], Data within the `CONNECT` packet could not be correctly parsed
    MalformedPacket,
    /// 130[0x82], Data in the `CONNECT` packet does not conform to this specification
    ProtocolError,
    /// 131[0x83], The `CONNECT` is valid but is not accepted by this Server
    ImplementationSpecificError,
    /// 132[0x84], The Server does not support the version of the MQTT protocol requested by the Client.
    UnsupportedProtocolVersion,
    /// 133[0x85], The Client Identifier is a valid string but is not allowed by the Server
    ClientIdentifierNotValid,
    /// 134[0x86], The Server does not accept the User Name or Password specified by the Client
    BadUsernameOrPassword,
    /// 135[0x87], The Client is not authorized to connect
    NotAuthorized,
    /// 136[0x88], The MQTT Server is not available
    ServerUnavailable,
    /// 137[0x89], The Server is busy. Try again later
    ServerBusy,
    /// 138[0x8A], This Client has been banned by administrative action. Contact the server administrator
    Banned,
    /// 140[0x8C], The authentication method is not supported or does not match the authentication method currently in use
    BadAuthenticationMethod,
    /// 144[0x90], The Will Topic Name is not malformed, but is not accepted by this Server
    TopicNameInvalid,
    /// 149[0x95], The `CONNECT` packet exceeded the maximum permissible size
    PacketTooLarge,
    /// 151[0x97], An implementation or administrative imposed limit has been exceeded
    QuotaExceeded,
    /// 153[0x99], The Will Payload does not match the specified Payload Format Indicator
    PayloadFormatInvalid,
    /// 154[0x9A], The Server does not support retained messages, and Will Retain was set to 1
    RetainNotSupported,
    /// 155[0x9B], The Server does not support the QoS set in Will QoS
    QoSNotSupported,
    /// 156[0x9C], The Client should temporarily use another server
    UseAnotherServer,
    /// 157[0x9D], The Client should permanently use another server
    ServerMoved,
    /// 159[0x9F], The connection rate limit has been exceeded
    ConnectionRateExceeded,
}

impl FromToU8<ConnectReasonCode> for ConnectReasonCode {
    fn to_u8(&self) -> u8 {
        match *self {
            ConnectReasonCode::Success => 0,
            ConnectReasonCode::UnspecifiedError => 128,
            ConnectReasonCode::MalformedPacket => 129,
            ConnectReasonCode::ProtocolError => 130,
            ConnectReasonCode::ImplementationSpecificError => 131,
            ConnectReasonCode::UnsupportedProtocolVersion => 132,
            ConnectReasonCode::ClientIdentifierNotValid => 133,
            ConnectReasonCode::BadUsernameOrPassword => 134,
            ConnectReasonCode::NotAuthorized => 135,
            ConnectReasonCode::ServerUnavailable => 136,
            ConnectReasonCode::ServerBusy => 137,
            ConnectReasonCode::Banned => 138,
            ConnectReasonCode::BadAuthenticationMethod => 140,
            ConnectReasonCode::TopicNameInvalid => 144,
            ConnectReasonCode::PacketTooLarge => 149,
            ConnectReasonCode::QuotaExceeded => 151,
            ConnectReasonCode::PayloadFormatInvalid => 153,
            ConnectReasonCode::RetainNotSupported => 154,
            ConnectReasonCode::QoSNotSupported => 155,
            ConnectReasonCode::UseAnotherServer => 156,
            ConnectReasonCode::ServerMoved => 157,
            ConnectReasonCode::ConnectionRateExceeded => 159
        }
    }

    fn from_u8(byte: u8) -> Result<ConnectReasonCode, Error> {
        match byte {
            0 => Ok(ConnectReasonCode::Success),
            128 => Ok(ConnectReasonCode::UnspecifiedError),
            129 => Ok(ConnectReasonCode::MalformedPacket),
            130 => Ok(ConnectReasonCode::ProtocolError),
            131 => Ok(ConnectReasonCode::ImplementationSpecificError),
            132 => Ok(ConnectReasonCode::UnsupportedProtocolVersion),
            133 => Ok(ConnectReasonCode::ClientIdentifierNotValid),
            134 => Ok(ConnectReasonCode::BadUsernameOrPassword),
            135 => Ok(ConnectReasonCode::NotAuthorized),
            136 => Ok(ConnectReasonCode::ServerUnavailable),
            137 => Ok(ConnectReasonCode::ServerBusy),
            138 => Ok(ConnectReasonCode::Banned),
            140 => Ok(ConnectReasonCode::BadAuthenticationMethod),
            144 => Ok(ConnectReasonCode::TopicNameInvalid),
            149 => Ok(ConnectReasonCode::PacketTooLarge),
            151 => Ok(ConnectReasonCode::QuotaExceeded),
            153 => Ok(ConnectReasonCode::PayloadFormatInvalid),
            154 => Ok(ConnectReasonCode::RetainNotSupported),
            155 => Ok(ConnectReasonCode::QoSNotSupported),
            156 => Ok(ConnectReasonCode::UseAnotherServer),
            157 => Ok(ConnectReasonCode::ServerMoved),
            159 => Ok(ConnectReasonCode::ConnectionRateExceeded),
            n => Err(Error::InvalidReasonCode(n))
        }
    }
}

#[cfg(test)]
mod test {
    use bytes::{BytesMut, BufMut, Buf};
    use crate::connect::{ConnectVariableHeader, Connect};
    use crate::FromToBuf;
    use crate::PropertyValue::Byte;

    #[test]
    fn test_variable_header_example() {
        let mut buf = BytesMut::with_capacity(64);
        buf.put_u8(0);
        buf.put_u8(4);
        buf.put(&b"MQTT"[..]);
        buf.put_u8(5);
        buf.put_u8(0b11001110);
        buf.put_u8(0);
        buf.put_u8(10);
        buf.put_u8(5);
        buf.put_u8(17);
        buf.put_u32(10);

        let variable_header = ConnectVariableHeader::from_buf(&mut buf)
            .expect("Failed to parse Connect Variable Header");

        let mut buf = BytesMut::with_capacity(64);
        variable_header.to_buf(&mut buf);
        println!("{:?}", buf.to_vec());
        println!("{:?}", variable_header);
        println!("{:?}", ConnectVariableHeader::from_buf(&mut buf));
    }

    #[test]
    fn test_take() {
        use bytes::{BufMut, buf::BufExt};

        let mut buf = b"hello world"[..].take(100);
        let mut dst = vec![];

        dst.put(&mut buf);
        println!("{}", dst.len());

        let mut buf = buf.into_inner();
        dst.clear();
        dst.put(&mut buf);
        println!("{}", buf.len());
    }

    #[test]
    fn test_connect() {
        let connect_bytes = &[
            0b0001_0000u8, 49,  // fixed header
            0x00, 0x04, 'M' as u8, 'Q' as u8, 'T' as u8, 'T' as u8, // protocol name
            0x05, // protocol version
            0b1100_1110u8, // connect flag
            0x00, 0x10, // keep alive
            0x05, 0x11, 0x00, 0x00, 0x00, 0x10, // connect properties
            0x00, 0x03, 'c' as u8, 'i' as u8, 'd' as u8, // client id
            0x05, 0x02, 0x00, 0x00, 0x00, 0x10, // will properties
            0x00, 0x04, 'w' as u8, 'i' as u8, 'l' as u8, 'l' as u8, // will topic
            0x00, 0x01, 'p' as u8, // will payload
            0x00, 0x06, 'i' as u8, 'a' as u8, 'm' as u8, 'a' as u8, 'z' as u8, 'y' as u8, // username
            0x00, 0x06, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06
        ];
        let mut buf = BytesMut::with_capacity(64);
        buf.put_slice(connect_bytes);
        let connect = Connect::from_buf(&mut buf)
            .expect("Failed to parse Connect Packet");

        let mut buf = BytesMut::with_capacity(64);
        connect.to_buf(&mut buf);
        println!("{:?}", buf.to_vec());
        assert_eq!(connect, Connect::from_buf(&mut buf).unwrap());
    }
}