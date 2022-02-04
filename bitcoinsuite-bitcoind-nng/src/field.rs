use std::borrow::Cow;

use bitcoinsuite_error::{ErrorMeta, Result};
use thiserror::Error;

#[derive(Error, Debug, ErrorMeta)]
pub enum FlatbufferFieldError {
    #[critical()]
    #[error("MissingField: {0}")]
    MissingField(Cow<'static, str>),
}

pub trait OptionExt: Sized {
    type T;
    fn field(self, name: &'static str) -> Result<Self::T>;
}

impl<T> OptionExt for Option<T> {
    type T = T;
    fn field(self, name: &'static str) -> Result<T> {
        Ok(self.ok_or_else(|| FlatbufferFieldError::MissingField(name.into()))?)
    }
}
