use crate::Res;
use nom::bytes::complete::take;
use nom::{Err as NomErr, InputTakeAtPosition};
use nom::error::{context, ErrorKind, VerboseError, VerboseErrorKind};
use nom::sequence::pair;
use nom::multi::{fold_many_m_n, length_value, length_data};
use nom::number::complete::be_u16;
use nom::combinator::success;
use std::str;

fn is_variable_bytes_end<T>(i: T) -> Res<T, T>
    where
        T: InputTakeAtPosition<Item = u8>,
{
    i.split_at_position1_complete(
        |item| item & 0x80 == 0,
        ErrorKind::HexDigit,
    )
}

pub fn read_variable_bytes(input: &[u8]) -> Res<&[u8], (usize, usize)> {
    context(
        "read variable bytes",
        pair(fold_many_m_n(0, 3, is_variable_bytes_end, Vec::new(), |mut acc: Vec<_>, item| {
            acc.extend_from_slice(item);
            acc
        }), take(1usize)),
    )(input)
        .map(|(next_input, mut res)| {
            res.0.extend_from_slice(res.1);
            let mut remain_len = 0;
            for i in 0..res.0.len() {
                remain_len += (res.0[i] as usize & 0x7F) << (i * 7);
            }
            (next_input, (remain_len, res.0.len()))
        })
}

/// equal to named!(read_string<&[u8], &[u8]>, length_data!(be_u16));
///
///
pub fn read_string(input: &[u8]) -> Res<&[u8], &str> {
    context(
        "read string",
        length_data(be_u16)
    )(input)
        .and_then(|(next_input, res)| {
            match str::from_utf8(res) {
                Ok(s) => Ok((next_input, s)),
                Err(_) => Err(NomErr::Error(VerboseError {
                    errors: vec![
                        (res, VerboseErrorKind::Context("read string -> str::from_utf8"))
                    ]
                }))
            }
        })
}

#[cfg(test)]
mod test_bytes {
    use crate::bytes::{read_variable_bytes, read_string};
    use bytes::BytesMut;
    use mqtt_codec::write_string;

    #[test]
    fn test_read_variable_bytes() {
        let bytes = &[0];
        let variable_bytes = read_variable_bytes(bytes);
        println!("{:?}", variable_bytes);

    }

    #[test]
    fn test_read_string() {
        // let mut buf = BytesMut::with_capacity(64);
        // write_string("iamazy".to_string(), &mut buf);
        // println!("{:?}", buf.to_vec());
        let bytes = &[0, 6, 105, 97, 109, 97, 122, 121];
        let variable_bytes = read_string(bytes);
        assert_eq!(variable_bytes, Ok((&vec![][..], "iamazy")));
    }
}