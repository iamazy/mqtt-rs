use crate::fixed_header::FixedHeader;
use crate::packet::{PacketId, PacketType, PacketCodec};
use crate::{Mqtt5Property, FromToU8, Frame, Error, write_string, read_string, write_variable_bytes};
use crate::publish::Qos;
use bytes::{BytesMut, BufMut, Buf};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Subscribe {
    fixed_header: FixedHeader,
    variable_header: SubscribeVariableHeader,
    // (topic filter, subscription options)
    payload: Vec<(String, SubscriptionOptions)>,
}

impl PacketCodec<Subscribe> for Subscribe {
    fn from_buf_extra(buf: &mut BytesMut, fixed_header: FixedHeader) -> Result<Subscribe, Error> {
        let variable_header = SubscribeVariableHeader::from_buf(buf)
            .expect("Failed to parse Subscribe Variable Header");
        let mut remaining = fixed_header.remaining_length - 2
            - variable_header.subscribe_property.property_length
            - write_variable_bytes(variable_header.subscribe_property.property_length, |_| {});
        let mut payload = Vec::<(String, SubscriptionOptions)>::new();
        while remaining > 0 {
            let topic_filter = read_string(buf).expect("Failed to parse Topic Filter");
            let subscription_options = SubscriptionOptions::from_buf(buf)
                .expect("Failed to parse Subscription Options");
            payload.push((topic_filter.clone(), subscription_options));
            remaining = remaining - topic_filter.len() - 3;
        }
        Ok(Subscribe {
            fixed_header,
            variable_header,
            payload,
        })
    }
}

impl Frame<Subscribe> for Subscribe {
    fn to_buf(&self, buf: &mut impl BufMut) -> usize {
        let mut len = self.fixed_header.to_buf(buf);
        len += self.variable_header.to_buf(buf);
        for (topic_filter, subscription_options) in self.payload.clone() {
            len += write_string(topic_filter, buf);
            len += subscription_options.to_buf(buf);
        }
        len
    }

    fn from_buf(buf: &mut BytesMut) -> Result<Subscribe, Error> {
        let fixed_header = Subscribe::decode_fixed_header(buf);
        assert_eq!(fixed_header.packet_type, PacketType::SUBSCRIBE);
        assert_eq!(fixed_header.dup, false, "The dup of Subscribe Fixed Header must be set to false");
        assert_eq!(fixed_header.qos, Qos::AtLeastOnce, "The qos of Subscribe Fixed Header must be set to be AtLeastOnce");
        assert_eq!(fixed_header.retain, false, "The retain of Subscribe Fixed Header must be set to false");
        Subscribe::from_buf_extra(buf, fixed_header)
    }

