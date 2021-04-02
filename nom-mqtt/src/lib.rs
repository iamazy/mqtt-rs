#[allow(unused)]
#[macro_use]
extern crate nom;

use crate::bytes::{read_bytes, read_string, read_variable_bytes};
use crate::packet::{
    Auth, AuthVariableHeader, ConnAck, ConnAckFlags, ConnAckVariableHeader, Connect, ConnectFlags,
    ConnectPayload, ConnectVariableHeader, Disconnect, DisconnectVariableHeader, FixedHeader,
    Mqtt5Property, Packet, PacketType, PingReq, PingResp, PropertyValue, Protocol, PubAck,
    PubAckVariableHeader, PubComp, PubCompVariableHeader, PubRec, PubRecVariableHeader, PubRel,
    PubRelVariableHeader, Publish, PublishVariableHeader, Qos, SubAck, SubAckVariableHeader,
    Subscribe, SubscribeVariableHeader, SubscriptionOptions, UnSubAck, UnSubAckVariableHeader,
    UnSubscribe, UnSubscribeVariableHeader,
};
use nom::branch::alt;
use nom::bytes::complete::take;
use nom::combinator::{all_consuming, cond, verify, flat_map};
use nom::error::{context, ErrorKind, ParseError, VerboseError};
use nom::number::complete::{be_u16, be_u32, be_u8};
use nom::sequence::{pair, tuple};
use nom::{Err as NomErr, InputIter, InputTake, Parser};
use std::collections::HashMap;
use nom::multi::{many0, fold_many0};

type IResult<I, O, E = (I, ErrorKind)> = Result<(I, O), NomErr<E>>;
type Res<T, U> = IResult<T, U, VerboseError<T>>;

pub mod bytes;
pub mod error;
pub mod packet;
pub mod reason_code;
#[cfg(test)]
mod tests;

pub fn parse(input: &[u8]) -> Res<&[u8], Packet> {
    all_consuming(alt((
        connect,
        connack,
        publish,
        puback,
        pubrec,
        pubrel,
        pubcomp,
        subscribe,
        suback,
        unsubscribe,
        unsuback,
        ping_req,
        ping_resp,
        disconnect,
        auth,
    )))(input)
}

fn auth(input: &[u8]) -> Res<&[u8], Packet> {
    context(
        "auth",
        verify(
            map_fixed_header(
                verify(fixed_header, |fixed_header| {
                    fixed_header.packet_type == PacketType::AUTH
                }),
                auth_variable_header,
            ),
            |(_, _, payload)| payload.len() == 0,
        ),
    )(input)
        .map(|(next_input, (fixed_header, variable_header, _))| {
            (
                next_input,
                Packet::Auth(Auth {
                    fixed_header,
                    variable_header,
                }),
            )
        })
}

fn auth_variable_header(input: &[u8]) -> Res<&[u8], AuthVariableHeader> {
    context("auth variable header", pair(be_u8, mqtt5_property))(input).map(
        |(next_input, (reason_code, auth_property))| {
            (
                next_input,
                AuthVariableHeader {
                    auth_reason_code: reason_code.into(),
                    auth_property,
                },
            )
        },
    )
}

fn connack(input: &[u8]) -> Res<&[u8], Packet> {
    context(
        "connack",
        verify(
            map_fixed_header(
                verify(fixed_header, |fixed_header| {
                    fixed_header.packet_type == PacketType::CONNACK
                }),
                connack_variable_header,
            ),
            |(_, _, payload)| payload.len() == 0,
        ),
    )(input)
        .map(|(next_input, (fixed_header, variable_header, _))| {
            (
                next_input,
                Packet::ConnAck(ConnAck {
                    fixed_header,
                    variable_header,
                }),
            )
        })
}

fn connack_variable_header(input: &[u8]) -> Res<&[u8], ConnAckVariableHeader> {
    context(
        "connack variable header",
        tuple((be_u8, be_u8, mqtt5_property)),
    )(input)
        .map(|(next_input, (flag, reason_code, connack_property))| {
            (
                next_input,
                ConnAckVariableHeader {
                    connack_flags: ConnAckFlags {
                        // fault tolerance
                        session_present: (flag & 0b1111_1111) > 1,
                    },
                    connect_reason_code: reason_code.into(),
                    connack_property,
                },
            )
        })
}

