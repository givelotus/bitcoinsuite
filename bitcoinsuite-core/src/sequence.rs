use crate::{BitcoinCode, Bytes, BytesMut, Result};

pub const CSV_TYPE_FLAG: u32 = 1 << 22;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SequenceNo {
    num: u32,
}

impl SequenceNo {
    pub fn finalized() -> Self {
        SequenceNo { num: 0xffff_ffff }
    }

    pub fn from_u32(num: u32) -> Self {
        SequenceNo { num }
    }

    pub fn as_u32(&self) -> u32 {
        self.num
    }
}

impl BitcoinCode for SequenceNo {
    fn ser_to(&self, bytes: &mut BytesMut) {
        self.num.ser_to(bytes)
    }

    fn deser(data: &mut Bytes) -> Result<Self> {
        Ok(SequenceNo {
            num: u32::deser(data)?,
        })
    }
}
