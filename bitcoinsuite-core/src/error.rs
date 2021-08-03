use hex::FromHexError;
use thiserror::Error;

use crate::BytesError;

#[derive(Error, Debug)]
pub enum BitcoinSuiteError {
    #[error("Bytes error: {0}")]
    Bytes(#[from] BytesError),
    #[error("Invalid size: expected {expected}, got {actual}")]
    InvalidSize { expected: usize, actual: usize },
    #[error("Inconsistent Op::Push: {0:02x} is not a valid push opcode")]
    InconsistentOpPush(u8),
    #[error("Parsing number failed")]
    NumberParseError,
    #[error("Unknown network: {0}")]
    UnknownNetwork(String),
    #[error("From hex error: {0}")]
    Hex(#[from] FromHexError),
}

pub type Result<T> = std::result::Result<T, BitcoinSuiteError>;
