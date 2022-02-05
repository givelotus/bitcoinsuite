use std::borrow::Cow;

use bitcoinsuite_test_utils::UtilError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BitcoindError {
    #[error("Bitcoind test instance IO error: {0}")]
    TestInstance(std::io::Error),
    #[error("JSON RPC error: {0}")]
    JsonRpc(String),
    #[error("JSON error: {0}")]
    JsonError(#[from] json::JsonError),
    #[error("Invalid UTF8")]
    UTF8(#[from] std::string::FromUtf8Error),
    #[error("Bitcoind exited")]
    BitcoindExited,
    #[error("Timeout {0}")]
    Timeout(Cow<'static, str>),
    #[error("Utils {0}")]
    Util(#[from] UtilError),
}

pub type Result<T> = std::result::Result<T, BitcoindError>;
