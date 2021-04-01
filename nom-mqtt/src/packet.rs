use crate::reason_code::{
    AuthenticateReasonCode, ConnectReasonCode, DisconnectReasonCode, PubAckReasonCode,
    PubCompReasonCode, PubRecReasonCode, PubRelReasonCode, SubscribeReasonCode,
    UnSubscribeReasonCode,
};
use std::collections::HashMap;
use std::num::NonZeroU16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    MQTT5,
}

/// # Fixed Header format
///
/// <table style="border: 0" cellspacing="0" cellpadding="0">
///  <tbody><tr>
///   <td style="width:300px; height: 25px;border: 1px #A4A4A4 solid">
///   <p align="center" style="text-align:center"><b>Bit</b></p>
///   </td>
///   <td style="width:300px;border: 1px #A4A4A4 solid">
///   <p align="center" style="text-align:center"><b>7</b></p>
///   </td>
///   <td style="width:300px;border: 1px #A4A4A4 solid">
///   <p align="center" style="text-align:center"><b>6</b></p>
///   </td>
///   <td style="width:300px;border: 1px #A4A4A4 solid">
///   <p align="center" style="text-align:center"><b>5</b></p>
///   </td>
///   <td style="width:300px;border: 1px #A4A4A4 solid">
///   <p align="center" style="text-align:center"><b>4</b></p>
///   </td>
///   <td style="width:300px;border: 1px #A4A4A4 solid">
///   <p align="center" style="text-align:center"><b>3</b></p>
///   </td>
///   <td style="width:300px;border: 1px #A4A4A4 solid">
///   <p align="center" style="text-align:center"><b>2</b></p>
///   </td>
///   <td style="width:300px;border: 1px #A4A4A4 solid">
///   <p align="center" style="text-align:center"><b>1</b></p>
///   </td>
///   <td style="width:300px;border: 1px #A4A4A4 solid">
///   <p align="center" style="text-align:center"><b>0</b></p>
///   </td>
///  </tr>
///  <tr>
///   <td style="width:300px;height: 25px;border: 1px #A4A4A4 solid">
///   <p style="text-align:center">byte 1</p>
///   </td>
///   <td style="width:300px;border: 1px #A4A4A4 solid" colspan="4">
///   <p align="center" style="text-align:center" >MQTT Control Packet
///   type</p>
///   </td>
///   <td style="width:300px;border: 1px #A4A4A4 solid" colspan="4">
///   <p align="center" style="text-align:center">Flags specific to
///   each MQTT Control Packet type</p>
///   </td>
///  </tr>
///  <tr>
///   <td style="width:300px;height: 25px;border: 1px #A4A4A4 solid">
///   <p style="text-align:center">byte 2…</p>
///   </td>
///   <td style="width:300px;border: 1px #A4A4A4 solid" colspan="8">
///   <p align="center" style="text-align:center">Remaining Length</p>
///   </td>
///  </tr>
/// </tbody></table>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedHeader {
    pub packet_type: PacketType,
    pub dup: bool,
    pub qos: Qos,
    pub retain: bool,
    /// This is the length of the Variable Header plus the length of the Payload. It is encoded as a Variable Byte Integer
    pub remaining_length: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Publish<'a> {
    pub fixed_header: FixedHeader,
    pub variable_header: PublishVariableHeader<'a>,
    pub payload: &'a [u8],
}

