use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;
use bytes::{BytesMut, BufMut};
use mqtt_codec::{Error, Frame};
use std::io;
use mqtt_codec::packet::Packet;

#[derive(Debug)]
pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut
}

impl Connection {

    pub fn new(socket: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(socket),
            buffer: BytesMut::with_capacity(4 * 1024)
        }
    }

    pub(crate) async fn read_packet(&mut self) -> Result<Packet, Error> {
        Packet::parse(&mut self.buffer)
    }

    pub async fn write_packet(&mut self, packet: &Packet) -> Result<(), Error> {
        let mut buf = vec![];
        match packet {
            Packet::Connect(connect) => {
                connect.to_buf(&mut buf);
            }
            Packet::ConnAck(connack) => {
                connack.to_buf(&mut buf);
            }
            Packet::Publish(publish) => {
                publish.to_buf(&mut buf);
            }
            Packet::PubAck(puback) => {
                puback.to_buf(&mut buf);
            }
            Packet::PubRec(pubrec) => {
                pubrec.to_buf(&mut buf);
            }
            Packet::PubRel(pubrel) => {
                pubrel.to_buf(&mut buf);
            }
            Packet::PubComp(pubcomp) => {
                pubcomp.to_buf(&mut buf);
            }
            Packet::Subscribe(subscribe) => {
                subscribe.to_buf(&mut buf);
            }
            Packet::SubAck(suback) => {
                suback.to_buf(&mut buf);
            }
            Packet::UnSubscribe(unsubscribe) => {
                unsubscribe.to_buf(&mut buf);
            }
            Packet::UnSubAck(unsuback) => {
                unsuback.to_buf(&mut buf);
            }
            Packet::PingReq(pingreq) => {
                pingreq.to_buf(&mut buf);
            }
            Packet::PingResp(pingresp) => {
                pingresp.to_buf(&mut buf);
            }
            Packet::Disconnect(disconnect) => {
                disconnect.to_buf(&mut buf);
            }
            Packet::Auth(auth) => {
                auth.to_buf(&mut buf);
            }
            Packet::Error(err) => {
                buf.put_slice(err.as_bytes());
            }
        }
        self.stream.write_all(&buf).await?;
        Ok(())
    }
}