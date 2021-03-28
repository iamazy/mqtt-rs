use crate::auth::Auth;
use crate::connack::ConnAck;
use crate::connect::Connect;
use crate::disconnect::Disconnect;
use crate::fixed_header::FixedHeader;
use crate::pingreq::PingReq;
use crate::pingresp::PingResp;
use crate::puback::PubAck;
use crate::pubcomp::PubComp;
use crate::publish::Publish;
use crate::pubrec::PubRec;
use crate::pubrel::PubRel;
use crate::suback::SubAck;
use crate::subscribe::Subscribe;
use crate::unsuback::UnSubAck;
use crate::unsubscribe::UnSubscribe;
use crate::{Error, Frame, FromToU8};
use bytes::{Buf, BufMut, BytesMut};
use std::num::NonZeroU16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PacketId(NonZeroU16);

impl PacketId {
    pub fn new(value: u16) -> Self {
        PacketId(NonZeroU16::new(value).unwrap())
    }

    pub fn get(self) -> u16 {
        self.0.get()
    }

    pub fn to_buf(self, buf: &mut impl BufMut) -> usize {
        buf.put_u16(self.get());
        2
    }

    pub fn length(&self) -> usize {
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
    CONNECT = 1,
    /// 2, Server to client, Connect acknowledgement
    CONNACK = 2,
    /// 3, Two-way, Publish message
    PUBLISH = 3,
    /// 4, Two-way, Publish acknowledgement (QoS 1)
    PUBACK = 4,
    /// 5, Two-way, Publish received (QoS 2 delivery part 1)
    PUBREC = 5,
    /// 6, Two-way, Publish release (QoS 2 delivery part 2)
    PUBREL = 6,
    /// 7, Two-way, Publish complete (Qos 2 delivery part 3)
    PUBCOMP = 7,
    /// 8, Client to server, Subscribe request
    SUBSCRIBE = 8,
    /// 9, Server to client, Subscribe acknowledgement
    SUBACK = 9,
    /// 10, Client to server, Unsubscribe request
    UNSUBSCRIBE = 10,
    /// 11, Server to client, Unsubscribe acknowledgement
    UNSUBACK = 11,
    /// 12, Client to server, Ping request
    PINGREQ = 12,
    /// 13, Server to client, Ping response
    PINGRESP = 13,
    /// 14, Two-way, Disconnect notification
    DISCONNECT = 14,
    /// 15, Two-way, Authentication exchange
    AUTH = 15,
}

impl From<u8> for PacketType {
    fn from(byte: u8) -> Self {
        match byte {
            1 => PacketType::CONNECT,
            2 => PacketType::CONNACK,
            3 => PacketType::PUBLISH,
            4 => PacketType::PUBACK,
            5 => PacketType::PUBREC,
            6 => PacketType::PUBREL,
            7 => PacketType::PUBCOMP,
            8 => PacketType::SUBSCRIBE,
            9 => PacketType::SUBACK,
            10 => PacketType::UNSUBSCRIBE,
            11 => PacketType::UNSUBACK,
            12 => PacketType::PINGREQ,
            13 => PacketType::PINGRESP,
            14 => PacketType::DISCONNECT,
            15 => PacketType::AUTH,
            _ => unimplemented!("no other packet type supported"),
        }
    }
}

impl Default for PacketType {
    fn default() -> Self {
        PacketType::DISCONNECT
    }
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
            n => Err(Error::InvalidPacketType(n)),
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
    Auth(Auth),
}

impl Packet {
    pub fn parse(buf: &mut BytesMut) -> Result<Packet, Error> {
        if buf.remaining() <= 1 {
            return Err(Error::Incomplete);
        }
        let fixed_header = FixedHeader::from_buf(buf).expect("Failed to parse Fixed Header");
        let packet = match fixed_header.packet_type {
            PacketType::CONNECT => {
                let connect = Connect::from_buf_extra(buf, fixed_header)
                    .expect("Failed to parse Connect Packet");
                Packet::Connect(connect)
            }
            PacketType::CONNACK => {
                let connack = ConnAck::from_buf_extra(buf, fixed_header)
                    .expect("Failed to parse ConnAck Packet");
                Packet::ConnAck(connack)
            }
            PacketType::PUBLISH => {
                let publish = Publish::from_buf_extra(buf, fixed_header)
                    .expect("Failed to parse Publish Packet");
                Packet::Publish(publish)
            }
            PacketType::PUBACK => {
                let puback = PubAck::from_buf_extra(buf, fixed_header)
                    .expect("Failed to parse PubAck Packet");
                Packet::PubAck(puback)
            }
            PacketType::PUBREC => {
                let pubrec = PubRec::from_buf_extra(buf, fixed_header)
                    .expect("Failed to parse PubRec Packet");
                Packet::PubRec(pubrec)
            }
            PacketType::PUBREL => {
                let pubrel = PubRel::from_buf_extra(buf, fixed_header)
                    .expect("Failed to parse PubRel Packet");
                Packet::PubRel(pubrel)
            }
            PacketType::PUBCOMP => {
                let pubcomp = PubComp::from_buf_extra(buf, fixed_header)
                    .expect("Failed to parse PubComp Packet");
                Packet::PubComp(pubcomp)
            }
            PacketType::SUBSCRIBE => {
                let subscribe = Subscribe::from_buf_extra(buf, fixed_header)
                    .expect("Failed to parse Subscribe Packet");
                Packet::Subscribe(subscribe)
            }
            PacketType::SUBACK => {
                let suback = SubAck::from_buf_extra(buf, fixed_header)
                    .expect("Failed to parse SubAck Packet");
                Packet::SubAck(suback)
            }
            PacketType::UNSUBSCRIBE => {
                let unsubscribe = UnSubscribe::from_buf_extra(buf, fixed_header)
                    .expect("Failed to parse UnSubscribe Packet");
                Packet::UnSubscribe(unsubscribe)
            }
            PacketType::UNSUBACK => {
                let unsuback = UnSubAck::from_buf_extra(buf, fixed_header)
                    .expect("Failed to parse UnSubAck Packet");
                Packet::UnSubAck(unsuback)
            }
            PacketType::PINGREQ => {
                let pingreq = PingReq::from_buf_extra(buf, fixed_header)
                    .expect("Failed to parse PingReq Packet");
                Packet::PingReq(pingreq)
            }
            PacketType::PINGRESP => {
                let pingresp = PingResp::from_buf_extra(buf, fixed_header)
                    .expect("Failed to parse PingResp Packet");
                Packet::PingResp(pingresp)
            }
            PacketType::DISCONNECT => {
                let disconnect = Disconnect::from_buf_extra(buf, fixed_header)
                    .expect("Failed to parse Disconnect Packet");
                Packet::Disconnect(disconnect)
            }
            PacketType::AUTH => {
                let auth =
                    Auth::from_buf_extra(buf, fixed_header).expect("Failed to parse Auth Packet");
                Packet::Auth(auth)
            }
        };
        Ok(packet)
    }
}

pub trait PacketCodec<R> {
    fn decode_fixed_header(buf: &mut BytesMut) -> FixedHeader {
        FixedHeader::from_buf(buf).expect("Failed to parse Fixed Header")
    }

    fn from_buf_extra(buf: &mut BytesMut, fixed_header: FixedHeader) -> Result<R, Error>;
}
