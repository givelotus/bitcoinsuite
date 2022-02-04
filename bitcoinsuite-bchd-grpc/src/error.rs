use bitcoinsuite_error::{ErrorMeta, Report};
use thiserror::Error;

use crate::BchdSlpError;

#[derive(Debug, Error, ErrorMeta)]
pub enum BchdError {
    #[critical()]
    #[error("Cert file IO error")]
    CertFile,

    #[critical()]
    #[error("BCHD test instance IO error")]
    TestInstanceIo,

    #[critical()]
    #[error("Tonic connection error")]
    TonicTransport,
}

pub fn extract_error_meta(report: &Report) -> Option<&dyn ErrorMeta> {
    if let Some(err) = report.downcast_ref::<BchdError>() {
        Some(err)
    } else if let Some(err) = report.downcast_ref::<BchdSlpError>() {
        Some(err)
    } else {
        None
    }
}
