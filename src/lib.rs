mod packet;
mod frame;
mod publish;
mod protocol;
mod connect;
mod connack;
mod error;
mod decoder;
pub use error::Error;
use bytes::{BufMut, BytesMut};

trait FromToU8<R> {
    fn to_u8(&self) -> u8;
    fn from_u8(byte: u8) -> Result<R, Error>;
}

trait FromToBuf<R> {
    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error>;
    fn from_buf(buf: &mut BytesMut) -> Result<R, Error>;
}