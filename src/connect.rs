use crate::{Qos, Error};
use crate::protocol::Protocol;


/// Message that the server should publish when the client disconnects
#[derive(Debug, Clone, PartialEq)]
pub struct LastWill {
    /// Will Topic
    ///
    /// if `Will Flag` is set to 1, `Will Topic` will be the next field in payload.
    /// `Will Topic` must be a string encoded by `UTF-8`
    pub topic: String,
    /// Will Payload
    ///
    /// if `Will Flag` is set to 1, `Will Payload` will be the next field in payload.
    /// `Will Payload` defines the application message that is to be published to the
    /// `Will Topic`
    pub payload: Vec<u8>,
    pub qos: Qos,
    /// if the `retain` flag is set to 1, in a `PUBLISH` packet sent by client to a server,
    /// the server must store the application message and its `Qos`, so that it can be delivered
    /// to future subscribers whose subscriptions match its topic name
    pub retain: bool,
}

/// Connect Return Code
///
/// url: http://docs.oasis-open.org/mqtt/mqtt/v3.1.1/os/mqtt-v3.1.1-os.html#_Toc385349257
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectReturnCode {
    /// Connection accepted
    Accepted,
    /// The server does not support the level of the MQTT protocol
    /// requested by the client
    UnacceptableProtocolVersion,
    /// The client identifier is correct UTF-8 but not allowed by the server
    IdentifierRejected,
    /// The network connection has been made but the MQTT service is unavailable
    ServerUnavailable,
    /// The data in the username or password is malformed
    BadCredentials,
    /// The client is not authorized to connect
    NotAuthorized
}

impl ConnectReturnCode {

    fn to_u8(&self) -> u8 {
        match *self {
            ConnectReturnCode::Accepted => 0,
            ConnectReturnCode::UnacceptableProtocolVersion => 1,
            ConnectReturnCode::IdentifierRejected => 2,
            ConnectReturnCode::ServerUnavailable => 3,
            ConnectReturnCode::BadCredentials => 4,
            ConnectReturnCode::NotAuthorized => 5,
        }
    }

    fn from_u8(byte: u8) -> Result<ConnectReturnCode, Error> {
        match byte {
            0 => Ok(ConnectReturnCode::Accepted),
            1 => Ok(ConnectReturnCode::UnacceptableProtocolVersion),
            2 => Ok(ConnectReturnCode::IdentifierRejected),
            3 => Ok(ConnectReturnCode::ServerUnavailable),
            4 => Ok(ConnectReturnCode::BadCredentials),
            5 => Ok(ConnectReturnCode::NotAuthorized),
            n => Err(Error::InvalidConnectReturnCode(n))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Connect {
    pub protocol: Protocol,
    pub keep_alive: u16,
    pub client_id: String,
    pub clean_session: bool,
    pub last_will: Option<LastWill>,
    pub username: Option<String>,
    pub password: Option<String>,
}

/// Connect acknowledgement
///
/// https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901074
pub struct ConnAck {
    /// Informs the client whether the server is using session state from a previous
    /// connection for this client id. This allows the client and server to have consistent
    /// view of the session state
    pub session_present: bool,
    pub code: ConnectReturnCode,
}



















