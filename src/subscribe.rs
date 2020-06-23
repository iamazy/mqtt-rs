use crate::frame::FixedHeader;
use crate::packet::PacketId;
use crate::{Mqtt5Property, FromToU8, FromToBuf, Error};
use crate::publish::Qos;
use bytes::{BytesMut, BufMut, Buf};
use crate::decoder::{read_string, write_string};

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
        for (topic_filter, subscription_options) in self.payload {
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
            payload.push((topic_filter, subscription_options));
            remaining = remaining - topic_filter.len() - 1;
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
        option_byte |= (self.retain_handling << 4);
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