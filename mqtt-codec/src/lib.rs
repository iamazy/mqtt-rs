pub mod packet;
pub mod fixed_header;
pub mod publish;
pub mod puback;
pub mod pubrec;
pub mod pubrel;
pub mod pubcomp;
pub mod subscribe;
pub mod suback;
pub mod unsubscribe;
pub mod unsuback;
pub mod pingreq;
pub mod pingresp;
pub mod disconnect;
pub mod auth;
pub mod protocol;
pub mod connect;
pub mod connack;
pub mod error;

pub use error::Error;
use bytes::{BufMut, BytesMut, Bytes, Buf};
use std::collections::HashMap;

pub trait FromToU8<R> {
    fn to_u8(&self) -> u8;
    fn from_u8(byte: u8) -> Result<R, Error>;
}

pub trait FromToBuf<R> {
    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error>;
    fn from_buf(buf: &mut BytesMut) -> Result<R, Error>;
}

pub fn write_string(string: String, buf: &mut impl BufMut) -> usize {
    write_bytes(Bytes::from(string), buf)
}

pub fn read_string(buf: &mut BytesMut) -> Result<String, Error> {
    String::from_utf8(read_bytes(buf)?.to_vec()).map_err(|e| Error::InvalidString(e.utf8_error().to_string()))
}

pub fn write_bytes(bytes: Bytes, buf: &mut impl BufMut) -> usize {
    let len = bytes.len();
    assert!(len <= 65535, "Bytes length must less than or equal 65535");
    buf.put_u16(len as u16);
    buf.put_slice(bytes.bytes());
    len + 2
}

pub fn read_bytes(buf: &mut BytesMut) -> Result<Bytes, Error> {
    let len = buf.get_u16() as usize;
    if len > buf.remaining() {
        Err(Error::InvalidLength)
    } else {
        Ok(buf.split_to(len).to_bytes())
    }
}

/// # Examples
/// ```
/// use bytes::BytesMut;
/// use mqtt_codec::write_variable_bytes;
///
/// let mut buf = BytesMut::with_capacity(2);
/// let len = write_variable_bytes(136, &mut buf);
/// assert_eq!(len, 2);
/// assert_eq!(buf.to_vec(), [127,1])
/// ```
pub fn write_variable_bytes2(mut value: usize, buf: &mut impl BufMut) -> Result<usize, Error>{
    let mut len = 0;
    while value > 0 {
        let mut encoded_byte: u8 = (value % 0x7F) as u8;
        value = value / 0x7F;
        if value > 0 {
            encoded_byte |= 0x7F;
        }
        buf.put_u8(encoded_byte);
        len += 1;
    }
    Ok(len)
}

pub fn write_variable_bytes<T>(mut value: usize, mut callback: T) -> Result<usize, Error>
    where T: FnMut(u8)
{
    let mut len = 0;
    while value > 0 {
        let mut encoded_byte: u8 = (value % 0x7F) as u8;
        value = value / 0x7F;
        if value > 0 {
            encoded_byte |= 0x7F;
        }
        callback(encoded_byte);
        len += 1;
    }
    Ok(len)
}

