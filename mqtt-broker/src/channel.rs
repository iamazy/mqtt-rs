use bytes::Bytes;
use mqtt_core::codec::Packet;
use mqtt_core::{Connection, Result};
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct Channel {
    pub id: String,
    pub address: SocketAddr,
    pub connection: Arc<Mutex<Connection>>,
    pub channel_context: ChannelContext,
    is_open: bool,
}

#[derive(Debug, Default)]
pub struct ChannelContext {
    pub client_id: String,
    pub clean_start: bool,
    pub keep_alive: usize,
    pub session_expiry_interval: usize,
    pub receive_maximum: usize,
    pub maximum_packet_size: usize,
    pub topic_alias_maximum: usize,
    pub user_properties: Vec<(String, String)>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub content_type: String,
    pub response_topic: String,
    pub correlation_data: Bytes,
    pub payload_format_indicator: String,
    pub message_expiry_interval: usize,
    pub will_topic: Option<String>,
    pub will_payload: Option<Bytes>,
}

#[allow(dead_code)]
impl Channel {
    pub fn new(id: String, address: SocketAddr, connection: Arc<Mutex<Connection>>) -> Channel {
        Channel {
            id,
            address,
            connection,
            channel_context: ChannelContext::default(),
            is_open: true,
        }
    }

    pub async fn read_packet(&mut self) -> Result<Option<Packet>> {
        self.connection.lock().await.read_packet().await
    }

    pub async fn write_packet(&mut self, packet: &Packet) -> io::Result<()> {
        self.connection.lock().await.write_packet(packet).await
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn close(&mut self) {
        self.is_open = false
    }
}
