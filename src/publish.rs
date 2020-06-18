use crate::{FromToU8, Error};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PubAckReasonCode {
    /// 0[0x00], The message is accepted. Publication of the QoS 1 message proceeds
    Success,
    /// 16[0x10], The message is accepted but there are no subscribers. This is sent only by the Server.
    /// If the Server knows that there are no matching subscribers, it MAY use this Reason Code instead of 0x00 (Success)
    NoMatchingSubscribers,
    /// 128[0x80], The receiver does not accept the publish but either does not want to reveal the reason,
    /// or it does not match one of the other values
    UnspecifiedError,
    /// 131[0x83], The `PUBLISH` is valid but the receiver is not willing to accept it.
    ImplementationSpecificError,
    /// 135[0x87], The `PUBLISH` is not authorized
    NotAuthorized,
    /// 144[0x90], The Topic Name is not malformed, but is not accepted by this Client or Server
    TopicNameInvalid,
    /// 145[0x91], The Packet Identifier is already in use. This might indicate a mismatch in the Session State between
    /// the Client and Server
    PacketIdentifierInUse,
    /// 151[0x97], An implementation or administrative imposed limit has been exceeded
    QuotaExceeded,
    /// 153[0x99], The payload format does not match the specified Payload Format Indicator
    PayloadFormatInvalid
}


impl FromToU8<PubAckReasonCode> for PubAckReasonCode {
    fn to_u8(&self) -> u8 {
        match *self {
            PubAckReasonCode::Success => 0,
            PubAckReasonCode::NoMatchingSubscribers => 16,
            PubAckReasonCode::UnspecifiedError => 128,
            PubAckReasonCode::ImplementationSpecificError => 131,
            PubAckReasonCode::NotAuthorized => 135,
            PubAckReasonCode::TopicNameInvalid => 144,
            PubAckReasonCode::PacketIdentifierInUse => 145,
            PubAckReasonCode::QuotaExceeded => 151,
            PubAckReasonCode::PayloadFormatInvalid => 153,
        }
    }

    fn from_u8(byte: u8) -> Result<PubAckReasonCode, Error> {
        match byte {
            0 => Ok(PubAckReasonCode::Success),
            16 => Ok(PubAckReasonCode::NoMatchingSubscribers),
            128 => Ok(PubAckReasonCode::UnspecifiedError),
            131 => Ok(PubAckReasonCode::ImplementationSpecificError),
            135 => Ok(PubAckReasonCode::NotAuthorized),
            144 => Ok(PubAckReasonCode::TopicNameInvalid),
            145 => Ok(PubAckReasonCode::PacketIdentifierInUse),
            151 => Ok(PubAckReasonCode::QuotaExceeded),
            153 => Ok(PubAckReasonCode::PayloadFormatInvalid),
            n => Err(Error::InvalidReasonCode(n))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PubRecReasonCode {
    /// 0[0x00], The message is accepted. Publication of the QoS 2 message proceeds
    Success,
    /// 16[0x10], The message is accepted but there are no subscribers. This is sent only by the Server.
    /// If the Server knows that there are no matching subscribers, it MAY use this Reason Code instead of 0x00 (Success)
    NoMatchingSubscribers,
    /// 128[0x80], The receiver does not accept the publish but either does not want to reveal the reason,
    /// or it does not match one of the other values
    UnspecifiedError,
    /// 131[0x83], The PUBLISH is valid but the receiver is not willing to accept it
    ImplementationSpecificError,
    /// 135[0x87], The PUBLISH is not authorized
    NotAuthorized,
    /// 144[0x90], The Topic Name is not malformed, but is not accepted by this Client or Server
    TopicNameInvalid,
    /// 145[0x91], The Packet Identifier is already in use. This might indicate a mismatch in the
    /// Session State between the Client and Server.
    PacketIdentifierInUse,
    /// 151[0x97], An implementation or administrative imposed limit has been exceeded
    QuotaExceeded,
    /// 153[0x99], The payload format does not match the one specified in the Payload Format Indicator
    PayloadFormatInvalid,
}

impl FromToU8<PubRecReasonCode> for PubRecReasonCode {
    fn to_u8(&self) -> u8 {
        match *self {
            PubRecReasonCode::Success => 0,
            PubRecReasonCode::NoMatchingSubscribers => 16,
            PubRecReasonCode::UnspecifiedError => 128,
            PubRecReasonCode::ImplementationSpecificError => 131,
            PubRecReasonCode::NotAuthorized => 135,
            PubRecReasonCode::TopicNameInvalid => 144,
            PubRecReasonCode::PacketIdentifierInUse => 145,
            PubRecReasonCode::QuotaExceeded => 151,
            PubRecReasonCode::PayloadFormatInvalid => 153
        }
    }

    fn from_u8(byte: u8) -> Result<PubRecReasonCode, Error> {
        match byte {
            0 => Ok(PubRecReasonCode::Success),
            16 => Ok(PubRecReasonCode::NoMatchingSubscribers),
            128 => Ok(PubRecReasonCode::UnspecifiedError),
            131 => Ok(PubRecReasonCode::ImplementationSpecificError),
            135 => Ok(PubRecReasonCode::NotAuthorized),
            144 => Ok(PubRecReasonCode::TopicNameInvalid),
            145 => Ok(PubRecReasonCode::PacketIdentifierInUse),
            151 => Ok(PubRecReasonCode::QuotaExceeded),
            153 => Ok(PubRecReasonCode::PayloadFormatInvalid),
            n => Err(Error::InvalidReasonCode(n))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Qos {
    /// Qos value: 0
    AtMostOnce,
    /// Qos value: 1
    AtLeastOnce,
    /// Qos value: 2
    ExactlyOnce,
}

impl FromToU8<Qos> for Qos {

     fn to_u8(&self) -> u8 {
        match *self {
            Qos::AtMostOnce => 0,
            Qos::AtLeastOnce => 1,
            Qos::ExactlyOnce => 2
        }
    }

    fn from_u8(byte: u8) -> Result<Qos, Error> {
        match byte {
            0 => Ok(Qos::AtMostOnce),
            1 => Ok(Qos::AtLeastOnce),
            2 => Ok(Qos::ExactlyOnce),
            _ => Err(Error::MalformedPacket),
        }
    }
}