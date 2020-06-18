

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {

    // Bad Data Type
    InvalidDataFormat(String),

    MalformedVariableByteInteger,

    InvalidProtocol(String, u8),

    InvalidQos(u8),

    InvalidReasonCode(u8),

    InvalidHeader,

    InvalidLength,

    InvalidString(String),

    InvalidPropertyType(String),

    InvalidPacketType(u8),

    MalformedPacket

}