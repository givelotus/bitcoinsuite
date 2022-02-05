use std::{borrow::Cow, sync::atomic};

use lazy_static::lazy_static;

pub use bitcoinsuite_error_derive::ErrorMeta;
pub use eyre::{bail, Report, Result, WrapErr};

pub type ErrorMetaFunc<'a> = &'a dyn Fn(&Report) -> Option<&dyn ErrorMeta>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorSeverity {
    Unknown,
    InvalidUserInput,
    InvalidClientInput,
    Warning,
    Bug,
    Critical,
}

#[derive(Debug, Clone)]
pub struct ErrorDetails {
    pub severity: ErrorSeverity,
    pub error_code: Cow<'static, str>,
    pub tags: Cow<'static, [(Cow<'static, str>, Cow<'static, str>)]>,
    pub short_msg: String,
    pub msg: String,
    pub full_debug_report: String,
}

pub trait ErrorMeta {
    fn severity(&self) -> ErrorSeverity;
    fn error_code(&self) -> Cow<'static, str>;
    fn tags(&self) -> Cow<'static, [(Cow<'static, str>, Cow<'static, str>)]>;
}

pub trait ErrorFmt {
    fn fmt_err(&self) -> String;
}

impl ErrorFmt for eyre::Report {
    fn fmt_err(&self) -> String {
        format!("{:#}", self)
    }
}

lazy_static! {
    static ref ERROR_HANDLE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    static ref ERROR_HANDLE_IS_STARTED: atomic::AtomicBool = atomic::AtomicBool::new(false);
}

pub fn install() -> Result<()> {
    let lock = ERROR_HANDLE_LOCK.lock().unwrap();
    let is_started = ERROR_HANDLE_IS_STARTED.load(atomic::Ordering::SeqCst);
    if !is_started {
        stable_eyre::install()?;
        ERROR_HANDLE_IS_STARTED.store(true, atomic::Ordering::SeqCst);
    }
    std::mem::drop(lock);
    Ok(())
}