/// First usize is function returned value
/// Second usize is the number of consumed bytes
///
/// # Examples
/// ```
/// use bytes::{BytesMut, BufMut};
/// use mqtt_codec::read_variable_bytes;
///
/// let byte1 = 0b1000_1000;
/// let byte2 = 0b0000_0001;
/// let mut buf = BytesMut::with_capacity(2);
/// buf.put_u8(byte1);
/// buf.put_u8(byte2);
/// let value = read_variable_bytes(&mut buf).unwrap();
/// assert_eq!(value, 0b1000_1000);
/// ```
pub fn read_variable_bytes(buf: &mut BytesMut) -> Result<(usize, usize), Error> {
    let mut value: usize = 0;
    for pos in 0..=3 {
        if let Some(&byte) = buf.get(pos) {
            value += (byte as usize & 0x7F) << (pos * 7);
            if (byte & 0x80) == 0 {
                buf.advance(pos + 1);
                return Ok((value, pos + 1));
            }
        }
    }
    Err(Error::MalformedVariableByteInteger)
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mqtt5Property {
    pub property_length: usize,
    pub properties: HashMap<u32, PropertyValue>,
    pub append_length: usize,
}

impl Mqtt5Property {
    pub fn new() -> Mqtt5Property {
        Mqtt5Property {
            property_length: 0,
            properties: HashMap::<u32, PropertyValue>::default(),
            append_length: 0
        }
    }
}

impl FromToBuf<Mqtt5Property> for Mqtt5Property {

    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let properties = self.properties.clone();
        write_variable_bytes(self.property_length, |byte|buf.put_u8(byte))?;
        let mut len: usize = 0;
        for (key, value) in properties {
            match value {
                PropertyValue::Bit(val) => {
                    len += write_variable_bytes(key as usize, |byte|buf.put_u8(byte))?;
                    match val {
                        true => {
                            buf.put_u8(1);
                        }
                        false => {
                            buf.put_u8(0);
                        }
                    }
                    len += 1;
                }
                PropertyValue::Byte(val) => {
                    len += write_variable_bytes(key as usize, |byte|buf.put_u8(byte))?;
                    buf.put_u8(val);
                    len += 1;
                }
                PropertyValue::TwoByteInteger(val) => {
                    len += write_variable_bytes(key as usize, |byte|buf.put_u8(byte))?;
                    buf.put_u16(val);
                    len += 2;
                }
                PropertyValue::FourByteInteger(val) => {
                    len += write_variable_bytes(key as usize, |byte|buf.put_u8(byte))?;
                    buf.put_u32(val);
                    len += 4;
                }
                PropertyValue::String(val) => {
                    len += write_variable_bytes(key as usize, |byte|buf.put_u8(byte))?;
                    len += write_string(val, buf);
                }
                PropertyValue::VariableByteInteger(val) => {
                    len += write_variable_bytes(key as usize, |byte|buf.put_u8(byte))?;
                    len += write_variable_bytes(val, |byte|buf.put_u8(byte))?;
                }
                PropertyValue::Binary(val) => {
                    len += write_variable_bytes(key as usize, |byte|buf.put_u8(byte))?;
                    len += write_bytes(val, buf);
                }
                PropertyValue::StringPair(val) => {
                    for item in val {
                        len += write_variable_bytes(key as usize, |byte|buf.put_u8(byte))?;
                        len += write_string(item.0, buf);
                        len += write_string(item.1, buf);
                    }
                }
                PropertyValue::Multiple(val) => {
                    for property_value in val {
                        len += write_variable_bytes(key as usize, |byte|buf.put_u8(byte))?;
                        match property_value {
                            PropertyValue::VariableByteInteger(v) => {
                                len += write_variable_bytes(v, |byte|buf.put_u8(byte))?;
                            }
                            _ => panic!("Not support yet")
                        }
                    }
                }
            }
        }
        assert_eq!(self.property_length, len);
        Ok(self.property_length)
    }

    fn from_buf(buf: &mut BytesMut) -> Result<Mqtt5Property, Error> {
        let property_length = read_variable_bytes(buf)
            .expect("Failed to parse Mqtt5 Property length").0;
        let mut prop_len: usize = 0;
        let mut property = Mqtt5Property::new();
        property.property_length = property_length;
        let mut user_properties = Vec::<(String, String)>::new();
        let mut subscription_identifiers = vec![];
        while property_length > prop_len {
            let variable_bytes = read_variable_bytes(buf)
                .expect("Failed to parse Property Id");
            let property_id = variable_bytes.0;
            prop_len += variable_bytes.1;
            match property_id {
                // Payload Format Indicator -> Will, Publish
                0x01 => {
                    if property.properties.contains_key(&0x01) {
                        return Err(Error::InvalidProtocol("Cannot contains [Payload Format Indicator] more than once".to_string(), 0x18));
                    }
                    property.properties.insert(0x01, PropertyValue::Bit(buf.get_u8() & 0x01 == 1));
                    prop_len += 1;
                }
                // Message Expiry Interval -> Will, Publish
                0x02 => {
                    if property.properties.contains_key(&0x02) {
                        return Err(Error::InvalidProtocol("Cannot contains [Message Expiry Interval] more than once".to_string(), 0x02));
                    }
                    // If present, the Four Byte value is the lifetime of the Will Message in seconds and is sent as the Publication Expiry Interval when the Server publishes the Will Message.
                    // If absent, no Message Expiry Interval is sent when the Server publishes the Will Message.
                    property.properties.insert(0x02, PropertyValue::FourByteInteger(buf.get_u32()));
                    prop_len += 4;
                }
                // Content Type -> Will, Publish
                0x03 => {
                    if property.properties.contains_key(&0x03) {
                        return Err(Error::InvalidProtocol("Cannot contains [Content Type] more than once".to_string(), 0x03));
                    }
                    let content_type = read_string(buf).expect("Failed to parse Content Type");
                    property.properties.insert(0x03, PropertyValue::String(content_type.clone()));
                    prop_len += content_type.len() + 2;
                }
                // Response Topic -> Will, Publish
                0x08 => {
                    if property.properties.contains_key(&0x08) {
                        return Err(Error::InvalidProtocol("Cannot contains [Response Topic] more than once".to_string(), 0x08));
                    }
                    let response_topic = read_string(buf).expect("Failed to parse Response Topic");
                    property.properties.insert(0x08, PropertyValue::String(response_topic.clone()));
                    prop_len += response_topic.len() + 2;
                }
                // Correlation Data -> Will, Publish
                0x09 => {
                    if property.properties.contains_key(&0x09) {
                        return Err(Error::InvalidProtocol("Cannot contains [Correlation Data] more than once".to_string(), 0x09));
                    }
                    let length = buf.get_u16() as usize;
                    prop_len += length;
                    let mut data = buf.split_to(length);
                    property.properties.insert(0x09, PropertyValue::Binary(data.to_bytes()));
                }
                // Subscription Identifier
                0x0B => {
                    let subscription_identifier = read_variable_bytes(buf).expect("Failed to parse Subscription Identifier");
                    if subscription_identifier.0 == 0 {
                        return Err(Error::InvalidProtocol("The value of Subscription Identifier can not be zero".to_string(), 0x0B));
                    }
                    subscription_identifiers.push(PropertyValue::VariableByteInteger(subscription_identifier.0));
                    prop_len += subscription_identifier.1;
                }
                // Session Expiry Interval -> Connect, Connack
                0x11 => {
                    if property.properties.contains_key(&0x11) {
                        return Err(Error::InvalidProtocol("Cannot contains [Session Expire Interval] more than once".to_string(), 0x11));
                    }
                    property.properties.insert(0x11, PropertyValue::FourByteInteger(buf.get_u32()));
                    prop_len += 4;
                }
                // Assigned Client Identifier -> Connack
                0x12 => {
                    if property.properties.contains_key(&0x12) {
                        return Err(Error::InvalidProtocol("Cannot contains [Assigned Client Identifier] more than once".to_string(), 0x12));
                    }
                    let assigned_client_identifier = read_string(buf).expect("Failed to parse Assigned Client Identifier");
                    property.properties.insert(0x12, PropertyValue::String(assigned_client_identifier.clone()));
                    prop_len += assigned_client_identifier.len() + 2;
                }
                // Server Keep Alive -> Connack
                0x13 => {
                    if property.properties.contains_key(&0x13) {
                        return Err(Error::InvalidProtocol("Cannot contains [Server Keep Alive] more than once".to_string(), 0x13));
                    }
                    let server_keep_alive = buf.get_u16();
                    property.properties.insert(0x13, PropertyValue::TwoByteInteger(server_keep_alive));
                    prop_len += 2;
                }
                // Authentication Method -> Connect
                0x15 => {
                    if property.properties.contains_key(&0x15) {
                        return Err(Error::InvalidProtocol("Cannot contains [Authentication Method] more than once".to_string(), 0x15));
                    }
                    let authentication_method = read_string(buf).expect("Failed to parse Authentication Method");
                    property.properties.insert(0x15, PropertyValue::String(authentication_method.clone()));
                    prop_len += authentication_method.len() + 2;
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
                // Response Information -> Connack
                0x1A => {
                    if property.properties.contains_key(&0x1A) {
                        return Err(Error::InvalidProtocol("Cannot contains [Response Information] more than once".to_string(), 0x1A));
                    }
                    let response_information = read_string(buf).expect("Failed to parse Response Information");
                    property.properties.insert(0x1A, PropertyValue::String(response_information.clone()));
                    prop_len += response_information.len() + 2;
                }
                // Server Reference -> Connack
                0x1C => {
                    if property.properties.contains_key(&0x1C) {
                        return Err(Error::InvalidProtocol("Cannot contains [Server Reference] more than once".to_string(), 0x1C));
                    }
                    let server_information = read_string(buf).expect("Failed to parse Server Reference");
                    property.properties.insert(0x1C, PropertyValue::String(server_information.clone()));
                    prop_len += server_information.len() + 2;
                }
                // Reason String -> Connack
                0x1F => {
                    if property.properties.contains_key(&0x1F) {
                        return Err(Error::InvalidProtocol("Cannot contains [Reason String] more than once or for it have the value 0".to_string(), 0x1F));
                    }
                    let reason_string = read_string(buf).expect("Failed to parse Reason Strinng");
                    property.properties.insert(0x1F, PropertyValue::String(reason_string.clone()));
                    prop_len += reason_string.len() + 2;
                }
                // Receive Maximum -> Connect, Connack
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
                // Topic Alias Maximum -> Connect, Connack
                0x22 => {
                    //  It is a Protocol Error to include the Topic Alias Maximum value more than once.
                    if property.properties.contains_key(&0x22) {
                        return Err(Error::InvalidProtocol("Cannot contains [Topic Alias Maximum] more than once".to_string(), 0x22));
                    }
                    property.properties.insert(0x22, PropertyValue::TwoByteInteger(buf.get_u16()));
                    prop_len += 2;
                }
                // Topic Alias -> Publish
                0x23 => {
                    if property.properties.contains_key(&0x23) {
                        return Err(Error::InvalidProtocol("Cannot contains [Topic Alias] more than once".to_string(), 0x23));
                    }
                    property.properties.insert(0x23, PropertyValue::TwoByteInteger(buf.get_u16()));
                    prop_len += 2;
                }
                // Maximum Qos -> Connack
                0x24 => {
                    // It is a Protocol Error to include Maximum QoS more than once, or to have a value other than 0 or 1.
                    if property.properties.contains_key(&0x24) {
                        return Err(Error::InvalidProtocol("Cannot contains [Maximum Qos] more than once".to_string(), 0x24));
                    }
                    let maximum_qos = buf.get_u8();
                    match maximum_qos {
                        0 | 1 => {},
                        _ => return Err(Error::InvalidProtocol("Maximum Qos cannot have a value other than 0 or 1".to_string(), 0x24))
                    }
                    property.properties.insert(0x24, PropertyValue::Byte(maximum_qos));
                    prop_len += 1;
                }
                // Retain Available -> Connack
                0x25 => {
                    if property.properties.contains_key(&0x25) {
                        return Err(Error::InvalidProtocol("Cannot contains [Retain Available] more than once".to_string(), 0x25));
                    }
                    let retain_available = buf.get_u8();
                    match retain_available {
                        0 | 1 => {},
                        _ => return Err(Error::InvalidProtocol("Retain Available cannot have a value other than 0 or 1".to_string(), 0x25))
                    }
                    property.properties.insert(0x25, PropertyValue::Byte(retain_available));
                    prop_len += 1;
                }
                // User Property -> Connect, Will, Publish
                0x26 => {
                    // The User Property is allowed to appear multiple times to represent multiple name, value pairs.
                    // The same name is allowed to appear more than once.
                    let name = read_string(buf).expect("Failed to parse User property");
                    let value = read_string(buf).expect("Failed to parse User property");
                    user_properties.push((name.clone(), value.clone()));
                    prop_len += name.len() + 2;
                    prop_len += value.len() + 2;
                }
                // Maximum Packet Size -> Connect, Connack
                0x27 => {
                    // It is a Protocol Error to include the Maximum Packet Size more than once, or for the value to be set to zero.
                    // If the Maximum Packet Size is not present, no limit on the packet size is imposed beyond the limitations in the
                    // protocol as a result of the remaining length encoding and the protocol header sizes.
                    let maximum_packet_size = buf.get_u32();
                    if property.properties.contains_key(&0x27) || maximum_packet_size == 0 {
                        return Err(Error::InvalidProtocol("Cannot contains [Maximum Packet Size] more than once".to_string(), 0x27));
                    }
                    property.properties.insert(0x27, PropertyValue::FourByteInteger(maximum_packet_size));
                    prop_len += 4;
                }
                // Wildcard Subscription Available -> Connack
                0x28 => {
                    if property.properties.contains_key(&0x28) {
                        return Err(Error::InvalidProtocol("Cannot contains [Wildcard Subscription Available] more than once or for it have the value 0 or 1".to_string(), 0x28));
                    }
                    let wildcard_subscription_available = buf.get_u8();
                    match wildcard_subscription_available {
                        0 | 1 => {},
                        _ => return Err(Error::InvalidProtocol("Wildcard Subscription Available cannot have a value other than 0 or 1".to_string(), 0x28))
                    }
                    property.properties.insert(0x28, PropertyValue::Byte(wildcard_subscription_available));
                    prop_len += 1;
                }
                // Subscription Identifiers Available -> Connack
                0x29 => {
                    if property.properties.contains_key(&0x29) {
                        return Err(Error::InvalidProtocol("Cannot contains [Subscription Identifiers Available] more than once or for it have the value 0 or 1".to_string(), 0x29));
                    }
                    let subscription_identifier_available = buf.get_u8();
                    match subscription_identifier_available {
                        0 | 1 => {},
                        _ => return Err(Error::InvalidProtocol("Subscription Identifiers Available cannot have a value other than 0 or 1".to_string(), 0x29))
                    }
                    property.properties.insert(0x29, PropertyValue::Byte(subscription_identifier_available));
                    prop_len += 1;
                }
                // Shared Subscription Available -> Connack
                0x2A => {
                    if property.properties.contains_key(&0x29) {
                        return Err(Error::InvalidProtocol("Cannot contains [Shared Subscription Available] more than once or for it have the value 0 or 1".to_string(), 0x2A));
                    }
                    let shared_subscription_available = buf.get_u8();
                    match shared_subscription_available {
                        0 | 1 => {},
                        _ => return Err(Error::InvalidProtocol("Shared Subscription Available cannot have a value other than 0 or 1".to_string(), 0x2A))
                    }
                    property.properties.insert(0x2A, PropertyValue::Byte(shared_subscription_available));
                    prop_len += 1;
                }
                _ => {
                    return Err(Error::InvalidPropertyType("It's a invalid Property Type".to_string()));
                }
            }
        }
        if user_properties.len() > 0 {
            property.properties.insert(0x26, PropertyValue::StringPair(user_properties));
        }

        if subscription_identifiers.len() > 0 {
            property.properties.insert(0x0B, PropertyValue::Multiple(subscription_identifiers));
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
    SharedSubscriptionAvailable,
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
    StringPair(Vec<(String, String)>),
    Multiple(Vec<PropertyValue>)
}


#[cfg(test)]
mod test {
    use crate::{Mqtt5Property, PropertyValue, FromToBuf, read_bytes};
    use bytes::Buf;

    #[test]
    fn test_property2buf() {
        use bytes::BytesMut;

        let mut property = Mqtt5Property::new();
        property.properties.insert(0x11, PropertyValue::FourByteInteger(30));
        let mut list = Vec::new();
        list.push(("name".to_string(), "iamazy".to_string()));
        list.push(("age".to_string(), "23".to_string()));
        property.properties.insert(0x26, PropertyValue::StringPair(list));
        property.property_length = 30;
        let mut buf = BytesMut::with_capacity(64);
        property.to_buf(&mut buf);
        println!("{:?}", buf.to_vec());
    }

    #[test]
    fn test_buf2property() {
        use bytes::BytesMut;

        let vec: Vec<u8> = vec![30, 17, 0, 0, 0, 30, 38, 0, 4, 110, 97, 109, 101, 0, 6, 105, 97, 109, 97, 122, 121, 38, 0, 3, 97, 103, 101, 0, 2, 50, 51];
        let mut buf = BytesMut::from(vec.as_slice());
        let property = Mqtt5Property::from_buf(&mut buf).expect("Failed to parse Mqtt5 Property");
        println!("{:?}", property);
    }

    #[test]
    fn test_decode() {
        use bytes::{BytesMut, BufMut};

        let mut buf = BytesMut::with_capacity(64);

        buf.put_slice(&[4u8, 4u8, 'M' as u8, 'Q' as u8, 'T' as u8, 'T' as u8, 5u8]);

        println!("{}", buf.get_u16() as i32);
        println!("{:?}", buf.remaining());

        let vec = read_bytes(&mut buf);
        println!("{:?}", vec);
    }
}