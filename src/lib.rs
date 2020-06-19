mod packet;
mod frame;
mod publish;
mod protocol;
mod connect;
mod connack;
mod error;
mod decoder;
pub use error::Error;
use bytes::{BufMut, BytesMut, Bytes, Buf};
use std::collections::{LinkedList, HashMap};
use crate::decoder::{read_variable_byte, read_string};

trait FromToU8<R> {
    fn to_u8(&self) -> u8;
    fn from_u8(byte: u8) -> Result<R, Error>;
}

trait FromToBuf<R> {
    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error>;
    fn from_buf(buf: &mut BytesMut) -> Result<R, Error>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mqtt5Property {
    property_length: usize,
    properties: HashMap<u8, PropertyValue>,
}

impl Mqtt5Property {

    fn new() -> Mqtt5Property {
        Mqtt5Property {
            property_length: 0,
            properties: HashMap::<u8, PropertyValue>::default(),
        }
    }
}

impl FromToBuf<Mqtt5Property> for Mqtt5Property{

    fn to_buf(&self, _buf: &mut impl BufMut) -> Result<usize, Error> {
        unimplemented!()
    }

    fn from_buf(buf: &mut BytesMut) -> Result<Mqtt5Property, Error> {
        let property_length = read_variable_byte(buf)
            .expect("Failed to parse Mqtt5 Property length");
        let mut prop_len: usize = 0;
        let mut property = Mqtt5Property::new();
        property.property_length = property_length;
        let mut user_properties = LinkedList::<(String, String)>::new();

        while property_length > prop_len {
            let property_id = read_variable_byte(buf)
                .expect("Failed to parse Property Id");
            prop_len += 1;
            match property_id {
                // Payload Format Indicator -> Will
                0x01 => {
                    if property.properties.contains_key(&0x01) {
                        return Err(Error::InvalidProtocol("Cannot contains [Payload Format Indicator] more than once".to_string(), 0x18));
                    }
                    property.properties.insert(0x01, PropertyValue::Bit(buf.get_u8() & 0x01 == 1));
                    prop_len += 1;
                }
                // Message Expiry Interval -> Will
                0x02 => {
                    if property.properties.contains_key(&0x02) {
                        return Err(Error::InvalidProtocol("Cannot contains [Message Expiry Interval] more than once".to_string(), 0x02));
                    }
                    // If present, the Four Byte value is the lifetime of the Will Message in seconds and is sent as the Publication Expiry Interval when the Server publishes the Will Message.
                    // If absent, no Message Expiry Interval is sent when the Server publishes the Will Message.
                    property.properties.insert(0x02, PropertyValue::FourByteInteger(buf.get_u32()));
                    prop_len += 4;
                }
                // Content Type -> Will
                0x03 => {
                    if property.properties.contains_key(&0x03) {
                        return Err(Error::InvalidProtocol("Cannot contains [Content Type] more than once".to_string(), 0x03));
                    }
                    let content_type = read_string(buf).expect("Failed to parse Content Type");
                    prop_len += content_type.clone().into_bytes().len();
                    property.properties.insert(0x03, PropertyValue::String(content_type));
                }
                // Response Topic -> Will
                0x08 => {
                    if property.properties.contains_key(&0x08) {
                        return Err(Error::InvalidProtocol("Cannot contains [Response Topic] more than once".to_string(), 0x08));
                    }
                    let response_topic = read_string(buf).expect("Failed to parse Response Topic");
                    prop_len += response_topic.clone().into_bytes().len();
                    property.properties.insert(0x08, PropertyValue::String(response_topic));
                }
                // Correlation Data -> Will
                0x09 => {
                    if property.properties.contains_key(&0x09) {
                        return Err(Error::InvalidProtocol("Cannot contains [Correlation Data] more than once".to_string(), 0x09));
                    }
                    let length = buf.get_u16() as usize;
                    prop_len += length;
                    let mut data = buf.split_to(length);
                    property.properties.insert(0x09, PropertyValue::Binary(data.to_bytes()));
                }
                // Session Expiry Interval -> Connect
                0x11 => {
                    if property.properties.contains_key(&0x11) {
                        return Err(Error::InvalidProtocol("Cannot contains [Session Expire Interval] more than once".to_string(), 0x11));
                    }
                    property.properties.insert(0x11, PropertyValue::FourByteInteger(buf.get_u32()));
                    prop_len += 4;
                }
                // Authentication Method -> Connect
                0x15 => {
                    if property.properties.contains_key(&0x15) {
                        return Err(Error::InvalidProtocol("Cannot contains [Authentication Method] more than once".to_string(), 0x15));
                    }
                    let authentication_method = read_string(buf).expect("Failed to parse Authentication Method");
                    prop_len += authentication_method.clone().into_bytes().len();
                    property.properties.insert(0x15, PropertyValue::String(authentication_method));
                }
                // Authentication Data -> Connect
                0x16 => {
                    if property.properties.contains_key(&0x16) {
                        return Err(Error::InvalidProtocol("Cannot contains [Authentication Data] more than once".to_string(), 0x16));
                    }
                    let length = buf.get_u16() as usize;
                    prop_len += length;
                    let mut data = buf.split_to(length);
                    property.properties.insert(0x16, PropertyValue::Binary(data.to_bytes()));
                }
                // Request Problem Information -> Connect
                0x17 => {
                    let request_problem_information = buf.get_u8();
                    if property.properties.contains_key(&0x17) ||
                        (request_problem_information != 0 && request_problem_information != 1) {
                        return Err(Error::InvalidProtocol("Cannot contains [Request Problem Information] more than once or to have a value other than 0 or 1".to_string(), 0x17));
                    }
                    property.properties.insert(0x17, PropertyValue::Bit(request_problem_information & 0x01 == 1));
                    prop_len += 1;
                }
                // Will Delay Interval -> Will
                0x18 => {
                    if property.properties.contains_key(&0x18) {
                        return Err(Error::InvalidProtocol("Will properties cannot contains [Will Delay Interval] more than once".to_string(), 0x18));
                    }
                    property.properties.insert(0x18, PropertyValue::FourByteInteger(buf.get_u32()));
                    prop_len += 4;
                }
                // Request Response Information -> Connect
                0x19 => {
                    let request_response_information = buf.get_u8();
                    if property.properties.contains_key(&0x19) ||
                        (request_response_information != 0 && request_response_information != 1) {
                        return Err(Error::InvalidProtocol("Cannot contains [Request Response Information] more than once or to have a value other than 0 or 1".to_string(), 0x19));
                    }
                    property.properties.insert(0x19, PropertyValue::Bit(request_response_information & 0x01 == 1));
                    prop_len += 1;
                }
                // Receive Maximum -> Connect
                0x21 => {
                    // It is a Protocol Error to include the Receive Maximum value more than once or for receive maximum to have the value 0
                    let receive_maximum = buf.get_u16();
                    if property.properties.contains_key(&0x21) || receive_maximum == 0 {
                        return Err(Error::InvalidProtocol("Cannot contains [Receive Maximum] more than once or for it have the value 0".to_string(), 0x21));
                    }
                    // The Client uses this value to limit the number of QoS 1 and QoS 2 publications that it is willing to process concurrently.
                    // There is no mechanism to limit the QoS 0 publications that the Server might try to send
                    property.properties.insert(0x21, PropertyValue::TwoByteInteger(receive_maximum));
                    prop_len += 2;
                }
                // Topic Alias Maximum -> Connect
                0x22 => {
                    //  It is a Protocol Error to include the Topic Alias Maximum value more than once.
                    if property.properties.contains_key(&0x22) {
                        return Err(Error::InvalidProtocol("Cannot contains [Topic Alias Maximum] more than once".to_string(), 0x22));
                    }
                    property.properties.insert(0x22, PropertyValue::TwoByteInteger(buf.get_u16()));
                    prop_len += 2;
                }
                // User Property -> Connect, Will
                0x26 => {
                    // The User Property is allowed to appear multiple times to represent multiple name, value pairs.
                    // The same name is allowed to appear more than once.
                    let name = read_string(buf).expect("Failed to parse User property");
                    let value = read_string(buf).expect("Failed to parse User property");
                    prop_len += name.clone().into_bytes().len();
                    prop_len += value.clone().into_bytes().len();
                    user_properties.push_back((name, value));
                }
                // Maximum Packet Size -> Connect
                0x27 => {
                    // It is a Protocol Error to include the Maximum Packet Size more than once, or for the value to be set to zero.
                    // If the Maximum Packet Size is not present, no limit on the packet size is imposed beyond the limitations in the
                    // protocol as a result of the remaining length encoding and the protocol header sizes.
                    let maximum_packet_size = buf.get_u32();
                    if property.properties.contains_key(&0x27) || maximum_packet_size == 0{
                        return Err(Error::InvalidProtocol("Cannot contains [Maximum Packet Size] more than once or for it have the value 0".to_string(), 0x27));
                    }
                    property.properties.insert(0x27, PropertyValue::FourByteInteger(maximum_packet_size));
                    prop_len += 4;
                }
                _ => {
                    return Err(Error::InvalidPropertyType("It's a invalid Property Type".to_string()));
                }
            }
        }
        if user_properties.len() > 0 {
            property.properties.insert(0x26, PropertyValue::StringPair(user_properties));
        }
        Ok(property)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PropertyType {
    PayloadFormatIndicator,
    MessageExpiryInterval,
    ContentType,
    ResponseTopic,
    CorrelationData,
    SubscriptionIdentifier,
    SessionExpiryInterval,
    AssignedClientIdentifier,
    ServerKeepAlive,
    AuthenticationMethod,
    AuthenticationData,
    RequestProblemInformation,
    WillDelayInterval,
    RequestResponseInformation,
    ResponseInformation,
    ServerReference,
    ReasonString,
    ReceiveMaximum,
    TopicAliasMaximum,
    TopicAlias,
    MaximumQos,
    RetainAvailable,
    UserProperty,
    MaximumPacketSize,
    WildcardSubscriptionAvailable,
    SubscriptionIdentifierAvailable,
    SharedSubscriptionAvailable
}

impl FromToU8<PropertyType> for PropertyType {
    fn to_u8(&self) -> u8 {
        match *self {
            PropertyType::PayloadFormatIndicator => 1,
            PropertyType::MessageExpiryInterval => 2,
            PropertyType::ContentType => 3,
            PropertyType::ResponseTopic => 8,
            PropertyType::CorrelationData => 9,
            PropertyType::SubscriptionIdentifier => 11,
            PropertyType::SessionExpiryInterval => 17,
            PropertyType::AssignedClientIdentifier => 18,
            PropertyType::ServerKeepAlive => 19,
            PropertyType::AuthenticationMethod => 21,
            PropertyType::AuthenticationData => 22,
            PropertyType::RequestProblemInformation => 23,
            PropertyType::WillDelayInterval => 24,
            PropertyType::RequestResponseInformation => 25,
            PropertyType::ResponseInformation => 26,
            PropertyType::ServerReference => 28,
            PropertyType::ReasonString => 31,
            PropertyType::ReceiveMaximum => 33,
            PropertyType::TopicAliasMaximum => 34,
            PropertyType::TopicAlias => 35,
            PropertyType::MaximumQos => 36,
            PropertyType::RetainAvailable => 37,
            PropertyType::UserProperty => 38,
            PropertyType::MaximumPacketSize => 39,
            PropertyType::WildcardSubscriptionAvailable => 40,
            PropertyType::SubscriptionIdentifierAvailable => 41,
            PropertyType::SharedSubscriptionAvailable => 42
        }
    }

    fn from_u8(byte: u8) -> Result<PropertyType, Error> {
        match byte {
            1 => Ok(PropertyType::PayloadFormatIndicator),
            2 => Ok(PropertyType::MessageExpiryInterval),
            3 => Ok(PropertyType::ContentType),
            8 => Ok(PropertyType::ResponseTopic),
            9 => Ok(PropertyType::CorrelationData),
            11 => Ok(PropertyType::SubscriptionIdentifier),
            17 => Ok(PropertyType::SessionExpiryInterval),
            18 => Ok(PropertyType::AssignedClientIdentifier),
            19 => Ok(PropertyType::ServerKeepAlive),
            21 => Ok(PropertyType::AuthenticationMethod),
            22 => Ok(PropertyType::AuthenticationData),
            23 => Ok(PropertyType::RequestProblemInformation),
            24 => Ok(PropertyType::WillDelayInterval),
            25 => Ok(PropertyType::RequestResponseInformation),
            26 => Ok(PropertyType::ResponseInformation),
            28 => Ok(PropertyType::ServerReference),
            31 => Ok(PropertyType::ReasonString),
            33 => Ok(PropertyType::ReceiveMaximum),
            34 => Ok(PropertyType::TopicAliasMaximum),
            35 => Ok(PropertyType::TopicAlias),
            36 => Ok(PropertyType::MaximumQos),
            37 => Ok(PropertyType::RetainAvailable),
            38 => Ok(PropertyType::UserProperty),
            39 => Ok(PropertyType::MaximumPacketSize),
            40 => Ok(PropertyType::WildcardSubscriptionAvailable),
            41 => Ok(PropertyType::SubscriptionIdentifierAvailable),
            42 => Ok(PropertyType::SharedSubscriptionAvailable),
            _ => Err(Error::InvalidPropertyType("It's a invalid Property Type".to_string()))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PropertyValue {
    Bit(bool),
    Byte(u8),
    TwoByteInteger(u16),
    FourByteInteger(u32),
    String(String),
    VariableByteInteger(usize),
    Binary(Bytes),
    StringPair(LinkedList<(String, String)>)
}