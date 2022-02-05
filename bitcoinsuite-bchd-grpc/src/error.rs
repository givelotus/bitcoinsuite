use bitcoinsuite_test_utils::UtilError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BchdError {
    #[error("Cert file IO error: {0}")]
    CertFile(std::io::Error),
    #[error("BCHD test instance IO error: {0}")]
    TestInstanceIo(std::io::Error),
    #[error("Tonic connection error: {0}")]
    TonicTransport(#[from] tonic::transport::Error),
    #[error("Util: {0}")]
    Util(#[from] UtilError),
}

pub type Result<T> = std::result::Result<T, BchdError>;
