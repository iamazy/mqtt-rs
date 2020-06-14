use crate::Error;
use bytes::{BytesMut, Buf};
use crate::frame::FixedHeader;
use std::collections::LinkedList;

/// Parse Fixed Header
///
/// http://docs.oasis-open.org/mqtt/mqtt/v5.0/csprd02/mqtt-v5.0-csprd02.html#_Toc498345296
fn read_header(buf: &mut BytesMut) -> Result<Option<FixedHeader>, Error> {
    let mut len: usize = 0;
    // variable byte integer up to 4 bytes
    for pos in 0..=3 {
        // buf.get(0) consist of packet type (4 bit), flags (4 bit)
        if let Some(&byte) = buf.get(pos + 1) {
            // The least significant seven bits of each byte encode the data,
            // and the most significant bit is used to indicate whether there are bytes following
            // in the representation. Thus, each byte encodes 128 values (0-127) and a "continuation bit"
            len += (byte as usize & 0x7F) << (pos * 7);
            if (byte & 0x80) == 0 {
                let header = FixedHeader::new(buf.get_u8(), len)?;
                // reset buf start position to (pos + 1)
                buf.advance(pos + 1);
                return Ok(Some(header));
            }
        } else {
            return Ok(None);
        }
    }
    Err(Error::InvalidHeader)
}

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



pub fn read_user_properties(buf: &mut BytesMut) -> Result<LinkedList<(String, String)>, Error> {
    let user_properties = LinkedList::<(String, String)>::new();
}


#[cfg(test)]
mod test {
    use bytes::{Buf, BytesMut};
    use crate::decoder::{read_bytes, read_header};
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