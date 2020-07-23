use crate::fixed_header::FixedHeader;
use crate::{Mqtt5Property, FromToU8, Error, Frame};
use bytes::{BytesMut, BufMut, Buf};
use crate::publish::Qos;
use crate::packet::{PacketType, PacketCodec};

#[derive(Debug, Clone, PartialEq)]
pub struct Disconnect {
    fixed_header: FixedHeader,
    variable_header: DisconnectVariableHeader
}

impl Default for Disconnect {
    fn default() -> Self {
        let variable_header = DisconnectVariableHeader::default();
        let fixed_header = FixedHeader {
            packet_type: PacketType::DISCONNECT,
            dup: false,
            qos: Qos::AtMostOnce,
            retain: false,
            remaining_length: variable_header.length()
        };
        Disconnect {
            fixed_header,
            variable_header,
        }
    }
}

impl PacketCodec<Disconnect> for Disconnect {
    fn from_buf_extra(buf: &mut BytesMut, fixed_header: FixedHeader) -> Result<Disconnect, Error> {
        let variable_header = DisconnectVariableHeader::from_buf(buf)
            .expect("Failed to parse Disconnect Variable Header");
        Ok(Disconnect {
            fixed_header,
            variable_header
        })
    }
}

impl Frame<Disconnect> for Disconnect {
    fn to_buf(&self, buf: &mut impl BufMut) -> usize {
        let mut len = self.fixed_header.to_buf(buf);
        len += self.variable_header.to_buf(buf);
        len
    }

    fn from_buf(buf: &mut BytesMut) -> Result<Disconnect, Error> {
        let fixed_header = FixedHeader::from_buf(buf)
            .expect("Failed to parse Disconnect Fixed Header");
        assert_eq!(fixed_header.packet_type, PacketType::DISCONNECT);
        assert_eq!(fixed_header.dup, false, "The dup of Disconnect Fixed Header must be set to false");
        assert_eq!(fixed_header.qos, Qos::AtMostOnce, "The qos of Disconnect Fixed Header must be set to be AtMostOnce");
        assert_eq!(fixed_header.retain, false, "The retain of Disconnect Fixed Header must be set to false");
        Disconnect::from_buf_extra(buf, fixed_header)
    }

