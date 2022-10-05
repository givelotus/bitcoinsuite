use thiserror::Error;

use crate::SigHashType;

#[derive(Error, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum SignError {
    #[error("Input has no sign data")]
    NoSignData,
    #[error(
        "Could not find script code in sign data. \
             Found neither RedeemScript nor OutputScript"
    )]
    MissingScriptCode,
    #[error(
        "Could not find script code in sign data. \
             Found P2SH OutputScript, but no RedeemScript"
    )]
    MissingScriptCodeP2SH,
    #[error("Could not find value in sign data")]
    MissingValue,
    #[error("Sighash type {0} is invalid")]
    InvalidSigHashType(SigHashType),
    #[error("Invalid script encoding")]
    InvalidScriptEncoding,
    #[error("Multiple leftover outputs not supported")]
    MultipleLeftover,
    #[error("Inputs ({input_sum}) can only pay for {max_fee} fees, but {required_fee} required")]
    InsufficientInputsForFee {
        input_sum: i64,
        max_fee: i64,
        required_fee: i64,
    },
    #[error("OP_CODESEPARATOR #{0} not found")]
    CodesepNotFound(usize),
}

pub type Result<T> = std::result::Result<T, SignError>;
