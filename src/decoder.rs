use crate::Error;
use bytes::{BytesMut, Buf};
use crate::frame::FixedHeader;

pub fn read_string(buf: &mut BytesMut) -> Result<String, Error> {
    String::from_utf8(read_bytes(buf)?).map_err(|e| Error::InvalidString(e.utf8_error().to_string()))
}

pub fn read_bytes(buf: &mut BytesMut) -> Result<Vec<u8>, Error> {
    let len = buf.get_u16() as usize;
    if len > buf.remaining() {
        Err(Error::InvalidLength)
    } else {
        Ok(buf.split_to(len).to_vec())
    }
}

pub fn read_variable_byte_integer(buf: &mut BytesMut) -> Result<usize, Error> {
    let mut len: usize = 0;
    for pos in 0..=3 {
        if let Some(&byte) = buf.get(pos) {
            len += (byte as usize & 0x7F) << (pos * 7);
            if (byte & 0x80) == 0 {
                buf.advance(pos + 1);
                return Ok(len);
            }
        }
    }
    Err(Error::InvalidVariableByteIntegerFormat)
}


#[cfg(test)]
mod test {
    use bytes::{Buf, BytesMut};
    use crate::decoder::{read_bytes};
    use crate::{Error};
    #[test]
    fn test_decode() {
        use bytes::{BytesMut, BufMut};

        let mut buf = BytesMut::with_capacity(64);

        buf.put_slice(&[4u8,4u8,'M' as u8,'Q' as u8,'T' as u8,'T' as u8,5u8]);

        println!("{}", buf.get_u16() as i32);
        println!("{:?}", buf.remaining());

        let vec = read_bytes(&mut buf);
        println!("{:?}", vec);
    }
}