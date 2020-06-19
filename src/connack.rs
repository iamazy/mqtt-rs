use crate::connect::ConnectReasonCode;
use crate::Mqtt5Property;

#[derive(Debug, Clone, PartialEq)]
pub struct ConnAck {
    pub session_present: bool,
    pub reason_code: ConnectReasonCode,
    pub connack_property: Mqtt5Property
}