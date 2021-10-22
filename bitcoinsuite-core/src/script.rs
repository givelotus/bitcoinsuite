use crate::{
    bytes::Bytes,
    ecc::{PubKey, PUBKEY_LENGTH},
    BitcoinCode, BitcoinSuiteError, BytesError, BytesMut, Hashed, Op, Result, ShaRmd160,
};

#[derive(Debug, Clone, PartialEq, Eq, Default, Hash)]
pub struct Script {
    bytecode: Bytes,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScriptVariant {
    P2PK(PubKey),
    P2PKLegacy([u8; 65]),
    P2PKH(ShaRmd160),
    P2SH(ShaRmd160),
    P2TR(PubKey, Option<[u8; 32]>),
    Other(Script),
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

    pub fn p2pk(pubkey: &PubKey) -> Self {
        let mut bytes = BytesMut::new();
        bytes.put_slice(&[0x21]);
        bytes.put_slice(pubkey.as_slice());
        bytes.put_slice(&[0xac]);
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

    pub fn multisig<'a>(num_signers: u8, public_keys: impl IntoIterator<Item = &'a [u8]>) -> Self {
        assert!(num_signers != 0);
        assert!(num_signers <= 16);
        let mut bytes = BytesMut::new();
        bytes.put_slice(&[0x50 + num_signers]);
        let mut num_keys = 0;
        for public_key in public_keys {
            bytes.put_slice(&[public_key.len() as u8]);
            bytes.put_slice(public_key);
            num_keys += 1;
        }
        assert!(num_keys != 0);
        assert!(num_keys <= 16);
        bytes.put_slice(&[0x50 + num_keys as u8, 0xae]);
        Script {
            bytecode: bytes.freeze(),
        }
    }

    pub fn p2tr(commitment: &PubKey, state: Option<[u8; 32]>) -> Self {
        let mut bytes = BytesMut::new();
        bytes.put_slice(&[0x62, 0x51, 0x21]);
        bytes.put_slice(commitment.as_slice());
        if let Some(state) = state {
            bytes.put_slice(&[0x20]);
            bytes.put_slice(&state);
        }
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

    pub fn parse_variant(&self) -> ScriptVariant {
        match self.bytecode.as_ref() {
            [0x21, pubkey @ .., 0xac] if pubkey.len() == PUBKEY_LENGTH => {
                ScriptVariant::P2PK(PubKey::new_unchecked(pubkey.try_into().unwrap()))
            }
            [0x41, pubkey @ .., 0xac] if pubkey.len() == 65 => {
                ScriptVariant::P2PKLegacy(pubkey.try_into().unwrap())
            }
            [0x76, 0xa9, 0x14, hash @ .., 0x88, 0xac] if hash.len() == 20 => {
                ScriptVariant::P2PKH(ShaRmd160::from_slice(hash).unwrap())
            }
            [0xa9, 0x14, hash @ .., 0x87] if hash.len() == 20 => {
                ScriptVariant::P2SH(ShaRmd160::from_slice(hash).unwrap())
            }
            [0x62, 0x51, 0x21, rest @ ..] if rest.len() >= PUBKEY_LENGTH => {
                let commitment = &rest[..PUBKEY_LENGTH];
                let state = match &rest[PUBKEY_LENGTH..] {
                    [] => None,
                    [0x20, state @ ..] if state.len() == 32 => Some(state.try_into().unwrap()),
                    _ => return ScriptVariant::Other(self.clone()),
                };
                ScriptVariant::P2TR(PubKey::new_unchecked(commitment.try_into().unwrap()), state)
            }
            _ => ScriptVariant::Other(self.clone()),
        }
    }

    pub fn parse_p2pkh_spend(&self) -> Option<(Bytes, Bytes)> {
        let mut ops = self.ops();
        let sig_op = ops.next()?.ok()?;
        let sig = match sig_op {
            Op::Push(_, data) => data,
            _ => return None,
        };
        let pubkey_op = ops.next()?.ok()?;
        let pubkey = match pubkey_op {
            Op::Push(_, data) if data.len() == PUBKEY_LENGTH => data,
            _ => return None,
        };
        Some((pubkey, sig))
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
    use hex_literal::hex;

    use crate::{ecc::PubKey, BitcoinSuiteError, Hashed, Script, ScriptVariant, ShaRmd160};

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
    fn test_p2pk() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            Script::p2pk(&PubKey::new_unchecked([0; 33])),
            Script::from_slice(&[
                0x21, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0xac
            ]),
        );
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

    #[test]
    fn test_p2tr() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            Script::p2tr(&PubKey::new_unchecked([0; 33]), None),
            Script::from_slice(&[
                0x62, 0x51, 0x21, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ]),
        );
        assert_eq!(
            Script::p2tr(&PubKey::new_unchecked([1; 33]), Some([2; 32])),
            Script::from_slice(&[
                0x62, 0x51, 0x21, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0x20, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
            ]),
        );
        Ok(())
    }

