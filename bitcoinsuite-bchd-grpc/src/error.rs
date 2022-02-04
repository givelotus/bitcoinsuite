use bitcoinsuite_error::ErrorMeta;
use thiserror::Error;

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
