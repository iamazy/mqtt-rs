#[allow(unused)]
#[macro_use]
extern crate nom;

use crate::bytes::{read_bytes, read_string, read_variable_bytes};
use crate::packet::{
    FixedHeader, Mqtt5Property, PacketType, PingReq, PingResp, PropertyValue, Qos,
};
use nom::error::{context, ErrorKind, VerboseError};
use nom::number::complete::{be_u16, be_u32, be_u8};
use nom::sequence::pair;
use nom::Err as NomErr;
use std::collections::HashMap;

type IResult<I, O, E = (I, ErrorKind)> = Result<(I, O), NomErr<E>>;
type Res<T, U> = IResult<T, U, VerboseError<T>>;

pub mod bytes;
pub mod error;
pub mod packet;
pub mod reason_code;
#[cfg(test)]
mod tests;

pub fn fixed_header(input: &[u8]) -> Res<&[u8], FixedHeader> {
    context("fixed header", pair(be_u8, read_variable_bytes))(input).map(|(next_input, res)| {
        let (fixed_header_byte, (remaining_length, _)) = res;
        (
            next_input,
            FixedHeader {
                packet_type: PacketType::from(fixed_header_byte >> 4),
                dup: (fixed_header_byte >> 3) & 0x01 == 1,
                qos: Qos::from((fixed_header_byte >> 1) & 0x03),
                retain: fixed_header_byte & 0x01 == 1,
                remaining_length,
            },
        )
    })
}

pub fn ping_resp(input: &[u8]) -> Res<&[u8], PingResp> {
    context("pingresp", fixed_header)(input)
        .map(|(next_input, fixed_header)| (next_input, PingResp { fixed_header }))
}

pub fn ping_req(input: &[u8]) -> Res<&[u8], PingReq> {
    context("pingresp", fixed_header)(input)
        .map(|(next_input, fixed_header)| (next_input, PingReq { fixed_header }))
}

pub fn mqtt5_property(input: &[u8]) -> Res<&[u8], Mqtt5Property> {
    context("mqtt5 property", read_variable_bytes)(input).and_then(
        |(input, (property_length, _))| {
            let mut property = Mqtt5Property {
                property_length,
                properties: HashMap::new(),
            };
            let mut property_input = &input[0..property_length];
            let mut subscription_identifiers = vec![];
            let mut user_properties = vec![];
            while property_input.len() > 0 {
                match property_value(property_input) {
                    Ok((next_input, (property_id, property_value))) => {
                        if property_id == 0x0B {
                            subscription_identifiers.push(property_value);
                        } else if property_id == 0x26 {
                            user_properties.push(property_value);
                        } else {
                            property
                                .properties
                                .insert(property_id as u32, property_value);
                        }
                        property_input = next_input;
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                    }
                }
            }
            if subscription_identifiers.len() > 0 {
                property
                    .properties
                    .insert(0x0B, PropertyValue::Multiple(subscription_identifiers));
            }
            if user_properties.len() > 0 {
                property
                    .properties
                    .insert(0x26, PropertyValue::Multiple(user_properties));
            }
            Ok((&input[property_length..], property))
        },
    )
}

