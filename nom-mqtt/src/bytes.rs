use crate::Res;
use nom::bytes::complete::take;
use nom::{Err as NomErr, InputTakeAtPosition};
use nom::error::{context, ErrorKind};
use nom::sequence::pair;
use nom::multi::fold_many_m_n;


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

#[cfg(test)]
mod test_bytes {
    use crate::bytes::read_variable_bytes;

    #[test]
    fn test_read_variable_bytes() {
        let bytes = &[0];
        let variable_bytes = read_variable_bytes(bytes);
        println!("{:?}", variable_bytes);

    }
}