use crate::frame::FixedHeader;
use crate::packet::PacketId;
use crate::{Mqtt5Property, FromToU8, FromToBuf, Error};
use crate::publish::Qos;
use bytes::{BytesMut, BufMut, Buf};

#[derive(Debug, Clone, PartialEq)]
pub struct Subscribe {
    fixed_header: FixedHeader,
    subscribe_variable_header: SubscribeVariableHeader,
    // (topic filter, subscription options)
    payload: Vec<(String, SubscriptionOptions)>
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubscribeVariableHeader {
    packet_id: PacketId,
    subscribe_property: Mqtt5Property
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubscriptionOptions {
    maximum_qos: Qos,
    no_local: bool,
    retain_as_published: bool,
    retain_handling: u8,
}

impl FromToBuf<SubscriptionOptions> for SubscriptionOptions {
    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        unimplemented!()
    }

    fn from_buf(buf: &mut BytesMut) -> Result<SubscriptionOptions, Error> {
        let subscription_options = buf.get_u8();
        let maximum_qos = Qos::from_u8(subscription_options & 0b0000_0011)
            .expect("Failed to parse Maximum Qos");
        let no_local = (subscription_options >> 2) & 0x01 == 1;
        let retain_as_published = (subscription_options >> 3) & 0x01 == 1;
        let retain_handling = (subscription_options >> 4) & 0x03;
        Ok(SubscriptionOptions {
            maximum_qos,
            no_local,
            retain_as_published,
            retain_handling
        })
    }
}