fn disconnect(input: &[u8]) -> Res<&[u8], Packet> {
    context(
        "disconnect",
        verify(
            map_fixed_header(
                verify(fixed_header, |fixed_header| {
                    fixed_header.packet_type == PacketType::DISCONNECT
                }),
                disconnect_variable_header,
            ),
            |(_, _, payload)| payload.len() == 0,
        ),
    )(input)
        .map(|(next_input, (fixed_header, variable_header, _))| {
            (
                next_input,
                Packet::Disconnect(Disconnect {
                    fixed_header,
                    variable_header,
                }),
            )
        })
}

fn disconnect_variable_header(input: &[u8]) -> Res<&[u8], DisconnectVariableHeader> {
    context("disconnect vairable header", pair(be_u8, mqtt5_property))(input).map(
        |(next_input, (reason_code, disconnect_property))| {
            (
                next_input,
                DisconnectVariableHeader {
                    disconnect_reason_code: reason_code.into(),
                    disconnect_property,
                },
            )
        },
    )
}

fn ping_req(input: &[u8]) -> Res<&[u8], Packet> {
    context(
        "pingresp",
        verify(fixed_header, |fixed_header| {
            fixed_header.packet_type == PacketType::PINGREQ
        }),
    )(input)
        .map(|(next_input, fixed_header)| (next_input, Packet::PingReq(PingReq { fixed_header })))
}

fn ping_resp(input: &[u8]) -> Res<&[u8], Packet> {
    context(
        "pingresp",
        verify(fixed_header, |fixed_header| {
            fixed_header.packet_type == PacketType::PINGRESP
        }),
    )(input)
        .map(|(next_input, fixed_header)| (next_input, Packet::PingResp(PingResp { fixed_header })))
}

fn puback(input: &[u8]) -> Res<&[u8], Packet> {
    context(
        "puback",
        verify(
            map_fixed_header(
                verify(fixed_header, |fixed_header| {
                    fixed_header.packet_type == PacketType::PUBACK
                }),
                puback_variable_header,
            ),
            |(_, _, payload)| payload.len() == 0,
        ),
    )(input)
        .map(|(next_input, (fixed_header, variable_header, _))| {
            (
                next_input,
                Packet::PubAck(PubAck {
                    fixed_header,
                    variable_header,
                }),
            )
        })
}

fn puback_variable_header(input: &[u8]) -> Res<&[u8], PubAckVariableHeader> {
    context(
        "puback variable header",
        tuple((be_u16, be_u8, mqtt5_property)),
    )(input)
        .map(|(next_input, (packet_id, reason_code, puback_property))| {
            (
                next_input,
                PubAckVariableHeader {
                    packet_id,
                    puback_reason_code: reason_code.into(),
                    puback_property,
                },
            )
        })
}

fn pubcomp(input: &[u8]) -> Res<&[u8], Packet> {
    context(
        "pubcomp",
        verify(
            map_fixed_header(
                verify(fixed_header, |fixed_header| {
                    fixed_header.packet_type == PacketType::PUBCOMP
                }),
                pubcomp_variable_header,
            ),
            |(_, _, payload)| payload.len() == 0,
        ),
    )(input)
        .map(|(next_input, (fixed_header, variable_header, _))| {
            (
                next_input,
                Packet::PubComp(PubComp {
                    fixed_header,
                    variable_header,
                }),
            )
        })
}

fn pubcomp_variable_header(input: &[u8]) -> Res<&[u8], PubCompVariableHeader> {
    context(
        "pubcomp variable header",
        tuple((be_u16, be_u8, mqtt5_property)),
    )(input)
        .map(|(next_input, (packet_id, reason_code, pubcomp_property))| {
            (
                next_input,
                PubCompVariableHeader {
                    packet_id,
                    pubcomp_reason_code: reason_code.into(),
                    pubcomp_property,
                },
            )
        })
}

fn publish(input: &[u8]) -> Res<&[u8], Packet> {
    context(
        "publish",
        verify(
            map_fixed_header(
                verify(fixed_header, |fixed_header| {
                    fixed_header.packet_type == PacketType::PUBLISH
                }),
                publish_variable_header,
            ),
            |(_, _, payload)| payload.len() == 0,
        ),
    )(input)
        .map(|(next_input, (fixed_header, variable_header, payload))| {
            (
                next_input,
                Packet::Publish(Publish {
                    fixed_header,
                    variable_header,
                    payload,
                }),
            )
        })
}

fn publish_variable_header(input: &[u8]) -> Res<&[u8], PublishVariableHeader> {
    context(
        "publish variable header",
        tuple((read_string, be_u16, mqtt5_property)),
    )(input)
        .map(|(next_input, (topic_name, packet_id, publish_property))| {
            (
                next_input,
                PublishVariableHeader {
                    topic_name,
                    packet_id,
                    publish_property,
                },
            )
        })
}

