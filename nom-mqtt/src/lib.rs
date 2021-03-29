#[macro_use]
extern crate nom;

use nom::error::{ErrorKind, VerboseError, context};
use nom::Err as NomErr;
use mqtt_codec::{FixedHeader, PacketType, Qos, PingResp, PingReq, Mqtt5Property};
use nom::sequence::{tuple, pair};
use nom::number::complete::be_u8;
use crate::bytes::read_variable_bytes;

type IResult<I, O, E = (I, ErrorKind)> = Result<(I, O), NomErr<E>>;
type Res<T, U> = IResult<T, U, VerboseError<T>>;

#[cfg(test)]
mod tests;
mod bytes;

pub fn fixed_header(input: &[u8]) -> Res<&[u8], FixedHeader> {
    context(
        "fixed header",
        pair(be_u8, read_variable_bytes),
    )(input)
        .map(|(next_input, res)| {
            let (fixed_header_byte, (remaining_length, _)) = res;
            (next_input, FixedHeader {
                packet_type: PacketType::from(fixed_header_byte >> 4),
                dup: (fixed_header_byte >> 3) & 0x01 == 1,
                qos: Qos::from((fixed_header_byte >> 1) & 0x03),
                retain: fixed_header_byte & 0x01 == 1,
                remaining_length,
            })
        })
}

pub fn ping_resp(input: &[u8]) -> Res<&[u8], PingResp> {
    context(
        "pingresp",
        fixed_header,
    )(input)
        .map(|(next_input, fixed_header)| {
            (next_input, PingResp { fixed_header })
        })
}

pub fn ping_req(input: &[u8]) -> Res<&[u8], PingReq> {
    context(
        "pingresp",
        fixed_header,
    )(input)
        .map(|(next_input, fixed_header)| {
            (next_input, PingReq { fixed_header })
        })
}


#[cfg(test)]
mod tests_mqtt {
    use mqtt_codec::{FixedHeader, PacketType, Qos, Frame};
    use bytes::BytesMut;
    use crate::fixed_header;

    #[test]
    fn test_fixed_header() {
        let fixed_header0 = FixedHeader {
            packet_type: PacketType::PINGRESP,
            dup: false,
            qos: Qos::AtMostOnce,
            retain: false,
            remaining_length: 0,
        };
        let mut buf = BytesMut::with_capacity(64);
        fixed_header0.to_buf(&mut buf);
        let vec = buf.to_vec();
        let fixed_parsed = fixed_header(&vec[..]);
        match fixed_parsed {
            Ok((next_input, res)) => {
                println!("{:?}", res);
            }
            Err(e) => {
                eprintln!("{}", e.to_string());
            }
        }
    }
}