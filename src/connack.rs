use crate::connect::ConnectReasonCode;
use std::collections::LinkedList;
use crate::Mqtt5Property;

/// Connect acknowledgement
///
/// https://docs.oasis-open.org/mqtt/mqtt/v5.0/os/mqtt-v5.0-os.html#_Toc3901074
#[derive(Debug, Clone, PartialEq)]
pub struct ConnAck {
    /// Informs the client whether the server is using session state from a previous
    /// connection for this client id. This allows the client and server to have consistent
    /// view of the session state
    pub session_present: bool,
    pub reason_code: ConnectReasonCode,
    pub connack_property: Mqtt5Property
}