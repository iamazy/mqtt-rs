use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    MalformedVariableByteInteger,

    MalformedFixedHeader,

    InvalidProtocol(String, u8),

    InvalidQos(u8),

    InvalidReasonCode(u8),

    InvalidLength,

    InvalidString(String),

    InvalidPropertyType(String),

    InvalidPacketType(u8),

    MalformedPacket,

    Incomplete,

    Other(String),
}

impl From<String> for Error {
    fn from(src: String) -> Error {
        Error::Other(src.into())
    }
}

impl From<&str> for Error {
    fn from(src: &str) -> Error {
        src.to_string().into()
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::MalformedVariableByteInteger => "Malformed variable byte integer".fmt(fmt),
            Error::MalformedFixedHeader => "Malformed Fixed Header".fmt(fmt),
            Error::InvalidProtocol(err, _) => err.fmt(fmt),
            Error::InvalidQos(_) => "Invalid Qos".fmt(fmt),
            Error::InvalidReasonCode(_) => "Invalid Reason Code".fmt(fmt),
            Error::InvalidLength => "Invalid length".fmt(fmt),
            Error::InvalidString(err) => err.fmt(fmt),
            Error::InvalidPropertyType(err) => err.fmt(fmt),
            Error::InvalidPacketType(_) => "Invalid packet type".fmt(fmt),
            Error::MalformedPacket => "Malformed packet".fmt(fmt),
            Error::Incomplete => "Incomplete Packet".fmt(fmt),
            Error::Other(str) => str.fmt(fmt),
        }
    }
}

impl std::convert::From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        Error::MalformedPacket
    }
}
