mod packet;
mod frame;
mod publish;
mod protocol;
mod connect;
mod connack;
mod error;
pub use error::Error;

trait FromToU8<R> {
    fn to_u8(&self) -> u8;
    fn from_u8(byte: u8) -> Result<R, Error>;
}