use std::num::NonZeroU16;
use bytes::{BufMut, BytesMut};
use crate::{Error, FromToU8, Frame};
use crate::fixed_header::FixedHeader;
use crate::connect::Connect;
use crate::connack::ConnAck;
use crate::publish::Publish;
use crate::puback::PubAck;
use crate::pubrec::PubRec;
use crate::pubrel::PubRel;
use crate::pubcomp::PubComp;
use crate::subscribe::Subscribe;
use crate::suback::SubAck;
use crate::unsubscribe::UnSubscribe;
use crate::unsuback::UnSubAck;
use crate::pingreq::PingReq;
use crate::pingresp::PingResp;
use crate::disconnect::Disconnect;
use crate::auth::Auth;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PacketId(NonZeroU16);

impl PacketId {

    pub fn new(value: u16) -> Self {
        PacketId(NonZeroU16::new(value).unwrap())
    }

    pub fn get(self) -> u16 {
        self.0.get()
    }

    pub(crate) fn to_buf(self, buf: &mut impl BufMut) -> usize {
        buf.put_u16(self.get());
        2
    }
}

impl Default for PacketId {
    fn default() -> Self {
        PacketId::new(1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PacketType {
    /// 1, Client to server, Connection request
    CONNECT,
    /// 2, Server to client, Connect acknowledgement
    CONNACK,
    /// 3, Two-way, Publish message
    PUBLISH,
    /// 4, Two-way, Publish acknowledgement (QoS 1)
    PUBACK,
    /// 5, Two-way, Publish received (QoS 2 delivery part 1)
    PUBREC,
    /// 6, Two-way, Publish release (QoS 2 delivery part 2)
    PUBREL,
    /// 7, Two-way, Publish complete (Qos 2 delivery part 3)
    PUBCOMP,
    /// 8, Client to server, Subscribe request
    SUBSCRIBE,
    /// 9, Server to client, Subscribe acknowledgement
    SUBACK,
    /// 10, Client to server, Unsubscribe request
    UNSUBSCRIBE,
    /// 11, Server to client, Unsubscribe acknowledgement
    UNSUBACK,
    /// 12, Client to server, Ping request
    PINGREQ,
    /// 13, Server to client, Ping response
    PINGRESP,
    /// 14, Two-way, Disconnect notification
    DISCONNECT,
    /// 15, Two-way, Authentication exchange
    AUTH
}

impl FromToU8<PacketType> for PacketType {

    fn to_u8(&self) -> u8 {
        match *self {
            PacketType::CONNECT => 1,
            PacketType::CONNACK => 2,
            PacketType::PUBLISH => 3,
            PacketType::PUBACK => 4,
            PacketType::PUBREC => 5,
            PacketType::PUBREL => 6,
            PacketType::PUBCOMP => 7,
            PacketType::SUBSCRIBE => 8,
            PacketType::SUBACK => 9,
            PacketType::UNSUBSCRIBE => 10,
            PacketType::UNSUBACK => 11,
            PacketType::PINGREQ => 12,
            PacketType::PINGRESP => 13,
            PacketType::DISCONNECT => 14,
            PacketType::AUTH => 15,
        }
    }

    fn from_u8(byte: u8) -> Result<PacketType, Error> {
        match byte {
            1 => Ok(PacketType::CONNECT),
            2 => Ok(PacketType::CONNACK),
            3 => Ok(PacketType::PUBLISH),
            4 => Ok(PacketType::PUBACK),
            5 => Ok(PacketType::PUBREC),
            6 => Ok(PacketType::PUBREL),
            7 => Ok(PacketType::PUBCOMP),
            8 => Ok(PacketType::SUBSCRIBE),
            9 => Ok(PacketType::SUBACK),
            10 => Ok(PacketType::UNSUBSCRIBE),
            11 => Ok(PacketType::UNSUBACK),
            12 => Ok(PacketType::PINGREQ),
            13 => Ok(PacketType::PINGRESP),
            14 => Ok(PacketType::DISCONNECT),
            15 => Ok(PacketType::AUTH),
            n => Err(Error::InvalidPacketType(n))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Packet {
    Connect(Connect),
    ConnAck(ConnAck),
    Publish(Publish),
    PubAck(PubAck),
    PubRec(PubRec),
    PubRel(PubRel),
    PubComp(PubComp),
    Subscribe(Subscribe),
    SubAck(SubAck),
    UnSubscribe(UnSubscribe),
    UnSubAck(UnSubAck),
    PingReq(PingReq),
    PingResp(PingResp),
    Disconnect(Disconnect),
    Auth(Auth)
}

impl Packet {

    pub fn parse(buf: &mut BytesMut) -> Result<Packet, Error> {
        let fixed_header = FixedHeader::from_buf(buf)
            .expect("Failed to parse Fixed Header");
        return match fixed_header.packet_type {
            PacketType::CONNECT => {
                let connect = Connect::from_buf_extra(buf, fixed_header)
                    .expect("Failed to parse Connect Packet");
                Ok(Packet::Connect(connect))
            }
            PacketType::CONNACK => {
                let connack = ConnAck::from_buf_extra(buf, fixed_header)
                    .expect("Failed to parse ConnAck Packet");
                Ok(Packet::ConnAck(connack))
            }
            PacketType::PUBLISH => {
                let publish = Publish::from_buf_extra(buf, fixed_header)
                    .expect("Failed to parse Publish Packet");
                Ok(Packet::Publish(publish))
            }
            PacketType::PUBACK => {
                let puback = PubAck::from_buf_extra(buf, fixed_header)
                    .expect("Failed to parse PubAck Packet");
                Ok(Packet::PubAck(puback))
            }
            PacketType::PUBREC => {
                let pubrec = PubRec::from_buf_extra(buf, fixed_header)
                    .expect("Failed to parse PubRec Packet");
                Ok(Packet::PubRec(pubrec))
            }
            PacketType::PUBREL => {
                let pubrel = PubRel::from_buf_extra(buf, fixed_header)
                    .expect("Failed to parse PubRel Packet");
                Ok(Packet::PubRel(pubrel))
            }
            PacketType::PUBCOMP => {
                let pubcomp = PubComp::from_buf_extra(buf, fixed_header)
                    .expect("Failed to parse PubComp Packet");
                Ok(Packet::PubComp(pubcomp))
            }
            PacketType::SUBSCRIBE => {
                let subscribe = Subscribe::from_buf_extra(buf, fixed_header)
                    .expect("Failed to parse Subscribe Packet");
                Ok(Packet::Subscribe(subscribe))
            }
            PacketType::SUBACK => {
                let suback = SubAck::from_buf_extra(buf, fixed_header)
                    .expect("Failed to parse SubAck Packet");
                Ok(Packet::SubAck(suback))
            }
            PacketType::UNSUBSCRIBE => {
                let unsubscribe = UnSubscribe::from_buf_extra(buf, fixed_header)
                    .expect("Failed to parse UnSubscribe Packet");
                Ok(Packet::UnSubscribe(unsubscribe))
            }
            PacketType::UNSUBACK => {
                let unsuback = UnSubAck::from_buf_extra(buf, fixed_header)
                    .expect("Failed to parse UnSubAck Packet");
                Ok(Packet::UnSubAck(unsuback))
            }
            PacketType::PINGREQ => {
                let pingreq = PingReq::from_buf_extra(buf, fixed_header)
                    .expect("Failed to parse PingReq Packet");
                Ok(Packet::PingReq(pingreq))
            }
            PacketType::PINGRESP => {
                let pingresp = PingResp::from_buf_extra(buf, fixed_header)
                    .expect("Failed to parse PingResp Packet");
                Ok(Packet::PingResp(pingresp))
            }
            PacketType::DISCONNECT => {
                let disconnect = Disconnect::from_buf_extra(buf, fixed_header)
                    .expect("Failed to parse Disconnect Packet");
                Ok(Packet::Disconnect(disconnect))
            }
            PacketType::AUTH => {
                let auth = Auth::from_buf_extra(buf, fixed_header)
                    .expect("Failed to parse Auth Packet");
                Ok(Packet::Auth(auth))
            }
        }
    }
}


pub trait PacketCodec<R> {
    fn decode_fixed_header(buf: &mut BytesMut) -> FixedHeader {
        FixedHeader::from_buf(buf).expect("Failed to parse Fixed Header")
    }

    fn from_buf_extra(buf: &mut BytesMut, fixed_header: FixedHeader) -> Result<R, Error>;
}