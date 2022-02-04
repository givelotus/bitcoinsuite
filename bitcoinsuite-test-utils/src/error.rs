use bitcoinsuite_error::{ErrorMeta, Report};
use thiserror::Error;

#[derive(Debug, Error, ErrorMeta)]
pub enum UtilError {
    #[critical()]
    #[error("Picking ports failed")]
    PickPortsFailed,
}

pub fn extract_error_meta(report: &Report) -> Option<&dyn ErrorMeta> {
    if let Some(err) = report.downcast_ref::<UtilError>() {
        Some(err)
    } else {
        None
    }
}
