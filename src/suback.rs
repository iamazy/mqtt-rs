use crate::packet::PacketId;
use crate::{Mqtt5Property, FromToU8};
use crate::frame::FixedHeader;
use crate::subscribe::SubscribeReasonCode;

#[derive(Debug, Clone, PartialEq)]
pub struct SubAck {
    fixed_header: FixedHeader,
    suback_variable_header: SubAckVariableHeader,
    payload: Vec<SubscribeReasonCode>
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubAckVariableHeader {
    packet_id: PacketId,
    suback_property: Mqtt5Property,
}