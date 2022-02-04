use std::borrow::Cow;

pub use bitcoinsuite_error_derive::ErrorMeta;
pub use eyre::{bail, Report, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorSeverity {
    InvalidUserInput,
    InvalidClientInput,
    Warning,
    Bug,
    Critical,
}

pub trait ErrorMeta {
    fn severity(&self) -> ErrorSeverity;
    fn error_code(&self) -> Cow<'static, str>;
    fn tags(&self) -> Cow<'static, [(Cow<'static, str>, Cow<'static, str>)]>;
}
