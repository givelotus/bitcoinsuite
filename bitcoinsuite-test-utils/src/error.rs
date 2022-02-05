use thiserror::Error;

#[derive(Debug, Error)]
pub enum UtilError {
    #[error("Picking ports failed")]
    PickPortsFailed,
}

pub type Result<T> = std::result::Result<T, UtilError>;