    fn length(&self) -> usize {
        unimplemented!()
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SubscribeVariableHeader {
    packet_id: PacketId,
    subscribe_property: Mqtt5Property,
}

impl SubscribeVariableHeader {
    fn check_subscribe_property(subscribe_property: &mut Mqtt5Property) -> Result<(), Error> {
        for key in subscribe_property.properties.keys() {
            let key = *key;
            match key {
                0x0B | 0x26 => {}
                _ => {
                    return Err(Error::InvalidPropertyType("Subscribe Properties contains a invalid property".to_string()));
                }
            }
        }
        Ok(())
    }
}

impl Frame<SubscribeVariableHeader> for SubscribeVariableHeader {
    fn to_buf(&self, buf: &mut impl BufMut) -> usize {
        let mut len = self.packet_id.to_buf(buf);
        len += self.subscribe_property.to_buf(buf);
        len
    }

    fn from_buf(buf: &mut BytesMut) -> Result<SubscribeVariableHeader, Error> {
        let packet_id = PacketId::new(buf.get_u16());
        let mut subscribe_property = Mqtt5Property::from_buf(buf)
            .expect("Failed to parse Subscribe Properties");
        SubscribeVariableHeader::check_subscribe_property(&mut subscribe_property)?;
        Ok(SubscribeVariableHeader {
            packet_id,
            subscribe_property,
        })
    }

    fn length(&self) -> usize {
        unimplemented!()
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SubscriptionOptions {
    maximum_qos: Qos,
    no_local: bool,
    retain_as_published: bool,
    retain_handling: u8,
}

impl Frame<SubscriptionOptions> for SubscriptionOptions {
    fn to_buf(&self, buf: &mut impl BufMut) -> usize {
        let mut option_byte: u8 = 0b0000_0000;
        option_byte |= self.maximum_qos.to_u8();
        if self.no_local {
            option_byte |= 0b0000_0100;
        }
        if self.retain_as_published {
            option_byte |= 0b0000_1000;
        }
        option_byte |= self.retain_handling << 4;
        buf.put_u8(option_byte);
        1
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
            retain_handling,
        })
    }

    fn length(&self) -> usize {
        unimplemented!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SubscribeReasonCode {
    /// 0[0x00], The subscription is accepted and the maximum QoS sent will be QoS 0.
    /// This might be a lower QoS than was requested.
    GrantedQos0 = 0x00,
    /// 1[0x01], The subscription is accepted and the maximum QoS sent will be QoS 1.
    /// This might be a lower QoS than was requested.
    GrantedQos1 = 0x01,
    /// 2[0x02], The subscription is accepted and any received QoS will be sent to this subscription.
    GrantedQos2 = 0x02,
    /// 128[0x80], The subscription is not accepted and the Server either does not wish to reveal the
    /// reason or none of the other Reason Codes apply.
    UnspecifiedError = 0x80,
    /// 131[0x83], The SUBSCRIBE is valid but the Server does not accept it.
    ImplementationSpecificError = 0x83,
    /// 135[0x87], The Client is not authorized to make this subscription.
    NotAuthorized = 0x87,
    /// 143[0x8F], The Topic Filter is correctly formed but is not allowed for this Client.
    TopicFilterInvalid = 0x8F,
    /// 145[0x91], The specified Packet Identifier is already in use.
    PacketIdentifierInUse = 0x91,
    /// 151[0x97], An implementation or administrative imposed limit has been exceeded.
    QuotaExceeded = 0x97,
    /// 158[0x9E], The Server does not support Shared Subscriptions for this Client.
    SharedSubscriptionNotSupported = 0x9E,
    /// 161[0xA1], The Server does not support Subscription Identifiers; the subscription is not accepted.
    SubscriptionIdentifierNotSupported = 0xA1,
    /// 162[0xA2], The Server does not support Wildcard subscription; the subscription is not accepted.
    WildcardSubscriptionNotSupported = 0xA2,
}

impl Default for SubscribeReasonCode {
    fn default() -> Self {
        SubscribeReasonCode::UnspecifiedError
    }
}

impl FromToU8<SubscribeReasonCode> for SubscribeReasonCode {
    fn to_u8(&self) -> u8 {
        match *self {
            SubscribeReasonCode::GrantedQos0 => 0,
            SubscribeReasonCode::GrantedQos1 => 1,
            SubscribeReasonCode::GrantedQos2 => 2,
            SubscribeReasonCode::UnspecifiedError => 128,
            SubscribeReasonCode::ImplementationSpecificError => 131,
            SubscribeReasonCode::NotAuthorized => 135,
            SubscribeReasonCode::TopicFilterInvalid => 143,
            SubscribeReasonCode::PacketIdentifierInUse => 145,
            SubscribeReasonCode::QuotaExceeded => 151,
            SubscribeReasonCode::SharedSubscriptionNotSupported => 158,
            SubscribeReasonCode::SubscriptionIdentifierNotSupported => 161,
            SubscribeReasonCode::WildcardSubscriptionNotSupported => 162,
        }
    }

    fn from_u8(byte: u8) -> Result<SubscribeReasonCode, Error> {
        match byte {
            0 => Ok(SubscribeReasonCode::GrantedQos0),
            1 => Ok(SubscribeReasonCode::GrantedQos1),
            2 => Ok(SubscribeReasonCode::GrantedQos2),
            128 => Ok(SubscribeReasonCode::UnspecifiedError),
            131 => Ok(SubscribeReasonCode::ImplementationSpecificError),
            135 => Ok(SubscribeReasonCode::NotAuthorized),
            143 => Ok(SubscribeReasonCode::TopicFilterInvalid),
            145 => Ok(SubscribeReasonCode::PacketIdentifierInUse),
            151 => Ok(SubscribeReasonCode::QuotaExceeded),
            158 => Ok(SubscribeReasonCode::SharedSubscriptionNotSupported),
            161 => Ok(SubscribeReasonCode::SubscriptionIdentifierNotSupported),
            162 => Ok(SubscribeReasonCode::WildcardSubscriptionNotSupported),
            n => Err(Error::InvalidReasonCode(n))
        }
    }
}

#[cfg(test)]
mod test {
    use bytes::{BytesMut, BufMut};
    use crate::{Frame};
    use crate::subscribe::Subscribe;

    #[test]
    fn test_subscribe() {
        let subscribe_bytes = &[
            0b1000_0010u8, 17,  // fixed header
            0x00, 0x10, // packet identifier
            2, // properties length
            0x0B, // property id
            0x02, // subscription identifier
            0x00, 0x03, 'a' as u8, '/' as u8, 'b' as u8, // topic filter
            0x01, // subscription options
            0x00, 0x03, 'c' as u8, '/' as u8, 'd' as u8, // topic filter
            0x02, // subscription options
        ];
        let mut buf = BytesMut::with_capacity(64);
        buf.put_slice(subscribe_bytes);
        let subscribe = Subscribe::from_buf(&mut buf)
            .expect("Failed to parse Subscribe Packet");

        let mut buf = BytesMut::with_capacity(64);
        subscribe.to_buf(&mut buf);
        assert_eq!(subscribe, Subscribe::from_buf(&mut buf).unwrap());
    }
}