use crate::Res;
use nom::bytes::complete::take;
use nom::{Err as NomErr, InputTakeAtPosition, AsChar};
use nom::error::{context, ErrorKind};
use nom::sequence::pair;
use nom::multi::{many_m_n, fold_many_m_n};
use nom::bytes::streaming::{take_till1, take_while, take_while_m_n};
use nom::number::complete::be_u8;


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
        pair(fold_many_m_n(1, 4, is_variable_bytes_end, Vec::new(), |mut acc: Vec<_>, item| {
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
mod tests {
    use nom::IResult;
    use nom::bytes::complete::tag;
    use nom::multi::many_m_n;
    use crate::Res;
    use crate::bytes::read_variable_bytes;

    #[test]
    fn test_many_m_n() {
        let vec = &[
            0b1000_1000,
            0b0000_0001,    // fixed header,
        ];
        let result = read_variable_bytes(vec);
        println!("{:?}",  result);
        println!("{:?}", 0b1000_1000);
    }
}