/// # Equivalent to the following way of using macros
/// ```
/// #[macro_use]
/// extern crate nom;
/// use nom_mqtt::packet::PropertyValue;
///
/// named!(property_value<&[u8], (usize, usize, PropertyValue)>,
///     switch!(read_variable_bytes,
///         (0x01, res) => do_parse!(
///             payload_format_indicator: be_u8 >>
///             (0x01, res + 1, PropertyValue::Bit(payload_format_indicator & 0x01 == 1))
///         ) |
///         (0x02, res) => do_parse!(
///             message_expiry_interval: be_u32 >>
///             (0x02, res + 4, PropertyValue::FourByteInteger(message_expiry_interval))
///         ) |
///         ...
///     )
/// );
/// ```
pub fn property_value(input: &[u8]) -> Res<&[u8], (usize, PropertyValue)> {
    // Although the Property Identifier is defined as a Variable Byte Integer,
    // in this version of the specification all of the Property Identifiers are one byte long.
    context("property value", be_u8)(input).and_then(|(input, property_id)| {
        return match property_id {
            0x01 => context("payload format indicator", be_u8)(input).map(
                |(input, payload_format_indicator)| {
                    (
                        input,
                        (
                            0x01,
                            PropertyValue::Bit(payload_format_indicator & 0x01 == 1),
                        ),
                    )
                },
            ),
            0x02 => context("message expiry interval", be_u32)(input).map(
                |(input, message_expiry_interval)| {
                    (
                        input,
                        (
                            0x02,
                            PropertyValue::FourByteInteger(message_expiry_interval),
                        ),
                    )
                },
            ),
            0x03 => context("content type", read_string)(input)
                .map(|(input, content_type)| (input, (0x03, PropertyValue::String(content_type)))),
            0x08 => context("response topic", read_string)(input).map(|(input, response_topic)| {
                (input, (0x08, PropertyValue::String(response_topic)))
            }),
            0x09 => {
                context("correlation data", read_bytes)(input).map(|(input, correlation_data)| {
                    (input, (0x09, PropertyValue::Binary(correlation_data)))
                })
            }
            0x0B => context("subscription identifier", read_variable_bytes)(input).and_then(
                |(input, subscription_identifier)| {
                    if subscription_identifier.0 == 0 {
                        return Err(NomErr::Error(VerboseError { errors: vec![] }));
                    }
                    Ok((
                        input,
                        (
                            0x0B,
                            PropertyValue::VariableByteInteger(subscription_identifier.0),
                        ),
                    ))
                },
            ),
            0x11 => context("session expiry interval", be_u32)(input).map(
                |(input, session_expiry_interval)| {
                    (
                        input,
                        (
                            0x11,
                            PropertyValue::FourByteInteger(session_expiry_interval),
                        ),
                    )
                },
            ),
            0x12 => context("assigned client identifier", read_string)(input).map(
                |(input, assigned_client_identifier)| {
                    (
                        input,
                        (0x12, PropertyValue::String(assigned_client_identifier)),
                    )
                },
            ),
            0x13 => {
                context("server keep alive", be_u16)(input).map(|(input, server_keep_alive)| {
                    (
                        input,
                        (0x13, PropertyValue::TwoByteInteger(server_keep_alive)),
                    )
                })
            }
            0x15 => context("authentication method", read_string)(input).map(
                |(input, authentication_method)| {
                    (input, (0x15, PropertyValue::String(authentication_method)))
                },
            ),
            0x16 => context("authentication data", read_bytes)(input).map(
                |(input, authentication_data)| {
                    (input, (0x16, PropertyValue::Binary(authentication_data)))
                },
            ),
            0x17 => context("request problem information", be_u8)(input).map(
                |(input, request_problem_information)| {
                    (
                        input,
                        (
                            0x17,
                            PropertyValue::Bit(request_problem_information & 0x01 == 1),
                        ),
                    )
                },
            ),
            0x18 => {
                context("will delay interval", be_u32)(input).map(|(input, will_delay_interval)| {
                    (
                        input,
                        (0x18, PropertyValue::FourByteInteger(will_delay_interval)),
                    )
                })
            }
            0x19 => context("request response information", be_u8)(input).map(
                |(input, request_response_information)| {
                    (
                        input,
                        (
                            0x19,
                            PropertyValue::Bit(request_response_information & 0x01 == 1),
                        ),
                    )
                },
            ),
            0x1A => context("response information", read_string)(input).map(
                |(input, response_information)| {
                    (input, (0x1A, PropertyValue::String(response_information)))
                },
            ),
            0x1C => {
                context("server reference", read_string)(input).map(|(input, server_reference)| {
                    (input, (0x1C, PropertyValue::String(server_reference)))
                })
            }
            0x1F => context("reason string", read_string)(input).map(|(input, reason_string)| {
                (input, (0x1F, PropertyValue::String(reason_string)))
            }),
            0x21 => context("receive maximum", be_u16)(input).map(|(input, receive_maximum)| {
                (
                    input,
                    (0x21, PropertyValue::TwoByteInteger(receive_maximum)),
                )
            }),
            0x22 => {
                context("topic alias maximum", be_u16)(input).map(|(input, topic_alias_maximum)| {
                    (
                        input,
                        (0x22, PropertyValue::TwoByteInteger(topic_alias_maximum)),
                    )
                })
            }
            0x23 => context("topic alias", be_u16)(input).map(|(input, topic_alias)| {
                (input, (0x23, PropertyValue::TwoByteInteger(topic_alias)))
            }),
            0x24 => context("maximum qos", be_u8)(input)
                .map(|(input, maximum_qos)| (input, (0x24, PropertyValue::Byte(maximum_qos)))),
            0x25 => context("retain available", be_u8)(input).map(|(input, retain_available)| {
                (input, (0x25, PropertyValue::Byte(retain_available)))
            }),
            0x26 => context("user property", pair(read_string, read_string))(input).map(
                |(input, (name, value))| (input, (0x26, PropertyValue::StringPair(name, value))),
            ),
            0x27 => {
                context("maximum packet size", be_u32)(input).map(|(input, maximum_packet_size)| {
                    (
                        input,
                        (0x27, PropertyValue::FourByteInteger(maximum_packet_size)),
                    )
                })
            }
            0x28 => context("wildcard subscription available", be_u8)(input).map(
                |(input, wildcard_subscription_available)| {
                    (
                        input,
                        (0x28, PropertyValue::Byte(wildcard_subscription_available)),
                    )
                },
            ),
            0x29 => context("subscription identifier available", be_u8)(input).map(
                |(input, subscription_identifier_available)| {
                    (
                        input,
                        (0x29, PropertyValue::Byte(subscription_identifier_available)),
                    )
                },
            ),
            0x2A => context("shared subscription available", be_u8)(input).map(
                |(input, shared_subscription_available)| {
                    (
                        input,
                        (0x2A, PropertyValue::Byte(shared_subscription_available)),
                    )
                },
            ),
            _ => Err(NomErr::Error(VerboseError { errors: vec![] })),
        };
    })
}

#[cfg(test)]
mod tests_mqtt {
    use crate::mqtt5_property;

    #[test]
    fn test_mqtt5_property() {
        let vec = &[
            127, 1, 1, 2, 0, 0, 0, 100, 3, 0, 16, 97, 112, 112, 108, 105, 99, 97, 116, 105, 111,
            110, 47, 106, 115, 111, 110, 8, 0, 10, 109, 113, 116, 116, 95, 116, 111, 112, 105, 99,
            9, 0, 7, 109, 113, 116, 116, 49, 50, 51, 17, 0, 0, 0, 5, 18, 0, 3, 100, 100, 100, 19,
            0, 11, 21, 0, 10, 97, 117, 116, 104, 77, 101, 116, 104, 111, 100, 22, 0, 4, 97, 117,
            116, 104, 23, 1, 24, 0, 0, 0, 13, 25, 0, 33, 0, 2, 35, 0, 91, 36, 9, 37, 1, 38, 0, 4,
            110, 97, 109, 101, 0, 6, 105, 97, 109, 97, 122, 121, 38, 0, 3, 97, 103, 101, 0, 2, 50,
            52,
        ];
        match mqtt5_property(vec) {
            Ok(res) => {
                println!("{:?}", res);
            }
            Err(e) => {
                eprintln!("{:?}", e);
            }
        }
    }
}