fn pubrec(input: &[u8]) -> Res<&[u8], Packet> {
    context(
        "pubrec",
        verify(
            map_fixed_header(
                verify(fixed_header, |fixed_header| {
                    fixed_header.packet_type == PacketType::PUBREC
                }),
                pubrec_variable_header,
            ),
            |(_, _, payload)| payload.len() == 0,
        ),
    )(input)
        .map(|(next_input, (fixed_header, variable_header, _))| {
            (
                next_input,
                Packet::PubRec(PubRec {
                    fixed_header,
                    variable_header,
                }),
            )
        })
}

fn pubrec_variable_header(input: &[u8]) -> Res<&[u8], PubRecVariableHeader> {
    context(
        "pubrec variable header",
        tuple((be_u16, be_u8, mqtt5_property)),
    )(input)
        .map(|(next_input, (packet_id, reason_code, pubrec_property))| {
            (
                next_input,
                PubRecVariableHeader {
                    packet_id,
                    pubrec_reason_code: reason_code.into(),
                    pubrec_property,
                },
            )
        })
}

fn pubrel(input: &[u8]) -> Res<&[u8], Packet> {
    context(
        "pubrel",
        verify(
            map_fixed_header(
                verify(fixed_header, |fixed_header| {
                    fixed_header.packet_type == PacketType::PUBREL
                }),
                pubrel_variable_header,
            ),
            |(_, _, payload)| payload.len() == 0,
        ),
    )(input)
        .map(|(next_input, (fixed_header, variable_header, _))| {
            (
                next_input,
                Packet::PubRel(PubRel {
                    fixed_header,
                    variable_header,
                }),
            )
        })
}

fn pubrel_variable_header(input: &[u8]) -> Res<&[u8], PubRelVariableHeader> {
    context(
        "pubrel variable header",
        tuple((be_u16, be_u8, mqtt5_property)),
    )(input)
        .map(|(next_input, (packet_id, reason_code, pubrel_property))| {
            (
                next_input,
                PubRelVariableHeader {
                    packet_id,
                    pubrel_reason_code: reason_code.into(),
                    pubrel_property,
                },
            )
        })
}

fn suback(input: &[u8]) -> Res<&[u8], Packet> {
    context(
        "suback",
        flat_map(map_fixed_header(
            verify(fixed_header, |fixed_header| {
                fixed_header.packet_type == PacketType::SUBACK
            }), suback_variable_header, ),
                 |(fixed_header, variable_header, payloads)|
                     fold_many0(be_u8, (fixed_header, variable_header, Vec::with_capacity(payloads.len())), |(f, v, mut acc), item| {
                         acc.push(item.into());
                         (f, v, acc)
                     })),
    )(input)
        .map(|(next_input, (fixed_header, variable_header, payload))| {
            (
                next_input,
                Packet::SubAck(SubAck {
                    fixed_header,
                    variable_header,
                    payload,
                }),
            )
        })
}

fn suback_variable_header(input: &[u8]) -> Res<&[u8], SubAckVariableHeader> {
    context("suback variable header", pair(be_u16, mqtt5_property))(input).map(
        |(next_input, (packet_id, suback_property))| {
            (
                next_input,
                SubAckVariableHeader {
                    packet_id,
                    suback_property,
                },
            )
        },
    )
}

fn subscribe(input: &[u8]) -> Res<&[u8], Packet> {
    context(
        "subscribe",
        map_fixed_header(
            verify(fixed_header, |fixed_header| {
                fixed_header.packet_type == PacketType::SUBSCRIBE
            }),
            subscribe_variable_header,
        ),
    )(input)
        .and_then(
            |(next_input, (fixed_header, variable_header, payloads))| {
                let (_, payload) = many0(pair(read_string, subscription_options))(payloads)?;
                Ok((
                    next_input,
                    Packet::Subscribe(Subscribe {
                        fixed_header,
                        variable_header,
                        payload,
                    }),
                ))
            },
        )
}

fn subscription_options(input: &[u8]) -> Res<&[u8], SubscriptionOptions> {
    context("subscription options", be_u8)(input).map(|(next_input, option)| {
        (
            next_input,
            SubscriptionOptions {
                maximum_qos: (option & 0b0000_0011).into(),
                no_local: (option >> 2) & 0x01 == 1,
                retain_as_published: (option >> 3) & 0x01 == 1,
                retain_handling: (option >> 4) & 0x03,
            },
        )
    })
}

