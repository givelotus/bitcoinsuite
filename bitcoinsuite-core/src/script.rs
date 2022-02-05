use crate::{bytes::Bytes, BitcoinCode, BytesError, BytesMut, Hashed, Op, Result, ShaRmd160};

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct Script {
    bytecode: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct ScriptOpIter {
    remaining_bytecode: Bytes,
}

impl BitcoinCode for Script {
    fn ser_to(&self, bytes: &mut crate::BytesMut) {
        self.bytecode.ser_to(bytes)
    }

    fn deser(data: &mut Bytes) -> crate::Result<Self> {
        Ok(Script {
            bytecode: Bytes::deser(data)?,
        })
    }
}

impl Script {
    pub fn new(bytecode: Bytes) -> Self {
        Script { bytecode }
    }

    pub fn from_slice(slice: &[u8]) -> Self {
        Script {
            bytecode: Bytes::from_slice(slice),
        }
    }

    pub fn from_static_slice(slice: &'static [u8]) -> Self {
        Script {
            bytecode: Bytes::from_bytes(slice),
        }
    }

    pub fn from_hex(hex: &str) -> crate::Result<Self> {
        Ok(Script::new(Bytes::from_bytes(hex::decode(hex)?)))
    }

    pub fn opreturn(data: &[&[u8]]) -> Self {
        let mut bytes = BytesMut::new();
        bytes.put_slice(&[0x6a]);
        for &item in data {
            if item.is_empty() {
                bytes.put_slice(&[0x4c, 0]);
            } else if item.len() < 0x4c {
                bytes.put_slice(&[item.len() as u8]);
                bytes.put_slice(item);
            } else {
                bytes.put_slice(&[0x4c]);
                bytes.put_slice(&[item.len() as u8]);
                bytes.put_slice(item);
            }
        }
        Script {
            bytecode: bytes.freeze(),
        }
    }

    pub fn p2pkh(hash: &ShaRmd160) -> Self {
        let mut bytes = BytesMut::new();
        bytes.put_slice(&[0x76, 0xa9, 0x14]);
        bytes.put_slice(hash.as_slice());
        bytes.put_slice(&[0x88, 0xac]);
        Script {
            bytecode: bytes.freeze(),
        }
    }

    pub fn bytecode(&self) -> &Bytes {
        &self.bytecode
    }

    pub fn from_ops(ops_iter: impl Iterator<Item = Op>) -> Result<Self> {
        let mut bytecode = BytesMut::new();
        for op in ops_iter {
            op.ser_op(&mut bytecode)?;
        }
        Ok(Script {
            bytecode: bytecode.freeze(),
        })
    }

    pub fn ops(&self) -> ScriptOpIter {
        ScriptOpIter {
            remaining_bytecode: self.bytecode.clone(),
        }
    }

    pub fn hex(&self) -> String {
        self.bytecode.hex()
    }

    pub fn to_p2sh(&self) -> Self {
        let mut bytes = BytesMut::new();
        bytes.put_slice(&[0xa9, 0x14]);
        bytes.put_slice(ShaRmd160::digest(self.bytecode.clone()).as_slice());
        bytes.put_slice(&[0x87]);
        Script {
            bytecode: bytes.freeze(),
        }
    }
}

impl Iterator for ScriptOpIter {
    type Item = std::result::Result<Op, BytesError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_bytecode.is_empty() {
            None
        } else {
            Some(Op::deser_op(&mut self.remaining_bytecode))
        }
    }
}
