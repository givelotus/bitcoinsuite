use std::borrow::Cow;

use bitcoinsuite_error::{ErrorMeta, Report};
use thiserror::Error;

#[derive(Debug, Error, ErrorMeta)]
pub enum BitcoindError {
    #[critical()]
    #[error("Bitcoind test instance IO error")]
    TestInstance,

    #[critical()]
    #[error("JSON RPC error: {0}")]
    JsonRpc(String),

    #[critical()]
    #[error("JSON error")]
    JsonError,

    #[critical()]
    #[error("Invalid UTF8")]
    UTF8,

    #[critical()]
    #[error("Bitcoind exited")]
    BitcoindExited,

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
