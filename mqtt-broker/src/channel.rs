use std::net::SocketAddr;

pub struct Channel {
    id: String,
    address: SocketAddr,
    channel_context: ChannelContext,
}

pub struct ChannelContext {

}