use crate::connect::ConnectReasonCode;
use std::collections::LinkedList;

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
    pub connack_property: ConnAckProperty
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConnAckProperty {
    property_length: i32,

    session_expiry_interval: Option<usize>,

    receive_maximum: usize,

    maximum_qos: u8,

    retain_available: bool,

    maximum_packet_size: Option<usize>,

    assigned_client_identifier: Option<String>,

    topic_alias_maximum: usize,

    reason_string: Option<String>,

    user_properties: LinkedList<(String, String)>,

    wildcard_subscription_available: bool,

    subscription_identifier_available: bool,

    shared_subscription_available: bool,

    server_keep_alive: u16,

    response_information: Option<String>,

    server_reference: String,

    authentication_method: Option<String>,

    authentication_data: Option<Vec<u8>>
}