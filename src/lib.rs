mod packet;
mod frame;
mod publish;
mod protocol;
mod connect;
mod connack;
mod error;
mod decoder;
pub use error::Error;
use bytes::{BufMut, BytesMut};

trait FromToU8<R> {
    fn to_u8(&self) -> u8;
    fn from_u8(byte: u8) -> Result<R, Error>;
}

trait FromToBuf<R> {
    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error>;
    fn from_buf(buf: &mut BytesMut) -> Result<Option<R>, Error>;
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
            _ => Err(Error::InvalidPropertyType)
        }
    }
}