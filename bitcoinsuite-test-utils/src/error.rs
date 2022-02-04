use bitcoinsuite_error::ErrorMeta;
use thiserror::Error;

#[derive(Debug, Error, ErrorMeta)]
pub enum UtilError {
    #[critical()]
    #[error("Picking ports failed")]
    PickPortsFailed,
}
