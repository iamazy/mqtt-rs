use crate::frame::FixedHeader;
use crate::{Mqtt5Property, FromToU8, Error, FromToBuf};
use bytes::{BytesMut, BufMut, Buf};
use crate::publish::Qos;
use crate::packet::PacketType;

#[derive(Debug, Clone, PartialEq)]
pub struct Auth {
    fixed_header: FixedHeader,
    auth_variable_header: AuthVariableHeader
}

impl FromToBuf<Auth> for Auth {
    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        let mut len = self.fixed_header.to_buf(buf)?;
        len += self.auth_variable_header.to_buf(buf)?;
        Ok(len)
    }

    fn from_buf(buf: &mut BytesMut) -> Result<Auth, Error> {
        let fixed_header = FixedHeader::from_buf(buf)
            .expect("Failed to parse Auth Fixed Header");
        assert_eq!(fixed_header.packet_type, PacketType::AUTH);
        assert_eq!(fixed_header.dup, false, "The dup of Auth Fixed Header must be set to false");
        assert_eq!(fixed_header.qos, Qos::AtMostOnce, "The qos of Auth Fixed Header must be set to be AtMostOnce");
        assert_eq!(fixed_header.retain, false, "The retain of Auth Fixed Header must be set to false");
        let auth_variable_header = AuthVariableHeader::from_buf(buf)
            .expect("Failed to parse Auth Variable Header");
        Ok(Auth {
            fixed_header,
            auth_variable_header
        })

    }
}

#[derive(Debug, Clone, PartialEq)]
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

impl FromToBuf<AuthVariableHeader> for AuthVariableHeader {
    fn to_buf(&self, buf: &mut impl BufMut) -> Result<usize, Error> {
        buf.put_u8(self.reason_code.to_u8());
        let mut len = 0;
        len += self.auth_property.to_buf(buf)?;
        Ok(len)
    }

    fn from_buf(buf: &mut BytesMut) -> Result<AuthVariableHeader, Error> {
        let reason_code = AuthenticateReasonCode::from_u8(buf.get_u8())
            .expect("Failed to parse Authenticate Reason Code");
        let auth_property = Mqtt5Property::from_buf(buf)
            .expect("Failed to parse Auth Propertoes");
        Ok(AuthVariableHeader {
            reason_code,
            auth_property
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AuthenticateReasonCode{
    /// 0[0x00], Authentication is successful
    Success,
    /// 24[0x18], Continue the authentication with another step
    ContinueAuthentication,
    /// 25[0x19], Initiate a re-authentication
    ReAuthenticate
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