    fn length(&self) -> usize {
        self.fixed_header.length() + self.fixed_header.remaining_length
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DisconnectVariableHeader {
    reason_code: DisconnectReasonCode,
    disconnect_property: Mqtt5Property
}

impl DisconnectVariableHeader {

    fn check_disconnect_property(disconnect_property: &mut Mqtt5Property) -> Result<(), Error> {

        for key in disconnect_property.properties.keys() {
            let key = *key;
            match key {
                0x11 | 0x1C | 0x1F | 0x26 => {},
                _ => return Err(Error::InvalidPropertyType("Disconnect Properties contains a invalid property".to_string()))
            }
        }
        Ok(())
    }
}

impl Frame<DisconnectVariableHeader> for DisconnectVariableHeader {
    fn to_buf(&self, buf: &mut impl BufMut) -> usize {
        buf.put_u8(self.reason_code.to_u8());
        let mut len = 1;
        len += self.disconnect_property.to_buf(buf);
        len
    }

    fn from_buf(buf: &mut BytesMut) -> Result<DisconnectVariableHeader, Error> {
        let reason_code = DisconnectReasonCode::from_u8(buf.get_u8())
            .expect("Failed to parse Disconnect Reason Code");
        let mut disconnect_property = Mqtt5Property::from_buf(buf)
            .expect("Failed to parse Disconnect Properties");
        DisconnectVariableHeader::check_disconnect_property(&mut disconnect_property)?;
        Ok(DisconnectVariableHeader {
            reason_code,
            disconnect_property
        })
    }

    fn length(&self) -> usize {
        1 + self.disconnect_property.length()
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisconnectReasonCode {
    /// 0[0x00], Close the connection normally. Do not send the Will Message.
    NormalDisconnection = 0x00,
    /// 4[0x04], The Client wishes to disconnect but requires that the Server also publishes its Will Message.
    DisconnectWithWillMessage = 0x04,
    /// 128[0x80], The Connection is closed but the sender either does not wish to reveal the reason, or none of the other Reason Codes apply.
    UnspecifiedError = 0x80,
    /// 129[0x81], The received packet does not conform to this specification.
    MalformedPacket = 0x81,
    /// 130[0x82], An unexpected or out of order packet was received.
    ProtocolError = 0x82,
    /// 131[0x83], The packet received is valid but cannot be processed by this implementation.
    ImplementationSpecificError = 0x83,
    /// 135[0x87], The request is not authorized.
    NotAuthorized = 0x87,
    /// 137[0x89], The Server is busy and cannot continue processing requests from this Client.
    ServerBusy = 0x89,
    /// 139[0x8B], The Server is shutting down.
    ServerShuttingDown = 0x8B,
    /// 141[0x8D], The Connection is closed because no packet has been received for 1.5 times the Keepalive time.
    KeepAliveTimeout = 0x8D,
    /// 142[0x8E], Another Connection using the same ClientID has connected causing this Connection to be closed.
    SessionTakenOver = 0x8E,
    /// 143[0x8F], The Topic Filter is correctly formed, but is not accepted by this Sever.
    TopicFilterInvalid = 0x8F,
    /// 144[0x90], The Topic Name is correctly formed, but is not accepted by this Client or Server.
    TopicNameInvalid = 0x90,
    /// 147[0x93], The Client or Server has received more than Receive Maximum publication for which it has not sent PUBACK or PUBCOMP.
    ReceiveMaximumExceeded = 0x93,
    /// 148[0x94], The Client or Server has received a PUBLISH packet containing a Topic Alias which is greater than the Maximum Topic Alias it sent in the CONNECT or CONNACK packet.
    TopicAliasInvalid = 0x94,
    /// 149[0x95], The packet size is greater than Maximum Packet Size for this Client or Server.
    PacketTooLarge = 0x95,
    /// 150[0x96], The received data rate is too high.
    MessageRateTooHigh = 0x96,
    /// 151[0x97], An implementation or administrative imposed limit has been exceeded.
    QuotaExceeded = 0x97,
    /// 152[0x98], The Connection is closed due to an administrative action.
    AdministrativeAction = 0x98,
    /// 153[0x99], The payload format does not match the one specified by the Payload Format Indicator.
    PayloadFormatInvalid = 0x99,
    /// 154[0x9A], The Server has does not support retained messages.
    RetainNotSupported = 0x9A,
    /// 155[0x9B], The Client specified a QoS greater than the QoS specified in a Maximum QoS in the CONNACK.
    QosNotSupported = 0x9B,
    /// 156[0x9C], The Client should temporarily change its Server.
    UseAnotherServer = 0x9C,
    /// 157[0x9D], The Server is moved and the Client should permanently change its server location.
    ServerMoved = 0x9D,
    /// 158[0x9E], The Server does not support Shared Subscriptions.
    SharedSubscriptionNotSupported = 0x9E,
    /// 159[0x9F], This connection is closed because the connection rate is too high.
    ConnectionRateExceeded = 0x9F,
    /// 160[0xA0], The maximum connection time authorized for this connection has been exceeded.
    MaximumConnectTime = 0xA0,
    /// 161[0xA1], The Server does not support Subscription Identifiers; the subscription is not accepted.
    SubscriptionIdentifiersNotSupported = 0xA1,
    /// 162[0xA2], The Server does not support Wildcard subscription; the subscription is not accepted.
    WildcardSubscriptionsNotSupported = 0xA2
}

impl Default for DisconnectReasonCode {
    fn default() -> Self {
        DisconnectReasonCode::UnspecifiedError
    }
}


impl FromToU8<DisconnectReasonCode> for DisconnectReasonCode {
    fn to_u8(&self) -> u8 {
        match *self {
            DisconnectReasonCode::NormalDisconnection => 0,
            DisconnectReasonCode::DisconnectWithWillMessage => 4,
            DisconnectReasonCode::UnspecifiedError => 128,
            DisconnectReasonCode::MalformedPacket => 129,
            DisconnectReasonCode::ProtocolError => 130,
            DisconnectReasonCode::ImplementationSpecificError => 131,
            DisconnectReasonCode::NotAuthorized => 135,
            DisconnectReasonCode::ServerBusy => 137,
            DisconnectReasonCode::ServerShuttingDown => 139,
            DisconnectReasonCode::KeepAliveTimeout => 141,
            DisconnectReasonCode::SessionTakenOver => 142,
            DisconnectReasonCode::TopicFilterInvalid => 143,
            DisconnectReasonCode::TopicNameInvalid => 144,
            DisconnectReasonCode::ReceiveMaximumExceeded => 147,
            DisconnectReasonCode::TopicAliasInvalid => 148,
            DisconnectReasonCode::PacketTooLarge => 149,
            DisconnectReasonCode::MessageRateTooHigh => 150,
            DisconnectReasonCode::QuotaExceeded => 151,
            DisconnectReasonCode::AdministrativeAction => 152,
            DisconnectReasonCode::PayloadFormatInvalid => 153,
            DisconnectReasonCode::RetainNotSupported => 154,
            DisconnectReasonCode::QosNotSupported => 155,
            DisconnectReasonCode::UseAnotherServer => 156,
            DisconnectReasonCode::ServerMoved => 157,
            DisconnectReasonCode::SharedSubscriptionNotSupported => 158,
            DisconnectReasonCode::ConnectionRateExceeded => 159,
            DisconnectReasonCode::MaximumConnectTime => 160,
            DisconnectReasonCode::SubscriptionIdentifiersNotSupported => 161,
            DisconnectReasonCode::WildcardSubscriptionsNotSupported => 162
        }
    }

    fn from_u8(byte: u8) -> Result<DisconnectReasonCode, Error> {
        match  byte {
            0 => Ok(DisconnectReasonCode::NormalDisconnection),
            4 => Ok(DisconnectReasonCode::DisconnectWithWillMessage),
            128 => Ok(DisconnectReasonCode::UnspecifiedError),
            129 => Ok(DisconnectReasonCode::MalformedPacket),
            130 => Ok(DisconnectReasonCode::ProtocolError),
            131 => Ok(DisconnectReasonCode::ImplementationSpecificError),
            135 => Ok(DisconnectReasonCode::NotAuthorized),
            137 => Ok(DisconnectReasonCode::ServerBusy),
            139 => Ok(DisconnectReasonCode::ServerShuttingDown),
            141 => Ok(DisconnectReasonCode::KeepAliveTimeout),
            142 => Ok(DisconnectReasonCode::SessionTakenOver),
            143 => Ok(DisconnectReasonCode::TopicFilterInvalid),
            144 => Ok(DisconnectReasonCode::TopicNameInvalid),
            147 => Ok(DisconnectReasonCode::ReceiveMaximumExceeded),
            148 => Ok(DisconnectReasonCode::TopicAliasInvalid),
            149 => Ok(DisconnectReasonCode::PacketTooLarge),
            150 => Ok(DisconnectReasonCode::MessageRateTooHigh),
            151 => Ok(DisconnectReasonCode::QuotaExceeded),
            152 => Ok(DisconnectReasonCode::AdministrativeAction),
            153 => Ok(DisconnectReasonCode::PayloadFormatInvalid),
            154 => Ok(DisconnectReasonCode::RetainNotSupported),
            155 => Ok(DisconnectReasonCode::QosNotSupported),
            156 => Ok(DisconnectReasonCode::UseAnotherServer),
            157 => Ok(DisconnectReasonCode::ServerMoved),
            158 => Ok(DisconnectReasonCode::SharedSubscriptionNotSupported),
            159 => Ok(DisconnectReasonCode::ConnectionRateExceeded),
            160 => Ok(DisconnectReasonCode::MaximumConnectTime),
            161 => Ok(DisconnectReasonCode::SubscriptionIdentifiersNotSupported),
            162 => Ok(DisconnectReasonCode::WildcardSubscriptionsNotSupported),
            n => Err(Error::InvalidReasonCode(n))
        }
    }
}

#[cfg(test)]
mod test {
    use bytes::{BytesMut, BufMut};
    use crate::disconnect::Disconnect;
    use crate::Frame;

    #[test]
    fn test_disconnect() {
        let disconnect_bytes = &[
            0b1110_0000u8, 7,  // fixed header
            0x00, // disconnect reason code
            5, // properties length
            0x1F, // property id
            0x00, 0x02, 'I' as u8, 'a' as u8, // reason string
        ];
        let mut buf = BytesMut::with_capacity(64);
        buf.put_slice(disconnect_bytes);
        let disconnect = Disconnect::from_buf(&mut buf)
            .expect("Failed to parse Disconnect Packet");

        let mut buf = BytesMut::with_capacity(64);
        disconnect.to_buf(&mut buf);
        assert_eq!(disconnect, Disconnect::from_buf(&mut buf).unwrap());
    }
}