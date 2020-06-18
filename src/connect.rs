use crate::{Error, FromToU8, FromToBuf, PropertyValue, Mqtt5Property};
use crate::protocol::Protocol;
use std::collections::{LinkedList, HashMap};
use crate::publish::Qos;
use bytes::{BytesMut, BufMut, Buf};
use crate::PropertyType;
use crate::decoder::{read_string, read_variable_byte_integer, read_bytes};
use crate::frame::FixedHeader;
use crate::packet::PacketType;
use bytes::buf::BufExt;
use dashmap::DashMap;
use std::collections::hash_map::RandomState;
use crate::connect::ConnectReasonCode::ProtocolError;


pub struct Connect {
    fixed_header: FixedHeader,
    variable_header: ConnectVariableHeader,
    payload: ConnectPayload,
}

impl FromToBuf<Connect> for Connect {
    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        unimplemented!()
    }

    fn from_buf(buf: &mut BytesMut) -> Result<Connect, Error> {
        // parse fixed header
        let fixed_header_buf = buf.get_u8();
        let packet_type = PacketType::from_u8(fixed_header_buf >> 4).expect("Failed to parse Packet Type in Fixed Header");
        let fixed_header_remaining_length = read_variable_byte_integer(buf).expect("Failed to parse Fixed Header Remaining Length");
        let fixed_header = FixedHeader {
            packet_type,
            dup: false,
            qos: Qos::AtMostOnce,
            retain: false,
            remaining_length: fixed_header_remaining_length,
        };

        // parse variable header
        let protocol_name = read_string(buf).expect("Failed to parse protocol name");
        let protocol_level = buf.get_u8();
        let protocol = Protocol::new(&protocol_name, protocol_level).unwrap();
        let connect_flags = ConnectFlag::from_buf(buf).expect("Connect Flag can not be None, Please check it.");
        let keep_alive = buf.get_u16() as usize;

        // parse connect properties
        let mut connect_property_length = read_variable_byte_integer(buf)?;
        let mut prop_len: usize = 0;
        let mut connect_property = Mqtt5Property::new();
        let mut connect_user_properties = LinkedList::<(String, String)>::new();
        while connect_property_length > prop_len {
            let property_id = read_variable_byte_integer(buf)?;
            match property_id {
                0x11 => {
                    if connect_property.properties.contains_key(&0x11) {
                        return Err(Error::InvalidProtocol("Connect properties cannot contains [Session Expire Interval] more than once".to_string(), 0x11));
                    }
                    connect_property.properties.insert(0x11, PropertyValue::FourByteInteger(buf.get_u32()));
                    prop_len += 4;
                }
                0x21 => {
                    // It is a Protocol Error to include the Receive Maximum value more than once or for receive maximum to have the value 0
                    if connect_property.properties.contains_key(&0x21) ||
                        *connect_property.properties.get(&0x21).unwrap() == PropertyValue::TwoByteInteger(0) {
                        return Err(Error::InvalidProtocol("Connect properties cannot contains [Receive Maximum] more than once or for it have the value 0".to_string(), 0x21));
                    }
                    // The Client uses this value to limit the number of QoS 1 and QoS 2 publications that it is willing to process concurrently.
                    // There is no mechanism to limit the QoS 0 publications that the Server might try to send
                    connect_property.properties.insert(0x21, PropertyValue::TwoByteInteger(buf.get_u16()));
                    prop_len += 2;
                }
                0x27 => {
                    // It is a Protocol Error to include the Maximum Packet Size more than once, or for the value to be set to zero.
                    // If the Maximum Packet Size is not present, no limit on the packet size is imposed beyond the limitations in the
                    // protocol as a result of the remaining length encoding and the protocol header sizes.
                    if connect_property.properties.contains_key(&0x27) ||
                        *connect_property.properties.get(&0x27).unwrap() == PropertyValue::FourByteInteger(0) {
                        return Err(Error::InvalidProtocol("Connect properties cannot contains [Maximum Packet Size] more than once or for it have the value 0".to_string(), 0x27));
                    }
                    connect_property.properties.insert(0x27, PropertyValue::FourByteInteger(buf.get_u32()));
                    prop_len += 4;
                }
                0x22 => {
                    //  It is a Protocol Error to include the Topic Alias Maximum value more than once.
                    if connect_property.properties.contains_key(&0x22) {
                        return Err(Error::InvalidProtocol("Connect properties cannot contains [Topic Alias Maximum] more than once".to_string(), 0x22));
                    }
                    connect_property.properties.insert(0x22, PropertyValue::TwoByteInteger(buf.get_u16()));
                    prop_len += 2;
                }
                0x19 => {
                    let request_response_information = buf.get_u8();
                    if connect_property.properties.contains_key(&0x19) ||
                        (request_response_information != 0 && request_response_information != 1) {
                        return Err(Error::InvalidProtocol("Connect properties cannot contains [Request Response Information] more than once or to have a value other than 0 or 1".to_string(), 0x19));
                    }
                    connect_property.properties.insert(0x19, PropertyValue::Bit(request_response_information & 0x01 == 1));
                    prop_len += 1;
                }
                0x17 => {
                    let request_problem_information = buf.get_u8();
                    if connect_property.properties.contains_key(&0x17) ||
                        (request_problem_information != 0 && request_problem_information != 1) {
                        return Err(Error::InvalidProtocol("Connect properties cannot contains [Request Problem Information] more than once or to have a value other than 0 or 1".to_string(), 0x17));
                    }
                    connect_property.properties.insert(0x17, PropertyValue::Bit(request_problem_information & 0x01 == 1));
                    prop_len += 1;
                }
                0x26 => {
                    // The User Property is allowed to appear multiple times to represent multiple name, value pairs.
                    // The same name is allowed to appear more than once.
                    let name = read_string(buf).expect("Failed to parse User property in Connect Properties");
                    let value = read_string(buf).expect("Failed to parse User property in Connect Properties");
                    prop_len += name.clone().into_bytes().len();
                    prop_len += value.clone().into_bytes().len();
                    connect_user_properties.push_back((name, value));
                }
                0x15 => {
                    if connect_property.properties.contains_key(&0x15) {
                        return Err(Error::InvalidProtocol("Connect properties cannot contains [Authentication Method] more than once".to_string(), 0x15));
                    }
                    let authentication_method = read_string(buf).expect("Failed to parse Authentication Method in Connect Properties");
                    prop_len += authentication_method.clone().into_bytes().len();
                    connect_property.properties.insert(0x15, PropertyValue::String(authentication_method));
                }
                0x16 => {
                    if connect_property.properties.contains_key(&0x16) {
                        return Err(Error::InvalidProtocol("Connect properties cannot contains [Authentication Data] more than once".to_string(), 0x16));
                    }
                    let length = buf.get_u16() as usize;
                    let mut data = buf.split_to(length);
                    connect_property.properties.insert(0x16, PropertyValue::Binary(data.to_bytes()));
                    prop_len += length;
                }
                _ => {
                    return Err(Error::InvalidConnectPropertyType);
                }
            }
        }
        // If the Session Expiry Interval is absent the value 0 is used. If it is set to 0, or is absent,
        // the Session ends when the Network Connection is closed.
        // If the Session Expiry Interval is 0xFFFFFFFF (UINT_MAX), the Session does not expire.
        // The Client and Server MUST store the Session State after the Network Connection is closed if
        // the Session Expiry Interval is greater than 0
        if !connect_property.properties.contains_key(&0x11) {
            connect_property.properties.insert(0x11, PropertyValue::FourByteInteger(0));
        }
        // The value of Receive Maximum applies only to the current Network Connection.
        // If the Receive Maximum value is absent then its value defaults to 65,535
        if !connect_property.properties.contains_key(&0x21) {
            connect_property.properties.insert(0x21, PropertyValue::TwoByteInteger(65535));
        }
        // If the Topic Alias Maximum property is absent, the default value is 0.
        if !connect_property.properties.contains_key(&0x22) {
            connect_property.properties.insert(0x22, PropertyValue::TwoByteInteger(0));
        }
        // If the Request Response Information is absent, the value of 0 is used.
        if !connect_property.properties.contains_key(&0x19) {
            connect_property.properties.insert(0x19, PropertyValue::Bit(false));
        }
        if connect_user_properties.len() > 0 {
            connect_property.properties.insert(0x26, PropertyValue::StringPair(connect_user_properties));
        }
        //  It is a Protocol Error to include Authentication Data if there is no Authentication Method
        if connect_property.properties.contains_key(&0x16) &&
            !connect_property.properties.contains_key(&0x15) {
            return Err(Error::InvalidProtocol("Connect properties cannot contains Authentication Data if there is no Authentication Method".to_string(), 0x16));
        }

        let connect_variable_header = ConnectVariableHeader {
            protocol,
            connect_flags,
            keep_alive,
            connect_property,
        };

        // parse connect payload
        let mut connect_payload = ConnectPayload::new();
        let client_id = read_string(buf).expect("Failed to parse Client Id in CONNECT Payload");
        connect_payload.client_id = client_id;
        // parse will properties
        // If the Will Flag is set to 1, the Will Properties is the next field in the Payload.
        // The Will Properties field defines the Application Message properties to be sent with the Will Message when it is published,
        // and properties which define when to publish the Will Message. The Will Properties consists of a Property Length and the Properties
        if connect_variable_header.connect_flags.will_flag {
            let will_property_length = read_variable_byte_integer(buf).expect("Failed to parse Will Property length");
            let mut prop_len: usize = 0;
            let mut will_property = Mqtt5Property::new();
            let mut will_user_properties = LinkedList::<(String, String)>::new();
            while will_property_length > prop_len {
                let property_id = read_variable_byte_integer(buf).expect("Failed to parse Property Id in Will Properties");
                match property_id {
                    0x18 => {
                        if will_property.properties.contains_key(&0x18) {
                            return Err(Error::InvalidProtocol("Will properties cannot contains [Will Delay Interval] more than once".to_string(), 0x18));
                        }
                        will_property.properties.insert(0x18, PropertyValue::FourByteInteger(buf.get_u32()));
                        prop_len += 4;
                    }
                    0x01 => {
                        if will_property.properties.contains_key(&0x01) {
                            return Err(Error::InvalidProtocol("Will properties cannot contains [Payload Format Indicator] more than once".to_string(), 0x18));
                        }
                        will_property.properties.insert(0x01, PropertyValue::Bit(buf.get_u8() & 0x01 == 1));
                        prop_len += 1;
                    }
                    0x02 => {
                        if will_property.properties.contains_key(&0x02) {
                            return Err(Error::InvalidProtocol("Will properties cannot contains [Message Expiry Interval] more than once".to_string(), 0x02));
                        }
                        // If present, the Four Byte value is the lifetime of the Will Message in seconds and is sent as the Publication Expiry Interval when the Server publishes the Will Message.
                        // If absent, no Message Expiry Interval is sent when the Server publishes the Will Message.
                        will_property.properties.insert(0x02, PropertyValue::FourByteInteger(buf.get_u32()));
                        prop_len += 4;
                    }
                    0x03 => {
                        if will_property.properties.contains_key(&0x03) {
                            return Err(Error::InvalidProtocol("Will properties cannot contains [Content Type] more than once".to_string(), 0x03));
                        }
                        let content_type = read_string(buf).expect("Failed to parse Content Type in Will Properties");
                        prop_len += content_type.clone().into_bytes().len();
                        will_property.properties.insert(0x03, PropertyValue::String(content_type));
                    }
                    0x08 => {
                        if will_property.properties.contains_key(&0x08) {
                            return Err(Error::InvalidProtocol("Will properties cannot contains [Response Topic] more than once".to_string(), 0x08));
                        }
                        let response_topic = read_string(buf).expect("Failed to parse Response Topic in Will Properties");
                        prop_len += response_topic.clone().into_bytes().len();
                        will_property.properties.insert(0x08, PropertyValue::String(response_topic));
                    }
                    0x09 => {
                        if will_property.properties.contains_key(&0x09) {
                            return Err(Error::InvalidProtocol("Will properties cannot contains [Correlation Data] more than once".to_string(), 0x09));
                        }
                        let length = buf.get_u16() as usize;
                        prop_len += length;
                        let mut data = buf.split_to(length);
                        will_property.properties.insert(0x09, PropertyValue::Binary(data.to_bytes()));
                    }
                    0x26 => {
                        let name = read_string(buf).expect("Failed to parse User property in Will Properties");
                        let value = read_string(buf).expect("Failed to parse User property in Will Properties");
                        prop_len += name.clone().into_bytes().len();
                        prop_len += value.clone().into_bytes().len();
                        will_user_properties.push_back((name, value));
                    }
                    _ => {
                        return Err(Error::InvalidWillPropertyType);
                    }
                }
            }
            // If the Will Delay Interval is absent, the default value is 0 and there is no delay before the Will Message is published.
            if !will_property.properties.contains_key(&0x18) {
                will_property.properties.insert(0x18, PropertyValue::FourByteInteger(0));
            }
            if will_user_properties.len() > 0 {
                will_property.properties.insert(0x26, PropertyValue::StringPair(will_user_properties));
            }
            connect_payload.will_property = Some(will_property);

            let will_topic = read_string(buf).expect("Failed to parse Will Topic in Will Properties");
            connect_payload.will_topic = Some(will_topic);
            let will_payload = read_bytes(buf).expect("Failed to parse Will Payload in Will Properties");
            connect_payload.will_payload = Some(will_payload);
        }
        if connect_variable_header.connect_flags.username_flag {
            let username = read_string(buf).expect("Failed to parse username in Will Properties");
            connect_payload.username = Some(username);
        }
        if connect_variable_header.connect_flags.password_flag {
            let password = read_string(buf).expect("Failed to parse password in Will Properties");
            connect_payload.password = Some(password);
        }
        Ok(Connect {
            fixed_header,
            variable_header: connect_variable_header,
            payload: connect_payload,
        })
    }
}

