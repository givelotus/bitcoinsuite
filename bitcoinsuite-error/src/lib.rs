use std::{borrow::Cow, fmt::Display, sync::atomic};

use lazy_static::lazy_static;

pub use bitcoinsuite_error_derive::ErrorMeta;
pub use eyre::{bail, Report, Result, WrapErr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorSeverity {
    Unknown,
    NotFound,
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

pub fn report_to_details(
    report: &Report,
    detail_func: impl Fn(&Report) -> Option<&dyn ErrorMeta>,
) -> ErrorDetails {
    let short_msg = report.to_string();
    let msg = report.fmt_err();
    let full_debug_report = format!("{:?}", report);
    let meta = detail_func(report);
    match meta {
        Some(meta) => ErrorDetails {
            severity: meta.severity(),
            error_code: meta.error_code(),
            tags: meta.tags(),
            short_msg,
            msg,
            full_debug_report,
        },
        None => ErrorDetails {
            severity: ErrorSeverity::Unknown,
            error_code: "unknown".into(),
            tags: [].as_ref().into(),
            short_msg,
            msg,
            full_debug_report,
        },
    }
}

impl Display for ErrorSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
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
