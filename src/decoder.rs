use crate::Error;
use bytes::{BytesMut, Buf, BufMut, Bytes};

pub fn write_string(string: String, buf: &mut impl BufMut) -> usize {
    write_bytes(Bytes::from(string), buf)
}

pub fn read_string(buf: &mut BytesMut) -> Result<String, Error> {
    String::from_utf8(read_bytes(buf)?.to_vec()).map_err(|e| Error::InvalidString(e.utf8_error().to_string()))
}

pub fn write_bytes(bytes: Bytes, buf: &mut impl BufMut) -> usize {
    let len = bytes.len();
    assert!(len <= 65535, "Bytes length must less than or equal 65535");
    buf.put_u16(len as u16);
    buf.put_slice(bytes.bytes());
    len + 2
}

pub fn read_bytes(buf: &mut BytesMut) -> Result<Bytes, Error> {
    let len = buf.get_u16() as usize;
    if len > buf.remaining() {
        Err(Error::InvalidLength)
    } else {
        Ok(buf.split_to(len).to_bytes())
    }
}

/// # Examples
/// ```
/// use bytes::BytesMut;
///
/// let mut buf = BytesMut::with_capacity(2);
/// let len = write_variable_bytes(136, &mut buf);
/// assert_eq!(len, 2);
/// assert_eq!(buf.to_vec(), [127,1])
/// ```
pub fn write_variable_bytes2(mut value: usize, buf: &mut impl BufMut) -> Result<usize, Error>{
    let mut len = 0;
    while value > 0 {
        let mut encoded_byte: u8 = (value % 0x7F) as u8;
        value = value / 0x7F;
        if value > 0 {
            encoded_byte |= 0x7F;
        }
        buf.put_u8(encoded_byte);
        len += 1;
    }
    Ok(len)
}

pub fn write_variable_bytes<T>(mut value: usize, mut callback: T) -> Result<usize, Error>
    where T: FnMut(u8)
{
    let mut len = 0;
    while value > 0 {
        let mut encoded_byte: u8 = (value % 0x7F) as u8;
        value = value / 0x7F;
        if value > 0 {
            encoded_byte |= 0x7F;
        }
        callback(encoded_byte);
        len += 1;
    }
    Ok(len)
}

/// First usize is function returned value
/// Second usize is the number of consumed bytes
///
/// # Examples
/// ```
/// use bytes::{BytesMut, BufMut};
///
/// let byte1 = 0b1000_1000;
/// let byte2 = 0b0000_0001;
/// let mut buf = BytesMut::with_capacity(2);
/// buf.put_u8(byte1);
/// buf.put_u8(byte2);
/// let value = read_variable_bytes(&mut buf).unwrap();
/// assert_eq!(value, 0b1000_1000);
/// ```
pub fn read_variable_bytes(buf: &mut BytesMut) -> Result<(usize, usize), Error> {
    let mut value: usize = 0;
    for pos in 0..=3 {
        if let Some(&byte) = buf.get(pos) {
            value += (byte as usize & 0x7F) << (pos * 7);
            if (byte & 0x80) == 0 {
                buf.advance(pos + 1);
                return Ok((value, pos + 1));
            }
        }
    }
    Err(Error::MalformedVariableByteInteger)
}


#[cfg(test)]
mod test {
    use bytes::{Buf};
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