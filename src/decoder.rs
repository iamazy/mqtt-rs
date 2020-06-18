use crate::Error;
use bytes::{BytesMut, Buf, BufMut};

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

/// # Examples
/// ```
/// use bytes::BytesMut;
///
/// let mut buf = BytesMut::with_capacity(2);
/// write_variable_byte_integer(136, &mut buf);
/// assert_eq!(buf.to_vec(), [127,1])
/// ```
pub fn write_variable_byte_integer(mut value: usize, buf: &mut impl BufMut) {
    while value > 0 {
        let mut encoded_byte: u8 = (value % 0x7F) as u8;
        value = value / 0x7F;
        if value > 0 {
            encoded_byte |= 0x7F;
        }
        buf.put_u8(encoded_byte);
    }
}

/// # Examples
/// ```
/// use bytes::{BytesMut, BufMut};
///
/// let byte1 = 0b1000_1000;
/// let byte2 = 0b0000_0001;
/// let mut buf = BytesMut::with_capacity(2);
/// buf.put_u8(byte1);
/// buf.put_u8(byte2);
/// let value = read_variable_byte_integer(&mut buf).unwrap();
/// assert_eq!(value, 0b1000_1000);
/// ```
pub fn read_variable_byte_integer(buf: &mut BytesMut) -> Result<usize, Error> {
    let mut value: usize = 0;
    for pos in 0..=3 {
        if let Some(&byte) = buf.get(pos) {
            value += (byte as usize & 0x7F) << (pos * 7);
            if (byte & 0x80) == 0 {
                buf.advance(pos + 1);
                return Ok(value);
            }
        }
    }
    Err(Error::MalformedVariableByteInteger)
}


#[cfg(test)]
mod test {
    use bytes::{Buf, BytesMut, BufMut};
    use crate::decoder::read_bytes;


    #[test]
    fn test_decode() {
        use bytes::{BytesMut, BufMut};

        let mut buf = BytesMut::with_capacity(64);

        buf.put_slice(&[4u8, 4u8, 'M' as u8, 'Q' as u8, 'T' as u8, 'T' as u8, 5u8]);

        println!("{}", buf.get_u16() as i32);
        println!("{:?}", buf.remaining());

        let vec = read_bytes(&mut buf);
        println!("{:?}", vec);
    }
}