use crate::frame::FixedHeader;
use crate::packet::PacketId;
use crate::{Mqtt5Property, FromToU8, FromToBuf, Error, write_string, read_string};
use crate::publish::Qos;
use bytes::{BytesMut, BufMut, Buf};

#[derive(Debug, Clone, PartialEq)]
pub struct Subscribe {
    fixed_header: FixedHeader,
    subscribe_variable_header: SubscribeVariableHeader,
    // (topic filter, subscription options)
    payload: Vec<(String, SubscriptionOptions)>
}

impl FromToBuf<Subscribe> for Subscribe {

    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let mut len = self.fixed_header.to_buf(buf)?;
        len += self.subscribe_variable_header.to_buf(buf)?;
        for (topic_filter, subscription_options) in self.payload.clone() {
            len += write_string(topic_filter, buf);
            len += subscription_options.to_buf(buf)?;
        }
        Ok(len)
    }

    fn from_buf(buf: &mut BytesMut) -> Result<Subscribe, Error> {
        let fixed_header = FixedHeader::new(buf, false, Qos::AtLeastOnce, false)
            .expect("Failed to parse Subscribe Fixed Header");
        let subscribe_variable_header = SubscribeVariableHeader::from_buf(buf)
            .expect("Failed to parse Subscribe Variable Header");
        let mut remaining = fixed_header.remaining_length - subscribe_variable_header.subscribe_property.property_length - 2;
        let mut payload = Vec::<(String, SubscriptionOptions)>::new();
        while remaining > 0 {
            let topic_filter = read_string(buf).expect("Failed to parse Topic Filter");
            let subscription_options = SubscriptionOptions::from_buf(buf)
                .expect("Failed to parse Subscription Options");
            payload.push((topic_filter.clone(), subscription_options));
            remaining = remaining - topic_filter.len() - 3;
            if remaining < 0 {
                return Err(Error::MalformedPacket);
            }
        }
        Ok(Subscribe {
            fixed_header,
            subscribe_variable_header,
            payload
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubscribeVariableHeader {
    packet_id: PacketId,
    subscribe_property: Mqtt5Property
}

impl SubscribeVariableHeader {

    fn check_connack_property(subscribe_property: &mut Mqtt5Property) -> Result<(), Error> {
        for key in subscribe_property.properties.keys() {
            let key = *key;
            match key {
                0x0B | 0x26 => {},
                _ => {
                    return Err(Error::InvalidPropertyType("Subscribe Properties contains a invalid property".to_string()))
                }
            }
        }
        Ok(())
    }

}

impl FromToBuf<SubscribeVariableHeader> for SubscribeVariableHeader {
    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let mut len = self.packet_id.to_buf(buf)?;
        len += self.subscribe_property.to_buf(buf)?;
        Ok(len)
    }

    fn from_buf(buf: &mut BytesMut) -> Result<SubscribeVariableHeader, Error> {
        let packet_id = PacketId::new(buf.get_u16());
        let mut subscribe_property = Mqtt5Property::from_buf(buf)
            .expect("Failed to parse Subscribe Properties");
        SubscribeVariableHeader::check_connack_property(&mut subscribe_property);
        Ok(SubscribeVariableHeader {
            packet_id,
            subscribe_property
        })
    }
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
        let mut option_byte:u8= 0b0000_0000;
        option_byte |= self.maximum_qos.to_u8();
        if self.no_local {
            option_byte |= 0b0000_0100;
        }
        if self.retain_as_published {
            option_byte |= 0b0000_1000;
        }
        option_byte |= self.retain_handling << 4;
        buf.put_u8(option_byte);
        Ok(1)
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SubscribeReasonCode {
    /// 0[0x00], The subscription is accepted and the maximum QoS sent will be QoS 0.
    /// This might be a lower QoS than was requested.
    GrantedQos0,
    /// 1[0x01], The subscription is accepted and the maximum QoS sent will be QoS 1.
    /// This might be a lower QoS than was requested.
    GrantedQos1,
    /// 2[0x02], The subscription is accepted and any received QoS will be sent to this subscription.
    GrantedQos2,
    /// 128[0x80], The subscription is not accepted and the Server either does not wish to reveal the
    /// reason or none of the other Reason Codes apply.
    UnspecifiedError,
    /// 131[0x83], The SUBSCRIBE is valid but the Server does not accept it.
    ImplementationSpecificError,
    /// 135[0x87], The Client is not authorized to make this subscription.
    NotAuthorized,
    /// 143[0x8F], The Topic Filter is correctly formed but is not allowed for this Client.
    TopicFilterInvalid,
    /// 145[0X91], The specified Packet Identifier is already in use.
    PacketIdentifierInUse,
    /// 151[0X97], An implementation or administrative imposed limit has been exceeded.
    QuotaExceeded,
    /// 158[0X9E], The Server does not support Shared Subscriptions for this Client.
    SharedSubscriptionNotSupported,
    /// 161[0XA1], The Server does not support Subscription Identifiers; the subscription is not accepted.
    SubscriptionIdentifierNotSupported,
    /// 162[0XA2], The Server does not support Wildcard subscription; the subscription is not accepted.
    WildcardSubscriptionNotSupported
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
    use crate::subscribe::SubscriptionOptions;
    use crate::{FromToBuf, read_string, write_string};

    #[test]
    fn test_subscribe_payload() {
        let mut buf = BytesMut::with_capacity(64);
        buf.put_u8(0);
        buf.put_u8(3);
        buf.put_u8('a' as u8);
        buf.put_u8('/' as u8);
        buf.put_u8('b' as u8);
        buf.put_u8(1);
        buf.put_u8(0);
        buf.put_u8(3);
        buf.put_u8('c' as u8);
        buf.put_u8('/' as u8);
        buf.put_u8('d' as u8);
        buf.put_u8(2);
        println!("{:?}", buf.to_vec());

        let mut payload = Vec::<(String, SubscriptionOptions)>::new();
        let mut remaining = 12;
        while remaining > 0 {
            let topic_filter = read_string(&mut buf).expect("Failed to parse Topic Filter");
            let subscription_options = SubscriptionOptions::from_buf(&mut buf)
                .expect("Failed to parse Subscription Options");
            payload.push((topic_filter.clone(), subscription_options));
            remaining = remaining - topic_filter.len() - 3;
            if remaining < 0 {
                break;
            }
        }
        println!("{:?}", payload);

        let mut buf = BytesMut::with_capacity(64);
        let mut len = 0;
        for (topic_filter, subscription_options) in payload.clone() {
            len += write_string(topic_filter, &mut buf);
            len += subscription_options.to_buf(&mut buf).expect("Failed to encode Subscription Options");
        }
        println!("{:?}", buf.to_vec());

    }
}