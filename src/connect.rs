use crate::{Error, FromToU8, FromToBuf};
use crate::protocol::Protocol;
use std::collections::LinkedList;
use crate::publish::Qos;
use bytes::{BytesMut, BufMut, Buf};
use crate::PropertyType;
use crate::decoder::{read_string, read_variable_byte_integer};

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

/// CONNECT Packet
///
/// https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901033
#[derive(Debug, Clone, PartialEq)]
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
    pub connect_flag: ConnectFlag,
    /// [Keep Alive]
    /// position: byte 9 - byte 10
    ///
    /// It is the maximum time interval that is permitted to elapse between the point at which the Client finishes
    /// transmitting one MQTT Control Packet and the point it starts sending the next. It is the responsibility of
    /// the Client to ensure that the interval between MQTT Control Packets being sent does not exceed the Keep Alive
    /// value. If Keep Alive is non-zero and in the absence of sending any other MQTT Control Packets, the Client MUST
    /// send a `PINGREQ` packet
    pub keep_alive: u16,
    pub connect_property: ConnectProperty
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

/// CONNECT Properties
///
/// http://docs.oasis-open.org/mqtt/mqtt/v5.0/csprd02/mqtt-v5.0-csprd02.html#_Toc498345331
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectProperty {
    /// Property Length
    ///
    /// The length of the Properties in the `CONNECT` packet Variable Header encoded as a Variable Byte Integer
    property_length: usize,
    /// Session Expiry Interval
    ///
    /// unit: seconds
    session_expiry_interval: Option<u32>,
    /// Receive Maximum
    ///
    /// The Client uses this value to limit the number of QoS 1 and QoS 2 publications that it is willing to
    /// process concurrently. There is no mechanism to limit the QoS 0 publications that the Server might try
    /// to send.The value of Receive Maximum applies only to the current Network Connection. If the Receive
    /// Maximum value is absent then its value defaults to 65,535.
    receive_maximum: u16,
    /// Maximum Packet Size
    ///
    /// Representing the Maximum Packet Size the Client is willing to accept. If the Maximum Packet Size is
    /// not present, no limit on the packet size is imposed beyond the limitations in the protocol as a result
    /// of the remaining length encoding and the protocol header sizes
    maximum_packet_size: Option<u32>,
    /// Topic Alias Maximum
    ///
    /// representing the Topic Alias Maximum value. It is a Protocol Error to include the Topic Alias Maximum
    /// value more than once. If the Topic Alias Maximum property is absent, the default value is 0.
    topic_alias_maximum: u16,
    /// Request Response Information
    ///
    /// It is Protocol Error to include the Request Response Information more than once, or to have a value
    /// other than 0 or 1. If the Request Response Information is absent, the value of 0 is used.
    request_response_information: bool,
    /// Request Problem Information
    ///
    /// It is a Protocol Error to include Request Problem Information more than once, or to have a value other
    /// than 0 or 1. If the Request Problem Information is absent, the value of 1 is used.
    request_problem_information: bool,
    /// User Property
    ///
    /// User Property is allowed to appear multiple times to represent multiple name, value pairs. The same
    /// name is allowed to appear more than once
    user_properties: LinkedList<(String, String)>,
    /// Authentication Method
    ///
    /// It's a UTF-8 Encoded String containing the name of the authentication method used for extended
    /// authentication .It is a Protocol Error to include Authentication Method more than once.
    /// If Authentication Method is absent, extended authentication is not performed
    authentication_method: Option<String>,
    /// Authentication Data
    ///
    /// It's a Binary Data containing authentication data. It is a Protocol Error to include Authentication
    /// Data if there is no Authentication Method. It is a Protocol Error to include Authentication Data more than once
    authentication_data: Option<Vec<u8>>,
}

impl ConnectProperty {

