use crate::{
    bytes::Bytes,
    ecc::{PubKey, PUBKEY_LENGTH},
    BitcoinCode, BitcoinSuiteError, BytesError, BytesMut, Hashed, Op, Result, ShaRmd160,
};

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
        bytes.put_byte_array(hash.byte_array().clone());
        bytes.put_slice(&[0x88, 0xac]);
        Script {
            bytecode: bytes.freeze(),
        }
    }

    pub fn p2pkh_spend(pubkey: &PubKey, sig: Bytes) -> Self {
        let mut bytes = BytesMut::new();
        bytes.put_slice(&[sig.len() as u8]);
        bytes.put_bytes(sig);
        bytes.put_slice(&[PUBKEY_LENGTH as u8]);
        bytes.put_slice(pubkey.as_slice());
        Script {
            bytecode: bytes.freeze(),
        }
    }

    pub fn p2sh(hash: &ShaRmd160) -> Self {
        let mut bytes = BytesMut::new();
        bytes.put_slice(&[0xa9, 0x14]);
        bytes.put_byte_array(hash.byte_array().clone());
        bytes.put_slice(&[0x87]);
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

    pub fn is_p2sh(&self) -> bool {
        matches!(self.bytecode.as_ref(), [0xa9, 0x14, hash @ .., 0x87] if hash.len() == 20)
    }

    pub fn is_opreturn(&self) -> bool {
        self.bytecode
            .get(0)
            .map(|&opcode| opcode == 0x6a)
            .unwrap_or_default()
    }

    pub fn cut_out_codesep(&self, n_codesep: Option<usize>) -> Result<Script> {
        if let Some(n_codesep) = n_codesep {
            let mut n_codeseps_found = 0;
            let mut ops = self.ops();
            while let Some(op) = ops.next() {
                if let Op::Code(0xab) = op? {
                    if n_codesep == n_codeseps_found {
                        return Ok(Script::new(ops.remaining_bytecode));
                    }
                    n_codeseps_found += 1;
                }
            }
            return Err(BitcoinSuiteError::CodesepNotFound(n_codesep));
        }
        Ok(self.clone())
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

#[cfg(test)]
mod tests {
    use crate::{ecc::PubKey, BitcoinSuiteError, Script, ShaRmd160};

    #[test]
    fn test_cut_out_codesep_without() -> Result<(), Box<dyn std::error::Error>> {
        let script = Script::from_slice(&[0x51, 0x52, 0x93, 0x53, 0x87]);
        assert_eq!(script.cut_out_codesep(None)?, script);
        for i in 0..100 {
            match script.cut_out_codesep(Some(i)) {
                Err(BitcoinSuiteError::CodesepNotFound(pos)) => assert_eq!(pos, i),
                other => panic!("Unexpected result: {:?}", other),
            }
        }
        Ok(())
    }

    #[test]
    fn test_cut_out_codesep() -> Result<(), Box<dyn std::error::Error>> {
        let script =
            Script::from_slice(&[0x51, 0xab, 0x52, 0xab, 0xab, 0x93, 0x53, 0xab, 0x87, 0xab]);
        assert_eq!(script.cut_out_codesep(None)?, script);
        assert_eq!(
            script.cut_out_codesep(Some(0))?,
            Script::from_slice(&[0x52, 0xab, 0xab, 0x93, 0x53, 0xab, 0x87, 0xab])
        );
        assert_eq!(
            script.cut_out_codesep(Some(1))?,
            Script::from_slice(&[0xab, 0x93, 0x53, 0xab, 0x87, 0xab])
        );
        assert_eq!(
            script.cut_out_codesep(Some(2))?,
            Script::from_slice(&[0x93, 0x53, 0xab, 0x87, 0xab])
        );
        assert_eq!(
            script.cut_out_codesep(Some(3))?,
            Script::from_slice(&[0x87, 0xab])
        );
        assert_eq!(script.cut_out_codesep(Some(4))?, Script::from_slice(&[]));
        match script.cut_out_codesep(Some(5)) {
            Err(BitcoinSuiteError::CodesepNotFound(5)) => {}
            other => panic!("Unexpected result: {:?}", other),
        }
        Ok(())
    }

    #[test]
    fn test_p2pkh() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            Script::p2pkh(&ShaRmd160::new([0; 20])),
            Script::from_slice(&[
                0x76, 0xa9, 0x14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x88,
                0xac
            ]),
        );
        assert_eq!(
            Script::p2pkh(&ShaRmd160::new([0xff; 20])),
            Script::from_slice(&[
                0x76, 0xa9, 0x14, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x88, 0xac
            ]),
        );
        Ok(())
    }

    #[test]
    fn test_p2pkh_spend() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            Script::p2pkh_spend(&PubKey::new_unchecked([2; 33]), [7; 64].into()),
            Script::from_slice(&[
                0x40, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07,
                0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07,
                0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07,
                0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07,
                0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x07, 0x21, 0x02, 0x02, 0x02, 0x02,
                0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,
                0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02, 0x02,
                0x02
            ]),
        );
        Ok(())
    }

    #[test]
    fn test_p2sh() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            Script::p2sh(&ShaRmd160::new([0; 20])),
            Script::from_slice(&[
                0xa9, 0x14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x87,
            ]),
        );
        assert_eq!(
            Script::p2sh(&ShaRmd160::new([0xff; 20])),
            Script::from_slice(&[
                0xa9, 0x14, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x87,
            ]),
        );
        Ok(())
    }

    #[test]
    fn test_is_p2sh() -> Result<(), Box<dyn std::error::Error>> {
        assert!(Script::from_slice(&[
            0xa9, 0x14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x87,
        ])
        .is_p2sh());
        assert!(Script::from_slice(&[
            0xa9, 0x14, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x87,
        ])
        .is_p2sh());
        assert!(!Script::from_slice(&[
            0xa9, 0x14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x87,
        ])
        .is_p2sh());
        assert!(!Script::from_slice(&[
            0xa9, 0x14, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x88,
        ])
        .is_p2sh());
        assert!(!Script::from_slice(&[
            0xa9, 0x15, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x87,
        ])
        .is_p2sh());
        Ok(())
    }
}
