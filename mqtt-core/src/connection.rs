use tokio::io::{BufWriter, AsyncWriteExt, AsyncReadExt};
use tokio::net::TcpStream;
use bytes::BytesMut;
use mqtt_codec::{Error, Frame, Packet};
use std::io;

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

    pub async fn read_packet(&mut self) -> Result<Option<Packet>, Error> {
        loop {
            match Packet::parse(&mut self.buffer) {
                Ok(packet) => return Ok(Some(packet)),
                Err(Error::Incomplete) => {},
                Err(e) => return Err(e.into())
            }

            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                return if self.buffer.is_empty() {
                    Ok(None)
                } else {
                    Err("connection reset by peer".into())
                }
            }
        }
    }

    pub async fn write_packet(&mut self, packet: &Packet) -> io::Result<()> {
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
        }
        self.stream.write_all(&buf).await?;
        self.stream.flush().await
    }
}