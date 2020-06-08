use std::num::NonZeroU16;
use bytes::BufMut;
use crate::Error;

/// Packet Identifier
///
/// when `Qos == 1 || Qos == 2`, `Packet Identifier` should be present in `PUBLISH` Packet
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PacketId(NonZeroU16);

impl PacketId {

    pub fn new() -> Self {
        PacketId(NonZeroU16::new(1).unwrap())
    }

    /// Get `Packet Identifier` as a raw `u16`
    pub fn get(self) -> u16 {
        self.0.get()
    }

    pub(crate) fn to_buf(self, buf: &mut impl BufMut) -> Result<(), Error> {
        Ok(buf.put_u16(self.get()))
    }
}

impl Default for PacketId {
    fn default() -> Self {
        PacketId::new()
    }
}