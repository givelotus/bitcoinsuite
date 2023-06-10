use std::borrow::Cow;

use bitcoinsuite_error::{ErrorMeta, Report};
use thiserror::Error;

#[derive(Debug, Error, ErrorMeta, PartialEq, Eq)]
pub enum BitcoindError {
    #[critical()]
    #[error("Bitcoind test instance IO error")]
    TestInstance,

    #[critical()]
    #[error("Bitcoind client IO error")]
    Client,

    #[critical()]
    #[error("JSON RPC error: {0}")]
    JsonRpc(String),

    #[critical()]
    #[error("JSON RPC error ({code}): {message}")]
    JsonRpcCode { code: i32, message: String },

    #[critical()]
    #[error("JSON error")]
    JsonError,

    #[critical()]
    #[error("Invalid UTF8")]
    UTF8,

    #[critical()]
    #[error("Bitcoind exited: {stderr}\ndebug log tail:\n{debug_log_tail}")]
    BitcoindExited {
        stderr: String,
        debug_log_tail: String,
    },

    #[critical()]
    #[error("Timeout {0}")]
    Timeout(Cow<'static, str>),
}

pub fn extract_error_meta(report: &Report) -> Option<&dyn ErrorMeta> {
    if let Some(err) = report.downcast_ref::<BitcoindError>() {
        Some(err)
    } else {
        None
    }
}