    #[test]
    fn test_parse_script_variant_p2pk() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            Script::from_hex(
                "21010203040506070809101112131415161718192021222324252627282930313233ac"
            )?
            .parse_variant(),
            ScriptVariant::P2PK(PubKey::new_unchecked(hex!(
                "010203040506070809101112131415161718192021222324252627282930313233"
            ))),
        );
        assert_eq!(
            Script::from_hex(
                "21000000000000000000000000000000000000000000000000000000000000000000ac"
            )?
            .parse_variant(),
            ScriptVariant::P2PK(PubKey::new_unchecked([0; 33])),
        );
        for script_hex in [
            // missing opcodes
            "000000000000000000000000000000000000000000000000000000000000000000ac",
            "21000000000000000000000000000000000000000000000000000000000000000000",
            // wrong opcodes
            "20000000000000000000000000000000000000000000000000000000000000000000ac",
            "21000000000000000000000000000000000000000000000000000000000000000000ab",
            // wrong push size
            "200000000000000000000000000000000000000000000000000000000000000000ac",
        ] {
            assert_eq!(
                Script::from_hex(script_hex)?.parse_variant(),
                ScriptVariant::Other(Script::from_hex(script_hex)?),
            );
        }
        Ok(())
    }

    #[test]
    fn test_parse_script_variant_p2pk_legacy() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            Script::from_hex(
                "410102030405060708091011121314151617181920212223242526272829303132\
                 333435363738394041424344454647484950515253545556575859606162636465ac"
            )?
            .parse_variant(),
            ScriptVariant::P2PKLegacy(
                hex::decode(
                    "0102030405060708091011121314151617181920212223242526272829303132\
                     333435363738394041424344454647484950515253545556575859606162636465"
                )?
                .try_into()
                .unwrap()
            ),
        );
        assert_eq!(
            Script::from_hex(
                "410000000000000000000000000000000000000000000000000000000000000000\
                 000000000000000000000000000000000000000000000000000000000000000000ac"
            )?
            .parse_variant(),
            ScriptVariant::P2PKLegacy([0; 65]),
        );
        for script_hex in [
            // missing opcodes
            "0000000000000000000000000000000000000000000000000000000000000000\
             000000000000000000000000000000000000000000000000000000000000000000ac",
            "410000000000000000000000000000000000000000000000000000000000000000\
             000000000000000000000000000000000000000000000000000000000000000000",
            // wrong opcodes
            "400000000000000000000000000000000000000000000000000000000000000000\
             000000000000000000000000000000000000000000000000000000000000000000ac",
            "410000000000000000000000000000000000000000000000000000000000000000\
             000000000000000000000000000000000000000000000000000000000000000000ab",
            // wrong push size
            "400000000000000000000000000000000000000000000000000000000000000000\
             0000000000000000000000000000000000000000000000000000000000000000ac",
        ] {
            assert_eq!(
                Script::from_hex(script_hex)?.parse_variant(),
                ScriptVariant::Other(Script::from_hex(script_hex)?),
            );
        }
        Ok(())
    }

    #[test]
    fn test_parse_script_variant_p2pkh() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            Script::from_hex("76a91489abcdefabbaabbaabbaabbaabbaabbaabbaabba88ac")?.parse_variant(),
            ScriptVariant::P2PKH(ShaRmd160::from_hex(
                "89abcdefabbaabbaabbaabbaabbaabbaabbaabba"
            )?),
        );
        assert_eq!(
            Script::from_hex("76a914000000000000000000000000000000000000000088ac")?.parse_variant(),
            ScriptVariant::P2PKH(ShaRmd160::new([0; 20])),
        );
        for script_hex in [
            // missing opcodes
            "a914000000000000000000000000000000000000000088ac",
            "7614000000000000000000000000000000000000000088ac",
            "76a9000000000000000000000000000000000000000088ac",
            "76a9140000000000000000000000000000000000000088ac",
            "76a9140000000000000000000000000000000000000000ac",
            "76a914000000000000000000000000000000000000000088",
            // wrong opcodes
            "75a914000000000000000000000000000000000000000088ac",
            "76a814000000000000000000000000000000000000000088ac",
            "76a915000000000000000000000000000000000000000088ac",
            "76a914000000000000000000000000000000000000000087ac",
            "76a914000000000000000000000000000000000000000088ab",
            // wrong push size
            "76a9130000000000000000000000000000000000000088ac",
        ] {
            assert_eq!(
                Script::from_hex(script_hex)?.parse_variant(),
                ScriptVariant::Other(Script::from_hex(script_hex)?),
            );
        }
        Ok(())
    }

    #[test]
    fn test_parse_script_variant_p2sh() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            Script::from_hex("a91489abcdefabbaabbaabbaabbaabbaabbaabbaabba87")?.parse_variant(),
            ScriptVariant::P2SH(ShaRmd160::from_hex(
                "89abcdefabbaabbaabbaabbaabbaabbaabbaabba"
            )?),
        );
        assert_eq!(
            Script::from_hex("a914000000000000000000000000000000000000000087")?.parse_variant(),
            ScriptVariant::P2SH(ShaRmd160::new([0; 20])),
        );
        for script_hex in [
            // missing opcodes
            "14000000000000000000000000000000000000000087",
            "a9000000000000000000000000000000000000000087",
            "a9140000000000000000000000000000000000000087",
            "a9140000000000000000000000000000000000000000",
            // wrong opcodes
            "a814000000000000000000000000000000000000000087",
            "a915000000000000000000000000000000000000000087",
            "a914000000000000000000000000000000000000000088",
            "a9130000000000000000000000000000000000000087",
        ] {
            assert_eq!(
                Script::from_hex(script_hex)?.parse_variant(),
                ScriptVariant::Other(Script::from_hex(script_hex)?),
            );
        }
        Ok(())
    }

    #[test]
    fn test_parse_script_variant_p2tr() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            Script::from_hex(
                "625121010203040506070809101112131415161718192021222324252627282930313233"
            )?
            .parse_variant(),
            ScriptVariant::P2TR(
                PubKey::new_unchecked(hex!(
                    "010203040506070809101112131415161718192021222324252627282930313233"
                )),
                None
            ),
        );
        assert_eq!(
            Script::from_hex(
                "625121000000000000000000000000000000000000000000000000000000000000000000"
            )?
            .parse_variant(),
            ScriptVariant::P2TR(PubKey::new_unchecked([0; 33]), None),
        );
        assert_eq!(
            Script::from_hex(
                "625121010203040506070809101112131415161718192021222324252627282930313233\
                 203231302928272625242322212019181716151413121110090807060504030201"
            )?
            .parse_variant(),
            ScriptVariant::P2TR(
                PubKey::new_unchecked(hex!(
                    "010203040506070809101112131415161718192021222324252627282930313233"
                )),
                Some(hex!(
                    "3231302928272625242322212019181716151413121110090807060504030201"
                ))
            ),
        );
        assert_eq!(
            Script::from_hex(
                "625121000000000000000000000000000000000000000000000000000000000000000000"
            )?
            .parse_variant(),
            ScriptVariant::P2TR(PubKey::new_unchecked([0; 33]), None),
        );
        assert_eq!(
            Script::from_hex(
                "625121000000000000000000000000000000000000000000000000000000000000000000\
                 200000000000000000000000000000000000000000000000000000000000000000"
            )?
            .parse_variant(),
            ScriptVariant::P2TR(PubKey::new_unchecked([0; 33]), Some([0; 32])),
        );
        for script_hex in [
            // missing opcodes
            "5121000000000000000000000000000000000000000000000000000000000000000000",
            "6221000000000000000000000000000000000000000000000000000000000000000000",
            "6251000000000000000000000000000000000000000000000000000000000000000000",
            "6251210000000000000000000000000000000000000000000000000000000000000000",
            "5121000000000000000000000000000000000000000000000000000000000000000000\
             200000000000000000000000000000000000000000000000000000000000000000",
            "6221000000000000000000000000000000000000000000000000000000000000000000\
             200000000000000000000000000000000000000000000000000000000000000000",
            "6251000000000000000000000000000000000000000000000000000000000000000000\
             200000000000000000000000000000000000000000000000000000000000000000",
            "6251210000000000000000000000000000000000000000000000000000000000000000\
             200000000000000000000000000000000000000000000000000000000000000000",
            "625121000000000000000000000000000000000000000000000000000000000000000000\
             0000000000000000000000000000000000000000000000000000000000000000",
            "625121000000000000000000000000000000000000000000000000000000000000000000\
             2000000000000000000000000000000000000000000000000000000000000000",
            // wrong opcodes
            "615121000000000000000000000000000000000000000000000000000000000000000000",
            "625221000000000000000000000000000000000000000000000000000000000000000000",
            "625120000000000000000000000000000000000000000000000000000000000000000000",
            "645121000000000000000000000000000000000000000000000000000000000000000000\
             200000000000000000000000000000000000000000000000000000000000000000",
            "625221000000000000000000000000000000000000000000000000000000000000000000\
             200000000000000000000000000000000000000000000000000000000000000000",
            "625120000000000000000000000000000000000000000000000000000000000000000000\
             200000000000000000000000000000000000000000000000000000000000000000",
            "625121000000000000000000000000000000000000000000000000000000000000000000\
             1f0000000000000000000000000000000000000000000000000000000000000000",
            // wrong push sizes
            "6251200000000000000000000000000000000000000000000000000000000000000000",
            "6251200000000000000000000000000000000000000000000000000000000000000000\
             200000000000000000000000000000000000000000000000000000000000000000",
            "625121000000000000000000000000000000000000000000000000000000000000000000\
             1f00000000000000000000000000000000000000000000000000000000000000",
        ] {
            assert_eq!(
                Script::from_hex(script_hex)?.parse_variant(),
                ScriptVariant::Other(Script::from_hex(script_hex)?),
            );
        }
        Ok(())
    }
}
