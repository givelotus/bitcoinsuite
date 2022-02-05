use hex::FromHexError;
use thiserror::Error;

use crate::{ecc::EccError, BytesError, SignError};

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
    #[error("Invalid VarInt")]
    InvalidVarInt,
    #[error("Unknown network: {0}")]
    UnknownNetwork(String),
    #[error("OP_CODESEPARATOR #{0} not found")]
    CodesepNotFound(usize),
    #[error("From hex error: {0}")]
    Hex(#[from] FromHexError),
    #[error("Sign error: {0}")]
    Sign(#[from] SignError),
    #[error("Ecc error: {0}")]
    Ecc(#[from] EccError),
}

pub type Result<T> = std::result::Result<T, BitcoinSuiteError>;
