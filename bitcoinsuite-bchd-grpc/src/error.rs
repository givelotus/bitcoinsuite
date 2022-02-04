use bitcoinsuite_error::ErrorMeta;
use thiserror::Error;

#[derive(Debug, Error, ErrorMeta)]
pub enum BchdError {
    #[critical()]
    #[error("Cert file IO error: {0}")]
    CertFile(std::io::Error),

    #[critical()]
    #[error("BCHD test instance IO error: {0}")]
    TestInstanceIo(std::io::Error),

    #[critical()]
    #[error("Tonic connection error: {0}")]
    TonicTransport(#[from] tonic::transport::Error),
}
