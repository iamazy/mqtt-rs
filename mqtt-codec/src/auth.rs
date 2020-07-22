use crate::fixed_header::FixedHeader;
use crate::{Mqtt5Property, FromToU8, Error, Frame};
use bytes::{BytesMut, BufMut, Buf};
use crate::publish::Qos;
use crate::packet::{PacketType, PacketCodec};

#[derive(Debug, Clone, PartialEq)]
pub struct Auth {
    fixed_header: FixedHeader,
    variable_header: AuthVariableHeader
}

impl PacketCodec<Auth> for Auth {
    fn from_buf_extra(buf: &mut BytesMut, fixed_header: FixedHeader) -> Result<Auth, Error> {
        let variable_header = AuthVariableHeader::from_buf(buf)
            .expect("Failed to parse Auth Variable Header");
        Ok(Auth {
            fixed_header,
            variable_header
        })
    }
}

impl Default for Auth {
    fn default() -> Self {
        let variable_header = AuthVariableHeader::default();
        Auth {
            fixed_header: FixedHeader {
                packet_type: PacketType::AUTH,
                dup: false,
                qos: Qos::AtMostOnce,
                retain: false,
                remaining_length: 0,
            },
            variable_header,
        }
    }
}

impl Frame<Auth> for Auth {
    fn to_buf(&self, buf: &mut impl BufMut) -> usize {
        let mut len = self.fixed_header.to_buf(buf);
        len += self.variable_header.to_buf(buf);
        len
    }

    fn from_buf(buf: &mut BytesMut) -> Result<Auth, Error> {
        let fixed_header = Auth::decode_fixed_header(buf);
        assert_eq!(fixed_header.packet_type, PacketType::AUTH);
        assert_eq!(fixed_header.dup, false, "The dup of Auth Fixed Header must be set to false");
        assert_eq!(fixed_header.qos, Qos::AtMostOnce, "The qos of Auth Fixed Header must be set to be AtMostOnce");
        assert_eq!(fixed_header.retain, false, "The retain of Auth Fixed Header must be set to false");
        Auth::from_buf_extra(buf, fixed_header)
    }

    fn length(&self) -> usize {
        self.fixed_header.length() + self.variable_header.length()
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AuthVariableHeader {
    reason_code: AuthenticateReasonCode,
    auth_property: Mqtt5Property
}

impl AuthVariableHeader {
    fn check_auth_property(auth_property: &mut Mqtt5Property) -> Result<(), Error> {
        for key in auth_property.properties.keys() {
            let key = *key;
            match key {
                0x15 | 0x16 | 0x1F | 0x26 => {},
                _ => return Err(Error::InvalidPropertyType("Auth Properties contains a invalid property".to_string()))
            }
        }
        Ok(())
    }
}

impl Frame<AuthVariableHeader> for AuthVariableHeader {
    fn to_buf(&self, buf: &mut impl BufMut) -> usize {
        buf.put_u8(self.reason_code.to_u8());
        let mut len = 0;
        len += self.auth_property.to_buf(buf);
        len
    }

    fn from_buf(buf: &mut BytesMut) -> Result<AuthVariableHeader, Error> {
        let reason_code = AuthenticateReasonCode::from_u8(buf.get_u8())
            .expect("Failed to parse Authenticate Reason Code");
        let mut auth_property = Mqtt5Property::from_buf(buf)
            .expect("Failed to parse Auth Properties");
        AuthVariableHeader::check_auth_property(&mut auth_property)?;
        Ok(AuthVariableHeader {
            reason_code,
            auth_property
        })
    }

    fn length(&self) -> usize {
        1 + self.auth_property.length()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AuthenticateReasonCode{
    /// 0[0x00], Authentication is successful
    Success = 0x00,
    /// 24[0x18], Continue the authentication with another step
    ContinueAuthentication = 0x18,
    /// 25[0x19], Initiate a re-authentication
    ReAuthenticate = 0x19
}

impl Default for AuthenticateReasonCode {
    fn default() -> Self {
        AuthenticateReasonCode::ReAuthenticate
    }
}

impl FromToU8<AuthenticateReasonCode> for AuthenticateReasonCode {
    fn to_u8(&self) -> u8 {
        match *self {
            AuthenticateReasonCode::Success => 0,
            AuthenticateReasonCode::ContinueAuthentication => 24,
            AuthenticateReasonCode::ReAuthenticate => 25
        }
    }

    fn from_u8(byte: u8) -> Result<AuthenticateReasonCode, Error> {
        match byte {
            0 => Ok(AuthenticateReasonCode::Success),
            24 => Ok(AuthenticateReasonCode::ContinueAuthentication),
            25 => Ok(AuthenticateReasonCode::ReAuthenticate),
            n => Err(Error::InvalidReasonCode(n))
        }
    }
}

#[cfg(test)]
mod test {
    use bytes::{BytesMut, BufMut};
    use crate::Frame;
    use crate::auth::Auth;

    #[test]
    fn test_auth() {
        let auth_bytes = &[
            0b1111_0000u8, 8,  // fixed header,
            0x00, // authenticate reason code
            6, // properties length
            0x1F, // reason string identifier
            0x00, 0x03, 'h' as u8, 'e' as u8, 'l' as u8,
        ];
        let mut buf = BytesMut::with_capacity(64);
        buf.put_slice(auth_bytes);
        let auth = Auth::from_buf(&mut buf)
            .expect("Failed to parse Auth Packet");

        let mut buf = BytesMut::with_capacity(64);
        auth.to_buf(&mut buf);
        assert_eq!(auth, Auth::from_buf(&mut buf).unwrap());
    }
}