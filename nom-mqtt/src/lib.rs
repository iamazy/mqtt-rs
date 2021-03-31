#[macro_use]
extern crate nom;

use nom::error::{ErrorKind, VerboseError, context};
use nom::Err as NomErr;
use mqtt_codec::{FixedHeader, PacketType, Qos, PingResp, PingReq, Mqtt5Property, PropertyValue};
use nom::sequence::{tuple, pair};
use nom::number::complete::{be_u8, be_u32};
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

named!(property_value<&[u8], PropertyValue, VerboseError>,
    switch!(read_variable_bytes,
        (0x01, _) => do_parse!(
            bit: be_u8 >>
            (PropertyValue::Bit(bit & 0x01 == 1))
        ) |
        (0x02, _) => do_parse!(
            bytes: be_u32 >>
            (PropertyValue::FourByteInteger(bytes))
        )
    )
);

#[cfg(test)]
mod tests_mqtt {
    use mqtt_codec::{FixedHeader, PacketType, Qos, Frame};
    use bytes::BytesMut;
    use crate::{fixed_header, property_value};

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

    #[test]
    fn test_property_value() {
        let vec = &[1u8];
        let res = property_value(vec);
        println!("{:?}", res);
    }
}