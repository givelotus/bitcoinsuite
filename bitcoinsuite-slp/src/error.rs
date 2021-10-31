use bitcoinsuite_core::{Bytes, BytesError};
use thiserror::Error;

use crate::SlpAmount;

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum SlpError {
    #[error("First must be OP_RETURN (0x6a), but got 0x{opcode:02x}")]
    MissingOpReturn { opcode: u8 },
    #[error("Tx has no outputs")]
    NoOutputs,
    #[error("Non-push op: 0x{opcode:02x} at op {op_idx}")]
    NonPushOp { opcode: u8, op_idx: usize },
    #[error("Disallowed push: 0x{opcode:02x} at op {op_idx}")]
    DisallowedPush { opcode: u8, op_idx: usize },
    #[error(
        "Field has invalid length: expected one of {expected:?} but got {actual} for field \
        {field_name}"
    )]
    InvalidFieldSize {
        field_name: &'static str,
        expected: &'static [usize],
        actual: usize,
    },
    #[error("Too many decimals, only max. 9 allowed, but got {actual}")]
    InvalidDecimals { actual: usize },
    #[error("Mint baton at invalid output index, must be between 2 and 255, but got {actual}")]
    InvalidMintBatonIdx { actual: usize },
    #[error("NFT1 Child Genesis cannot have mint baton")]
    Nft1ChildCannotHaveMintBaton,
    #[error("Invalid NFT1 Child Genesis initial quantity, expected 1 but got {actual}")]
    Nft1ChildInvalidInitialQuantity { actual: SlpAmount },
    #[error("Invalid NFT1 Child Genesis decimals, expected 0 but got {actual}")]
    Nft1ChildInvalidDecimals { actual: u32 },
    #[error("Too few pushes, expected at least {expected} but only got {actual}")]
    TooFewPushes { expected: usize, actual: usize },
    #[error("Too few pushes, expected exactly {expected} but only got {actual}")]
    TooFewPushesExact { expected: usize, actual: usize },
    #[error("Pushed superfluous data: expected at most {expected} pushes, but got {actual}")]
    SuperfluousPushes { expected: usize, actual: usize },
    #[error("Invalid LOKAD ID: {}", .0.hex())]
    InvalidLokadId(Bytes),
    #[error("Token type has invalid length (1,2 != {}): {}", .0.len(), .0.hex())]
    InvalidTokenType(Bytes),
    #[error("Invalid tx type: {}", .0.hex())]
    InvalidTxType(Bytes),
    #[error("Invalid SEND: Output amounts ({output_sum}) exceed input amounts ({input_sum})")]
    OutputSumExceedInputSum {
        output_sum: SlpAmount,
        input_sum: SlpAmount,
    },
    #[error("Invalid NFT1 Child GENESIS: No group token")]
    HasNoNft1Group,
    #[error("Invalid MINT: No baton")]
    HasNoMintBaton,
    #[error("Found orphan txs")]
    FoundOrphanTx,
    #[error("Bytes error: {0}")]
    BytesError(#[from] BytesError),
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum BitcoinSuiteSlpError {
    #[error("Unknown coin protocol: {0}")]
    UnknownCoinProtocol(String),
}
