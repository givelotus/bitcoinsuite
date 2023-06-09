use std::fmt::{Debug, Display};

use bitcoinsuite_core::{Hashed, Result, Sha256d};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct TokenId {
    txid: Sha256d,
    token_id_be: [u8; 32],
}

impl TokenId {
    pub fn from_slice_be(token_id: &[u8]) -> Result<Self> {
        Ok(TokenId::new(Sha256d::from_slice_be(token_id)?))
    }

    pub fn from_slice_be_or_null(token_id: &[u8]) -> Self {
        TokenId::new(Sha256d::from_slice_be_or_null(token_id))
    }

    pub fn from_token_id_hex(token_id_hex: &str) -> Result<Self> {
        Ok(TokenId::new(Sha256d::from_hex_be(token_id_hex)?))
    }

    pub fn new(token_hash: Sha256d) -> Self {
        let mut token_id_be = token_hash.byte_array().array();
        token_id_be.reverse();
        TokenId {
            token_id_be,
            txid: token_hash,
        }
    }

    pub fn as_slice_be(&self) -> &[u8] {
        &self.token_id_be
    }

    pub fn token_id_be(&self) -> [u8; 32] {
        self.token_id_be
    }

    pub fn hash(&self) -> &Sha256d {
        &self.txid
    }
}

impl Debug for TokenId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TokenId({})", self.txid)
    }
}

impl Display for TokenId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <Sha256d as Display>::fmt(&self.txid, f)
    }
}
