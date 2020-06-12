use crate::{Error, FromToU8, FromToBuf};
use crate::protocol::Protocol;
use std::collections::LinkedList;
use crate::publish::Qos;
use bytes::{BytesMut, BufMut};

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
pub struct Connect {
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
    pub connect_property: ConnectProperty,
    pub connect_payload: ConnectPayload
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
    property_length: i32,
    /// Session Expiry Interval
    ///
    /// unit: seconds
    session_expiry_interval: Option<usize>,
    /// Receive Maximum
    ///
    /// The Client uses this value to limit the number of QoS 1 and QoS 2 publications that it is willing to
    /// process concurrently. There is no mechanism to limit the QoS 0 publications that the Server might try
    /// to send.The value of Receive Maximum applies only to the current Network Connection. If the Receive
    /// Maximum value is absent then its value defaults to 65,535.
    receive_maximum: usize,
    /// Maximum Packet Size
    ///
    /// Representing the Maximum Packet Size the Client is willing to accept. If the Maximum Packet Size is
    /// not present, no limit on the packet size is imposed beyond the limitations in the protocol as a result
    /// of the remaining length encoding and the protocol header sizes
    maximum_packet_size: Option<usize>,
    /// Topic Alias Maximum
    ///
    /// representing the Topic Alias Maximum value. It is a Protocol Error to include the Topic Alias Maximum
    /// value more than once. If the Topic Alias Maximum property is absent, the default value is 0.
    topic_alias_maximum: usize,
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

/// CONNECT Payload
///
/// http://docs.oasis-open.org/mqtt/mqtt/v5.0/csprd02/mqtt-v5.0-csprd02.html#_Toc498345343
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectPayload {
    client_id: Option<String>,
    will_property: Option<WillProperty>,
    will_topic: Option<String>,
    will_payload: Option<Vec<u8>>,
    username: Option<String>,
    password: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WillProperty {
    /// Property Length
    ///
    /// The length of the Properties in the Will Properties encoded as a Variable Byte Integer.
    property_length: i32,
    /// Will Delay Interval
    ///
    /// unit: seconds
    will_delay_interval: usize,
    /// Payload Format Indicator
    ///
    /// 0 (0x00) Byte Indicates that the Will Message is unspecified bytes,
    ///   which is equivalent to not sending a Payload Format Indicator.
    /// 1 (0x01) Byte Indicates that the Will Message is UTF-8 Encoded Character Data.
    ///   The UTF-8 data in the Payload MUST be well-formed UTF-8 as defined by the [Unicode specification] and restated in [RFC 3629]
    payload_format_indicator: u8,
    /// Message Expiry Interval
    ///
    /// If present, the Four Byte value is the lifetime of the Will Message in seconds and is sent as the Publication Expiry Interval when the Server publishes the Will Message.
    /// If absent, no Message Expiry Interval is sent when the Server publishes the Will Message.
    message_expiry_interval: Option<usize>,
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


impl FromToBuf<Connect> for Connect {


    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        unimplemented!()
    }

    fn from_buf(buf: &mut BytesMut) -> Result<Connect, Error> {
        unimplemented!()
    }
}











