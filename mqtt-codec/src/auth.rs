use crate::frame::FixedHeader;
use crate::{Mqtt5Property, FromToU8, Error, FromToBuf};
use bytes::{BytesMut, BufMut, Buf};
use crate::publish::Qos;

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
        let fixed_header = FixedHeader::new(buf, false, Qos::AtMostOnce, false)
            .expect("Failed to parse Auth Fixed Header");
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