#[derive(Debug, Clone, PartialEq)]
pub struct PublishVariableHeader<'a> {
    pub topic_name: &'a str,
    pub packet_id: u16,
    pub publish_property: Mqtt5Property<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PubAck<'a> {
    pub fixed_header: FixedHeader,
    pub variable_header: PubAckVariableHeader<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PubAckVariableHeader<'a> {
    pub packet_id: u16,
    pub puback_reason_code: PubAckReasonCode,
    pub puback_property: Mqtt5Property<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PubRec<'a> {
    pub fixed_header: FixedHeader,
    pub variable_header: PubRecVariableHeader<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PubRecVariableHeader<'a> {
    pub packet_id: u16,
    pub pubrec_reason_code: PubRecReasonCode,
    pub pubrec_property: Mqtt5Property<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PubRel<'a> {
    pub fixed_header: FixedHeader,
    pub variable_header: PubRelVariableHeader<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PubRelVariableHeader<'a> {
    pub packet_id: u16,
    pub pubrel_reason_code: PubRelReasonCode,
    pub pubrel_property: Mqtt5Property<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PubComp<'a> {
    pub fixed_header: FixedHeader,
    pub variable_header: PubCompVariableHeader<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PubCompVariableHeader<'a> {
    pub packet_id: u16,
    pub pubcomp_reason_code: PubCompReasonCode,
    pub pubcomp_property: Mqtt5Property<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Subscribe<'a> {
    pub fixed_header: FixedHeader,
    pub variable_header: SubscribeVariableHeader<'a>,
    // (topic filter, subscription options)
    pub payload: Vec<(&'a str, SubscriptionOptions)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubscribeVariableHeader<'a> {
    pub packet_id: u16,
    pub subscribe_property: Mqtt5Property<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubscriptionOptions {
    pub maximum_qos: Qos,
    pub no_local: bool,
    pub retain_as_published: bool,
    pub retain_handling: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubAck<'a> {
    pub fixed_header: FixedHeader,
    pub variable_header: SubAckVariableHeader<'a>,
    pub payload: Vec<SubscribeReasonCode>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubAckVariableHeader<'a> {
    pub packet_id: u16,
    pub suback_property: Mqtt5Property<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnSubscribe<'a> {
    pub fixed_header: FixedHeader,
    pub variable_header: UnSubscribeVariableHeader<'a>,
    pub payload: Vec<&'a str>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnSubscribeVariableHeader<'a> {
    pub packet_id: u16,
    pub unsubscribe_property: Mqtt5Property<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnSubAck<'a> {
    pub fixed_header: FixedHeader,
    pub variable_header: UnSubAckVariableHeader<'a>,
    pub payload: Vec<UnSubscribeReasonCode>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnSubAckVariableHeader<'a> {
    pub packet_id: u16,
    pub unsuback_property: Mqtt5Property<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PingReq {
    pub fixed_header: FixedHeader,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PingResp {
    pub fixed_header: FixedHeader,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Disconnect<'a> {
    pub fixed_header: FixedHeader,
    pub variable_header: DisconnectVariableHeader<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DisconnectVariableHeader<'a> {
    pub disconnect_reason_code: DisconnectReasonCode,
    pub disconnect_property: Mqtt5Property<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Auth<'a> {
    pub fixed_header: FixedHeader,
    pub variable_header: AuthVariableHeader<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuthVariableHeader<'a> {
    pub auth_reason_code: AuthenticateReasonCode,
    pub auth_property: Mqtt5Property<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Connect<'a> {
    pub fixed_header: FixedHeader,
    pub variable_header: ConnectVariableHeader<'a>,
    pub payload: ConnectPayload<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConnectVariableHeader<'a> {
    pub protocol: Protocol,
    pub connect_flags: ConnectFlags,
    pub keep_alive: u16,
    pub connect_property: Mqtt5Property<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConnectFlags {
    pub clean_start: bool,
    pub will_flag: bool,
    pub will_qos: Qos,
    pub will_retain: bool,
    pub username_flag: bool,
    pub password_flag: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConnectPayload<'a> {
    pub client_id: &'a str,
    pub will_property: Option<Mqtt5Property<'a>>,
    pub will_topic: Option<&'a str>,
    pub will_payload: Option<&'a [u8]>,
    pub username: Option<&'a str>,
    pub password: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConnAck<'a> {
    pub fixed_header: FixedHeader,
    pub variable_header: ConnAckVariableHeader<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConnAckVariableHeader<'a> {
    pub connack_flags: ConnAckFlags,
    pub connect_reason_code: ConnectReasonCode,
    pub connack_property: Mqtt5Property<'a>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConnAckFlags {
    pub session_present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PacketType {
    /// 1, Client to server, Connection request
    CONNECT = 1,
    /// 2, Server to client, Connect acknowledgement
    CONNACK = 2,
    /// 3, Two-way, Publish message
    PUBLISH = 3,
    /// 4, Two-way, Publish acknowledgement (QoS 1)
    PUBACK = 4,
    /// 5, Two-way, Publish received (QoS 2 delivery part 1)
    PUBREC = 5,
    /// 6, Two-way, Publish release (QoS 2 delivery part 2)
    PUBREL = 6,
    /// 7, Two-way, Publish complete (Qos 2 delivery part 3)
    PUBCOMP = 7,
    /// 8, Client to server, Subscribe request
    SUBSCRIBE = 8,
    /// 9, Server to client, Subscribe acknowledgement
    SUBACK = 9,
    /// 10, Client to server, Unsubscribe request
    UNSUBSCRIBE = 10,
    /// 11, Server to client, Unsubscribe acknowledgement
    UNSUBACK = 11,
    /// 12, Client to server, Ping request
    PINGREQ = 12,
    /// 13, Server to client, Ping response
    PINGRESP = 13,
    /// 14, Two-way, Disconnect notification
    DISCONNECT = 14,
    /// 15, Two-way, Authentication exchange
    AUTH = 15,
}

impl From<u8> for PacketType {
    fn from(byte: u8) -> Self {
        match byte {
            1 => PacketType::CONNECT,
            2 => PacketType::CONNACK,
            3 => PacketType::PUBLISH,
            4 => PacketType::PUBACK,
            5 => PacketType::PUBREC,
            6 => PacketType::PUBREL,
            7 => PacketType::PUBCOMP,
            8 => PacketType::SUBSCRIBE,
            9 => PacketType::SUBACK,
            10 => PacketType::UNSUBSCRIBE,
            11 => PacketType::UNSUBACK,
            12 => PacketType::PINGREQ,
            13 => PacketType::PINGRESP,
            14 => PacketType::DISCONNECT,
            15 => PacketType::AUTH,
            _ => unimplemented!("no other packet type supported"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Qos {
    /// Qos value: 0
    AtMostOnce = 0,
    /// Qos value: 1
    AtLeastOnce = 1,
    /// Qos value: 2
    ExactlyOnce = 2,
    /// Reserved, must not be used
    Reserved = 3,
}

impl From<u8> for Qos {
    fn from(byte: u8) -> Self {
        match byte {
            0 => Qos::AtMostOnce,
            1 => Qos::AtLeastOnce,
            2 => Qos::ExactlyOnce,
            _ => unimplemented!("no other qos supported"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mqtt5Property<'a> {
    pub property_length: usize,
    pub properties: HashMap<u32, PropertyValue<'a>>,
}

impl<'a> Mqtt5Property<'a> {
    pub fn new() -> Self {
        Mqtt5Property {
            property_length: 0,
            properties: HashMap::<u32, PropertyValue>::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PropertyType {
    PayloadFormatIndicator = 0x01,
    MessageExpiryInterval = 0x02,
    ContentType = 0x03,
    ResponseTopic = 0x08,
    CorrelationData = 0x09,
    SubscriptionIdentifier = 0x0B,
    SessionExpiryInterval = 0x11,
    AssignedClientIdentifier = 0x12,
    ServerKeepAlive = 0x13,
    AuthenticationMethod = 0x15,
    AuthenticationData = 0x16,
    RequestProblemInformation = 0x17,
    WillDelayInterval = 0x18,
    RequestResponseInformation = 0x19,
    ResponseInformation = 0x1A,
    ServerReference = 0x1C,
    ReasonString = 0x1F,
    ReceiveMaximum = 0x21,
    TopicAliasMaximum = 0x22,
    TopicAlias = 0x23,
    MaximumQos = 0x24,
    RetainAvailable = 0x25,
    UserProperty = 0x26,
    MaximumPacketSize = 0x27,
    WildcardSubscriptionAvailable = 0x28,
    SubscriptionIdentifierAvailable = 0x29,
    SharedSubscriptionAvailable = 0x2A,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PropertyValue<'a> {
    Bit(bool),
    Byte(u8),
    TwoByteInteger(u16),
    FourByteInteger(u32),
    String(&'a str),
    VariableByteInteger(usize),
    Binary(&'a [u8]),
    StringPair(&'a str, &'a str),
    Multiple(Vec<PropertyValue<'a>>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Packet<'a> {
    Connect(Connect<'a>),
    ConnAck(ConnAck<'a>),
    Publish(Publish<'a>),
    PubAck(PubAck<'a>),
    PubRec(PubRec<'a>),
    PubRel(PubRel<'a>),
    PubComp(PubComp<'a>),
    Subscribe(Subscribe<'a>),
    SubAck(SubAck<'a>),
    UnSubscribe(UnSubscribe<'a>),
    UnSubAck(UnSubAck<'a>),
    PingReq(PingReq),
    PingResp(PingResp),
    Disconnect(Disconnect<'a>),
    Auth(Auth<'a>),
}