/// CONNECT Packet
///
/// https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901033
#[derive(Debug, Clone)]
pub struct ConnectVariableHeader {
    /// Protocol
    ///
    /// [Protocol Name] is a UTF-8 Encoded String that represents the protocol name 'MQTT', The string's `offset`
    /// and `length` will not be changed by future versions of the MQTT specification
    /// position: byte 1 - byte 6
    ///
    /// [Protocol Level] of MQTT5 is 5[0x05]
    /// position: byte 7
    ///
    /// http://docs.oasis-open.org/mqtt/mqtt/v5.0/csprd02/mqtt-v5.0-csprd02.html#_Toc498345320
    pub protocol: Protocol,
    /// Connect Flags
    /// position: byte 8
    ///
    /// The Connect Flags byte contains a number of parameters specifying the behavior of the MQTT connection.
    /// It also indicates the presence or absence of fields in the payload.
    ///
    /// |  Bit |       7      |      6      |     5     | 4  | 3 |    2    |     1     |    0   |
    /// |      |User Name Flag|Password Flag|Will Retain|Will Qos|Will Flag|Clean Start|Reserved|
    /// |byte 8|      x       |      x      |     x     | x  | x |    x    |     x     |    0   |
    pub connect_flags: ConnectFlag,
    /// [Keep Alive]
    /// position: byte 9 - byte 10
    ///
    /// It is the maximum time interval that is permitted to elapse between the point at which the Client finishes
    /// transmitting one MQTT Control Packet and the point it starts sending the next. It is the responsibility of
    /// the Client to ensure that the interval between MQTT Control Packets being sent does not exceed the Keep Alive
    /// value. If Keep Alive is non-zero and in the absence of sending any other MQTT Control Packets, the Client MUST
    /// send a `PINGREQ` packet
    pub keep_alive: usize,
    pub connect_property: Mqtt5Property,
}