    fn new() -> ConnectProperty {
        ConnectProperty {
            property_length: 0,
            session_expiry_interval: None,
            receive_maximum: 0,
            maximum_packet_size: None,
            topic_alias_maximum: 0,
            request_response_information: false,
            request_problem_information: false,
            user_properties: LinkedList::<(String, String)>::new(),
            authentication_method: None,
            authentication_data: None
        }
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
    will_property: Option<WillProperty>,
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

impl FromToBuf<ConnectPayload> for ConnectPayload {

    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        unimplemented!()
    }

    fn from_buf(buf: &mut BytesMut) -> Result<Option<ConnectPayload>, Error> {
        unimplemented!()
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct WillProperty {
    /// Property Length
    ///
    /// The length of the Properties in the Will Properties encoded as a Variable Byte Integer.
    property_length: usize,
    /// Will Delay Interval
    ///
    /// unit: seconds
    will_delay_interval: u32,
    /// Payload Format Indicator
    ///
    /// 0 (0x00) Byte Indicates that the Will Message is unspecified bytes,
    ///   which is equivalent to not sending a Payload Format Indicator.
    /// 1 (0x01) Byte Indicates that the Will Message is UTF-8 Encoded Character Data.
    ///   The UTF-8 data in the Payload MUST be well-formed UTF-8 as defined by the [Unicode specification] and restated in [RFC 3629]
    payload_format_indicator: bool,
    /// Message Expiry Interval
    ///
    /// If present, the Four Byte value is the lifetime of the Will Message in seconds and is sent as the Publication Expiry Interval when the Server publishes the Will Message.
    /// If absent, no Message Expiry Interval is sent when the Server publishes the Will Message.
    message_expiry_interval: Option<u32>,
    /// Content Type
    ///
    /// It's a UTF-8 Encoded String describing the content of the Will Message, The value of the
    /// Content Type is defined by the sending and receiving application.
    content_type: String,
    /// Response Topic
    ///
    /// The presence of a Response Topic identifies the Will Message as a Request.
    response_topic: Option<String>,
    /// Correlation Data
    ///
    /// The Correlation Data is used by the sender of the Request Message to identify which request the Response Message is for when it is received.
    /// If the Correlation Data is not present, the Requester does not require any correlation data.
    correlation_data: Option<Vec<u8>>,
    /// User Properties
    ///
    /// The User Property is allowed to appear multiple times to represent multiple name, value pairs. The same name is allowed to appear more than once.
    user_properties: LinkedList<(String,String)>
}

impl WillProperty {

    fn new() -> WillProperty {
        WillProperty {
            property_length: 0,
            will_delay_interval: 0,
            payload_format_indicator: false,
            message_expiry_interval: Some(0),
            content_type: String::default(),
            response_topic: None,
            correlation_data: None,
            user_properties: LinkedList::new()
        }
    }
}

impl FromToBuf<WillProperty> for WillProperty {


    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        unimplemented!()
    }

    fn from_buf(buf: &mut BytesMut) -> Result<Option<WillProperty>, Error> {
        let property_length = read_variable_byte_integer(buf);
        let mut prop_len: usize = 0;
        let mut will_property = WillProperty::new();
        while property_length > prop_len {
            let property_id = read_variable_byte_integer(buf).unwrap();
            match property_id {
                0x18 => {
                    will_property.will_delay_interval = buf.get_u32();
                    prop_len += 4;
                },
                0x01 => {
                    will_property.payload_format_indicator = buf.get_u8() & 0x01 == 1;
                    prop_len += 1;
                },
                0x02 => {
                    will_property.message_expiry_interval = Some(buf.get_u32());
                    prop_len += 4;
                },
                0x03 => {
                    will_property.content_type = read_string(buf).unwrap();
                    prop_len += will_property.content_type.into_bytes().len();
                },
                0x08 => {
                    let response_topic = read_string(buf).unwrap();
                    prop_len += response_topic.into_bytes().len();
                    will_property.response_topic = Some(response_topic);
                },
                0x09 => {
                    let length = buf.get_u16() as usize;
                    let data  = buf.split_to(length);
                    will_property.correlation_data = Some(data.to_vec());
                    prop_len += length;
                },
                _ => return Err(Error::InvalidPropertyType)
            }
        }
        return Ok(Some(will_property))

    }
}


impl FromToBuf<ConnectVariableHeader> for Connect {


    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        unimplemented!()
    }

    fn from_buf(buf: &mut BytesMut) -> Result<Option<ConnectVariableHeader>, Error> {
        let protocol_name = read_string(buf)?;
        let protocol_level = buf.get_u8();
        let protocol = Protocol::new(&protocol_name, protocol_level).unwrap();
        let connect_flags = ConnectFlag::from_buf(buf);
        let keep_alive = buf.get_u16();
        unimplemented!()

    }
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

    fn from_buf(buf: &mut BytesMut) -> Result<Option<ConnectFlag>, Error> {
        let connect_flags = buf.get_u8();
        let clean_start = (connect_flags >> 1) & 0x01 > 0;
        let will_flag = (connect_flags >> 2) & 0x01 > 0;
        let will_qos = Qos::from_u8((connect_flags >> 3) & 0x03).unwrap();
        let will_retain = (connect_flags >> 5) & 0x01 > 0;
        let password_flag = (connect_flags >> 6) & 0x01 > 0;
        let username_flag = (connect_flags >> 7) & 0x01 > 0;
        Ok(Some(ConnectFlag {
            clean_start,
            will_flag,
            will_qos,
            will_retain,
            password_flag,
            username_flag
        }))
    }
}

impl FromToBuf<ConnectProperty> for ConnectProperty {

    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        unimplemented!()
    }

    fn from_buf(buf: &mut BytesMut) -> Result<Option<ConnectProperty>, Error> {
        let mut property_length = read_variable_byte_integer(buf).unwrap();
        let mut prop_len: usize = 0;
        let mut connect_property = ConnectProperty::new();
        while property_length > prop_len {
            let property_id = read_variable_byte_integer(buf).unwrap();
            match property_id {
                0x11 => {
                    connect_property.session_expiry_interval = Some(buf.get_u32());
                    prop_len += 4;
                },
                0x21 => {
                    connect_property.receive_maximum = buf.get_u16();
                    prop_len += 2;
                },
                0x27 => {
                    connect_property.maximum_packet_size = Some(buf.get_u32());
                    prop_len += 4;
                },
                0x22 => {
                    connect_property.topic_alias_maximum = buf.get_u16();
                    prop_len += 2;
                },
                0x19 => {
                    connect_property.request_response_information = buf.get_u8() & 0x01 == 1;
                    prop_len += 1;
                },
                0x17 => {
                    connect_property.request_problem_information = buf.get_u8() & 0x01 == 1;
                    prop_len += 1;
                },
                0x26 => {
                    let name = read_string(buf).unwrap();
                    let value = read_string(buf).unwrap();
                    prop_len += name.into_bytes().len();
                    prop_len += value.into_bytes().len();
                    connect_property.user_properties.push_back((name, value));
                },
                0x15 => {
                    let authentication_method = read_string(buf).unwrap();
                    prop_len += authentication_method.into_bytes().len();
                    connect_property.authentication_method = Some(authentication_method);
                },
                0x16 => {
                    let length = buf.get_u16() as usize;
                    let data = buf.split_to(length);
                    connect_property.authentication_data = Some(data.to_vec());
                    prop_len += length;
                },
                _ => {
                    return Err(Error::InvalidPropertyType);
                }
            }
        }
        Ok(Some(connect_property))
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn test() {
        let num = 1 >> 1;

        println!("{:?}", num);
    }
}









