use mqtt_codec::{Error, FromToBuf};
use std::io::{Cursor, Read};
use mqtt_codec::connect::Connect;
use bytes::{BytesMut, BufMut};
use mqtt_codec::fixed_header::FixedHeader;
use mqtt_codec::packet::{PacketType, Packet as Pkt};
use mqtt_codec::connack::ConnAck;
use mqtt_codec::publish::Publish;
use mqtt_codec::puback::PubAck;
use mqtt_codec::pubrec::PubRec;
use mqtt_codec::pubrel::PubRel;
use mqtt_codec::pubcomp::PubComp;
use mqtt_codec::subscribe::Subscribe;
use mqtt_codec::suback::SubAck;
use mqtt_codec::unsubscribe::UnSubscribe;
use mqtt_codec::unsuback::UnSubAck;
use mqtt_codec::pingreq::PingReq;
use mqtt_codec::pingresp::PingResp;
use mqtt_codec::disconnect::Disconnect;
use mqtt_codec::auth::Auth;


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