/// Connect Flag
///
/// http://docs.oasis-open.org/mqtt/mqtt/v5.0/csprd02/mqtt-v5.0-csprd02.html#_Toc498345323
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectFlag {
    clean_start: bool,
    will_flag: bool,
    will_qos: Qos,
    will_retain: bool,
    username_flag: bool,
    password_flag: bool,
}

impl FromToBuf<ConnectFlag> for ConnectFlag {
    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let mut connect_flags = 0b0000_0000;
        if self.clean_start {
            connect_flags |= 0b0000_0010;
        }
        if self.will_flag {
            connect_flags |= 0b0000_0100;
        }
        connect_flags |= self.will_qos.to_u8();
        if self.will_retain {
            connect_flags |= 0b0010_0000;
        }
        if self.password_flag {
            connect_flags |= 0b0100_0000;
        }
        if self.username_flag {
            connect_flags |= 0b1000_0000;
        }
        buf.put_u8(connect_flags);
        Ok(connect_flags as usize)
    }

    fn from_buf(buf: &mut BytesMut) -> Result<ConnectFlag, Error> {
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
        Ok(ConnectFlag {
            clean_start,
            will_flag,
            will_qos,
            will_retain,
            password_flag,
            username_flag,
        })
    }
}

/// CONNECT Payload
///
/// http://docs.oasis-open.org/mqtt/mqtt/v5.0/csprd02/mqtt-v5.0-csprd02.html#_Toc498345343
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectPayload {
    /// The Client Identifier (ClientID) identifies the Client to the Server. Each Client connecting
    /// to the Server has a unique ClientID. The ClientID MUST be used by Clients and by Servers to
    /// identify state that they hold relating to this MQTT Session between the Client and the Server
    client_id: String,
    will_property: Option<Mqtt5Property>,
    will_topic: Option<String>,
    will_payload: Option<Vec<u8>>,
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
}


/// Connect Reason Code
///
/// http://docs.oasis-open.org/mqtt/mqtt/v5.0/csprd02/mqtt-v5.0-csprd02.html#_Toc498345364
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
    use crate::packet::PacketType;
    use crate::FromToU8;

    #[test]
    fn test_take() {
        use bytes::{Buf, BufMut, buf::BufExt};

        let mut buf = b"hello world"[..].take(100);
        let mut dst = vec![];

        dst.put(&mut buf);
        println!("{}", dst.len());

        let mut buf = buf.into_inner();
        dst.clear();
        dst.put(&mut buf);
        println!("{}", buf.len());
    }
}








