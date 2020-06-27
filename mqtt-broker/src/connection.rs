use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;
use bytes::BytesMut;

#[derive(Debug)]
pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer:  BytesMut
}

impl Connection {

    pub fn new(socket: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(socket),
            buffer: BytesMut::with_capacity(4 * 1024)
        }
    }
}