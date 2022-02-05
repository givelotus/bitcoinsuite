use hex_literal::hex;

use crate::{ecc::EccError, ByteArray};

pub const PUBKEY_LENGTH: usize = 33;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PubKey([u8; PUBKEY_LENGTH]);

impl PubKey {
    pub fn new_unchecked(pubkey: [u8; PUBKEY_LENGTH]) -> Self {
        PubKey(pubkey)
    }

    pub fn from_hex_unchecked(hex: &str) -> Result<Self, EccError> {
        let pubkey = hex::decode(hex)?;
        Ok(PubKey(
            pubkey
                .as_slice()
                .try_into()
                .map_err(|_| EccError::InvalidPubKeyLen(pubkey.len()))?,
        ))
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    pub fn hex(&self) -> String {
        ByteArray::new(self.0).hex()
    }

    pub fn array(&self) -> [u8; PUBKEY_LENGTH] {
        self.0
    }
}

impl Default for PubKey {
    fn default() -> Self {
        PubKey(hex!(
            "020000000000000000000000000000000000000000000000000000000000000001"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::PubKey;

    #[test]
    fn test_as_slice() {
        let pubkey = PubKey::new_unchecked([2; 33]);
        assert_eq!(pubkey.as_slice(), &[2; 33]);
    }

    #[test]
    fn test_hex() {
        let pubkey = PubKey::new_unchecked([2; 33]);
        assert_eq!(
            pubkey.hex(),
            "020202020202020202020202020202020202020202020202020202020202020202"
        );
    }

    #[test]
    fn test_default() {
        let pubkey = PubKey::default();
        assert_eq!(
            pubkey.hex(),
            "020000000000000000000000000000000000000000000000000000000000000001"
        );
    }
}
