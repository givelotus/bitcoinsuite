use std::borrow::Cow;

use bitcoinsuite_error::ErrorMeta;
use thiserror::Error;

#[derive(Debug, Error, ErrorMeta)]
pub enum BitcoindError {
    #[critical()]
    #[error("Bitcoind test instance IO error: {0}")]
    TestInstance(std::io::Error),

    #[critical()]
    #[error("JSON RPC error: {0}")]
    JsonRpc(String),

    #[critical()]
    #[error("JSON error: {0}")]
    JsonError(#[from] json::JsonError),

    #[critical()]
    #[error("Invalid UTF8")]
    UTF8(#[from] std::string::FromUtf8Error),

    #[critical()]
    #[error("Bitcoind exited")]
    BitcoindExited,

    #[critical()]
    #[error("Timeout {0}")]
    Timeout(Cow<'static, str>),
}
