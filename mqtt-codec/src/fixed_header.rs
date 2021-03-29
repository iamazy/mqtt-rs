use crate::packet::PacketType;
use crate::publish::Qos;
use crate::{read_variable_bytes, write_variable_bytes, Error, Frame, FromToU8};
use bytes::{Buf, BufMut, BytesMut};

/// # Fixed Header format
///
/// <table style="border: 0" cellspacing="0" cellpadding="0">
///  <tbody><tr>
///   <td style="width:300px; height: 25px;border: 1px #A4A4A4 solid">
///   <p align="center" style="text-align:center"><b>Bit</b></p>
///   </td>
///   <td style="width:300px;border: 1px #A4A4A4 solid">
///   <p align="center" style="text-align:center"><b>7</b></p>
///   </td>
///   <td style="width:300px;border: 1px #A4A4A4 solid">
///   <p align="center" style="text-align:center"><b>6</b></p>
///   </td>
///   <td style="width:300px;border: 1px #A4A4A4 solid">
///   <p align="center" style="text-align:center"><b>5</b></p>
///   </td>
///   <td style="width:300px;border: 1px #A4A4A4 solid">
///   <p align="center" style="text-align:center"><b>4</b></p>
///   </td>
///   <td style="width:300px;border: 1px #A4A4A4 solid">
///   <p align="center" style="text-align:center"><b>3</b></p>
///   </td>
///   <td style="width:300px;border: 1px #A4A4A4 solid">
///   <p align="center" style="text-align:center"><b>2</b></p>
///   </td>
///   <td style="width:300px;border: 1px #A4A4A4 solid">
///   <p align="center" style="text-align:center"><b>1</b></p>
///   </td>
///   <td style="width:300px;border: 1px #A4A4A4 solid">
///   <p align="center" style="text-align:center"><b>0</b></p>
///   </td>
///  </tr>
///  <tr>
///   <td style="width:300px;height: 25px;border: 1px #A4A4A4 solid">
///   <p style="text-align:center">byte 1</p>
///   </td>
///   <td style="width:300px;border: 1px #A4A4A4 solid" colspan="4">
///   <p align="center" style="text-align:center" >MQTT Control Packet
///   type</p>
///   </td>
///   <td style="width:300px;border: 1px #A4A4A4 solid" colspan="4">
///   <p align="center" style="text-align:center">Flags specific to
///   each MQTT Control Packet type</p>
///   </td>
///  </tr>
///  <tr>
///   <td style="width:300px;height: 25px;border: 1px #A4A4A4 solid">
///   <p style="text-align:center">byte 2…</p>
///   </td>
///   <td style="width:300px;border: 1px #A4A4A4 solid" colspan="8">
///   <p align="center" style="text-align:center">Remaining Length</p>
///   </td>
///  </tr>
/// </tbody></table>
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FixedHeader {
    pub packet_type: PacketType,
    pub dup: bool,
    pub qos: Qos,
    pub retain: bool,
    /// This is the length of the Variable Header plus the length of the Payload. It is encoded as a Variable Byte Integer
    pub remaining_length: usize,
}

impl Frame<FixedHeader> for FixedHeader {
    fn to_buf(&self, buf: &mut impl BufMut) -> usize {
        let packet_type = self.packet_type.clone();
        let mut byte = packet_type.to_u8() << 4;
        if self.retain {
            byte |= 0x01;
        }
        byte |= self.qos.to_u8() << 1;
        if self.dup {
            byte |= 0b0000_1000;
        }
        let mut len = 1;
        buf.put_u8(byte);
        len += write_variable_bytes(self.remaining_length, |byte| buf.put_u8(byte));
        len
    }

    fn from_buf(buf: &mut BytesMut) -> Result<FixedHeader, Error> {
        let fixed_header_byte = buf.get_u8();
        let packet_type = PacketType::from_u8(fixed_header_byte >> 4)
            .expect("Failed to parse Packet Type in Fixed Header");
        let dup = (fixed_header_byte >> 3) & 0x01 == 1;
        let qos = Qos::from_u8((fixed_header_byte >> 1) & 0x03)?;
        let retain = fixed_header_byte & 0x01 == 1;
        let remaining_length = read_variable_bytes(buf)
            .expect("Failed to parse Fixed Header Remaining Length")
            .0;
        #[rustfmt::skip]
        assert_eq!(remaining_length, buf.len(), "The remaining_length of FixedHeader is invalid");
        Ok(FixedHeader {
            packet_type,
            dup,
            qos,
            retain,
            remaining_length,
        })
    }

    fn length(&self) -> usize {
        1 + write_variable_bytes(self.remaining_length, |_| {})
    }
}

#[cfg(test)]
mod test {
    use crate::pingresp::PingResp;
    use crate::{Frame, FixedHeader};
    use bytes::{BufMut, BytesMut};

    #[test]
    #[rustfmt::skip]
    fn test_fixed_header() {
        let bytes = &[
            0b1101_0000u8, 0, // fixed header
        ];
        let mut buf = BytesMut::with_capacity(64);
        buf.put_slice(bytes);
        let fixed_header = FixedHeader::from_buf(&mut buf).expect("Failed to parse Fixed Header");

        let mut buf = BytesMut::with_capacity(64);
        fixed_header.to_buf(&mut buf);
        println!("{:?}", fixed_header);
        assert_eq!(fixed_header, FixedHeader::from_buf(&mut buf).unwrap());
    }
}