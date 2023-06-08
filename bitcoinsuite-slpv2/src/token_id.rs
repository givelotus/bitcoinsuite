use std::{fmt::Debug, str::FromStr};

use bitcoinsuite_core::{Hashed, Result, Sha256d};

#[derive(Clone, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TokenId(Sha256d);

impl TokenId {
    pub fn from_hex(token_id_hex: &str) -> Result<Self> {
        Ok(TokenId(Sha256d::from_hex_be(token_id_hex)?))
    }

    pub fn from_slice(token_id: &[u8]) -> Result<Self> {
        Ok(TokenId(Sha256d::from_slice(token_id)?))
    }

    pub fn new(token_hash: Sha256d) -> Self {
        TokenId(token_hash)
    }

    pub fn hash(&self) -> &Sha256d {
        &self.0
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        self.0.byte_array().as_array()
    }
}

impl FromStr for TokenId {
    type Err = bitcoinsuite_core::BitcoinSuiteError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(TokenId::new(Sha256d::from_hex_be(s)?))
    }
}

impl Debug for TokenId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TokenId({})", self.0)
    }
}

impl std::fmt::Display for TokenId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_hex_be())
    }
}