fn subscribe_variable_header(input: &[u8]) -> Res<&[u8], SubscribeVariableHeader> {
    context("subscribe variable header", pair(be_u16, mqtt5_property))(input).map(
        |(next_input, (packet_id, subscribe_property))| {
            (
                next_input,
                SubscribeVariableHeader {
                    packet_id,
                    subscribe_property,
                },
            )
        },
    )
}

fn unsuback(input: &[u8]) -> Res<&[u8], Packet> {
    context(
        "unsuback",
        map_fixed_header(
            verify(fixed_header, |fixed_header| {
                fixed_header.packet_type == PacketType::UNSUBACK
            }),
            unsuback_variable_header,
        ),
    )(input)
        .and_then(|(next_input, (fixed_header, variable_header, payloads))| {
            let (_, payload) = fold_many0(be_u8, Vec::with_capacity(payloads.len()), |mut acc, item| {
                acc.push(item.into());
                acc
            })(payloads)?;
            Ok((
                next_input,
                Packet::UnSubAck(UnSubAck {
                    fixed_header,
                    variable_header,
                    payload,
                }),
            ))
        })
}

fn unsuback_variable_header(input: &[u8]) -> Res<&[u8], UnSubAckVariableHeader> {
    context("unsuback variable header", pair(be_u16, mqtt5_property))(input).map(
        |(next_input, (packet_id, unsuback_property))| {
            (
                next_input,
                UnSubAckVariableHeader {
                    packet_id,
                    unsuback_property,
                },
            )
        },
    )
}

fn unsubscribe(input: &[u8]) -> Res<&[u8], Packet> {
    context(
        "unsubscribe",
        map_fixed_header(
            verify(fixed_header, |fixed_header| {
                fixed_header.packet_type == PacketType::UNSUBSCRIBE
            }),
            unsubscribe_variable_header,
        ),
    )(input)
        .and_then(
            |(next_input, (fixed_header, variable_header, payloads))| {
                let (_, payload) = many0(read_string)(payloads)?;
                Ok((
                    next_input,
                    Packet::UnSubscribe(UnSubscribe {
                        fixed_header,
                        variable_header,
                        payload,
                    }),
                ))
            },
        )
}

fn unsubscribe_variable_header(input: &[u8]) -> Res<&[u8], UnSubscribeVariableHeader> {
    context("unsubscribe variable header", pair(be_u16, mqtt5_property))(input).map(
        |(next_input, (packet_id, unsubscribe_property))| {
            (
                next_input,
                UnSubscribeVariableHeader {
                    packet_id,
                    unsubscribe_property,
                },
            )
        },
    )
}

fn connect(input: &[u8]) -> Res<&[u8], Packet> {
    context(
        "connect",
        map_fixed_header(
            verify(fixed_header, |s| s.packet_type == PacketType::CONNECT),
            connect_variable_header,
        ),
    )(input)
        .and_then(|(next_input, (fixed_header, variable_header, payloads))| {
            let (payloads, client_id) = read_string(payloads)?;
            let (payloads, (will_property, will_topic, will_payload)) =
                match cond(variable_header.connect_flags.will_flag, tuple((mqtt5_property, read_string, read_bytes)))(payloads)? {
                    (payloads, Some((property, topic, payload))) => {
                        (payloads, (Some(property), Some(topic), Some(payload)))
                    }
                    (payloads, None) => (payloads, (None, None, None)),
                };
            let (_, (username, password)) = pair(
                cond(variable_header.connect_flags.username_flag, read_string),
                cond(variable_header.connect_flags.password_flag, read_string),
            )(payloads)?;
            Ok((
                next_input,
                Packet::Connect(Connect {
                    fixed_header,
                    variable_header,
                    payload: ConnectPayload {
                        client_id,
                        will_property,
                        will_topic,
                        will_payload,
                        username,
                        password,
                    },
                }),
            ))
        })
}

fn connect_flag(input: &[u8]) -> Res<&[u8], ConnectFlags> {
    context("connect flag", be_u8)(input).map(|(next_input, flag)| {
        (
            next_input,
            ConnectFlags {
                clean_start: (flag >> 1) & 0x01 > 0,
                will_flag: (flag >> 2) & 0x01 > 0,
                will_qos: ((flag >> 3) & 0x03).into(),
                will_retain: (flag >> 5) & 0x01 > 0,
                password_flag: (flag >> 6) & 0x01 > 0,
                username_flag: (flag >> 7) & 0x01 > 0,
            },
        )
    })
}

