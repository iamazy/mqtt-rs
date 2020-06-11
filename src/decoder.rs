use crate::Error;
use bytes::{BytesMut, Buf};
use crate::frame::FixedHeader;

// fn read_header(buf: &mut BytesMut) -> Result<Option<FixedHeader>, Error> {
//     let mut len: usize = 0;
//     for pos in 0..=3 {
//
//     }
// }

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


#[cfg(test)]
mod test {

    #[test]
    fn test_range() {
        for i in 0..3 {
            println!("{}", i);
        }
    }
}