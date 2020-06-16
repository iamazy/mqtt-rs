

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {

    InvalidProtocol(String, u8),

    InvalidQos(u8),

    InvalidReasonCode(u8),

    InvalidHeader,

    InvalidLength,

    InvalidString(String),

    InvalidPropertyType,

    InvalidVariableByteIntegerFormat,

    InvalidPacketType(u8),

    MalformedPacket

}