fn connect_variable_header(input: &[u8]) -> Res<&[u8], ConnectVariableHeader> {
    context(
        "connect variable header",
        tuple((protocol, connect_flag, be_u16, mqtt5_property)),
    )(input)
        .map(
            |(next_input, (protocol, connect_flags, keep_alive, connect_property))| {
                (
                    next_input,
                    ConnectVariableHeader {
                        protocol,
                        connect_flags,
                        keep_alive,
                        connect_property,
                    },
                )
            },
        )
}

fn protocol(input: &[u8]) -> Res<&[u8], Protocol> {
    context("protocol", pair(read_string, be_u8))(input).and_then(|(next_input, (name, level))| {
        if name == "MQTT" && level == 5u8 {
            return Ok((next_input, Protocol::MQTT5));
        }
        return Err(NomErr::Error(VerboseError { errors: vec![] }));
    })
}

pub fn map_fixed_header<I: Clone + InputIter + InputTake, O, E: ParseError<I>, F, G>(
    mut first: F,
    mut second: G,
) -> impl FnMut(I) -> IResult<I, (FixedHeader, O, I), E>
    where
        F: Parser<I, FixedHeader, E>,
        G: Parser<I, O, E>,
{
    move |input: I| {
        let (input, fixed_header) = first.parse(input)?;
        let (input, variable_header_and_payload) = take(fixed_header.remaining_length)(input)?;
        let (payload, variable_header) = second.parse(variable_header_and_payload)?;
        Ok((input, (fixed_header, variable_header, payload)))
    }
}

fn fixed_header(input: &[u8]) -> Res<&[u8], FixedHeader> {
    context("fixed header", pair(be_u8, read_variable_bytes))(input).map(|(next_input, (fixed_header_byte, (remaining_length, _)))| {
        (
            next_input,
            FixedHeader {
                packet_type: (fixed_header_byte >> 4).into(),
                dup: (fixed_header_byte >> 3) & 0x01 == 1,
                qos: Qos::from((fixed_header_byte >> 1) & 0x03),
                retain: fixed_header_byte & 0x01 == 1,
                remaining_length,
            },
        )
    })
}

fn mqtt5_property(input: &[u8]) -> Res<&[u8], Mqtt5Property> {
    context("mqtt5 property", read_variable_bytes)(input).and_then(
        |(input, (property_length, _))| {
            let mut property = Mqtt5Property {
                property_length,
                properties: HashMap::new(),
            };
            let (next_input, mut property_input) = take(property_length)(input)?;
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
            Ok((next_input, property))
        },
    )
}

fn property_value(input: &[u8]) -> Res<&[u8], (usize, PropertyValue)> {
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
    use crate::{mqtt5_property, parse};

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

    #[test]
    fn test_connect() {
        let vec = &[
            0b0001_0000u8, 52, // fixed header
            0x00, 0x04, 'M' as u8, 'Q' as u8, 'T' as u8, 'T' as u8,     // protocol name
            0x05,          // protocol version
            0b1100_1110u8, // connect flag
            0x00, 0x10, // keep alive
            0x05, 0x11, 0x00, 0x00, 0x00, 0x10, // connect properties
            0x00, 0x03, 'c' as u8, 'i' as u8, 'd' as u8, // client id
            0x05, 0x02, 0x00, 0x00, 0x00, 0x10, // will properties
            0x00, 0x04, 'w' as u8, 'i' as u8, 'l' as u8, 'l' as u8, // will topic
            0x00, 0x01, 'p' as u8, // will payload
            0x00, 0x06, 'i' as u8, 'a' as u8, 'm' as u8, 'a' as u8, 'z' as u8, 'y' as u8, // username
            0x00, 0x06, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06,
        ];
        match parse(vec) {
            Ok(res) => {
                println!("{:?}", res);
            }
            Err(e) => {
                eprintln!("{:?}", e);
            }
        }
    }

    #[test]
    fn test_suback() {
        let vec = &[
            0b1001_0000u8, 11, // fixed header
            0x00, 0x10, // packet identifier
            5,    // properties length
            0x1F, // property id
            0x00, 0x02, 'I' as u8, 'a' as u8, // reason string
            0x00, 0x01, 0x02,
        ];
        match parse(vec) {
            Ok(res) => {
                println!("{:?}", res);
            }
            Err(e) => {
                eprintln!("{:?}", e);
            }
        }
    }
}
