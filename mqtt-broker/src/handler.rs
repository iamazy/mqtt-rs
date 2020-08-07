use crate::channel::Channel;
use mqtt_core::codec::{ConnAck, Connect, Packet, PacketType, PingResp, Protocol};
use mqtt_core::Result;
use mqtt_core::Shutdown;
use std::borrow::Borrow;
use std::sync::Arc;
use tokio::sync::{mpsc, Semaphore};
use tracing::{debug, instrument};

#[derive(Debug)]
pub(crate) struct Handler {
    pub(crate) channel: Channel,
    pub(crate) shutdown: Shutdown,
    pub(crate) _shutdown_complete: mpsc::UnboundedSender<()>,
    pub(crate) limit_connections: Arc<Semaphore>,
}

impl Handler {
    #[instrument(skip(self))]
    pub(crate) async fn run(&mut self) -> Result<()> {
        while !self.shutdown.is_shutdown() {
            let maybe_packet = tokio::select! {
                res = self.channel.read_packet() => res?,
                _ = self.shutdown.recv() => {
                    return Ok(())
                }
            };
            let packet = match maybe_packet {
                Some(packet) => packet,
                None => return Ok(()),
            };
            self.process(&packet).await?;
        }
        Ok(())
    }

    async fn process(&mut self, packet: &Packet) -> Result<()> {
        #[rustfmt::skip]
        debug!("client: {}:{}, received packet {:?}", self.channel.address.ip(), self.channel.address.port(), packet);
        match packet.borrow() {
            Packet::Connect(connect) => {
                self.handle_connect(connect).await?;
            }
            Packet::PingReq(pingreq) => {
                self.channel
                    .write_packet(&Packet::PingResp(PingResp::default()))
                    .await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_connect(&mut self, connect: &Connect) -> Result<()> {
        let fixed_header = &connect.fixed_header;
        assert_eq!(fixed_header.packet_type, PacketType::CONNECT);
        let variable_header = &connect.variable_header;
        assert_eq!(variable_header.protocol, Protocol::MQTT5);
        if variable_header.keep_alive > 0 {
            self.channel.channel_context.keep_alive = variable_header.keep_alive as usize;
        }
        let connect_payload = connect.payload.clone();
        self.channel.channel_context.clean_start = variable_header.connect_flags.clean_start;
        self.channel.channel_context.client_id = connect_payload.client_id;
        self.channel.channel_context.will_topic = connect_payload.will_topic;
        self.channel.channel_context.will_payload = connect_payload.will_payload;
        if variable_header.connect_flags.username_flag {
            self.channel.channel_context.username = connect_payload.username;
        }
        if variable_header.connect_flags.password_flag {
            self.channel.channel_context.password = connect_payload.password;
        }
        self.send_connack().await?;
        Ok(())
    }

    async fn send_connack(&mut self) -> Result<()> {
        self.channel
            .write_packet(&Packet::ConnAck(ConnAck::default()))
            .await?;
        Ok(())
    }
}

impl Drop for Handler {
    fn drop(&mut self) {
        self.limit_connections.add_permits(1);
    }
}
