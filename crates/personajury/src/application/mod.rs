//! adapted from comparegraph crates/comparegraph/src/application/mod.rs (AppError, codes)
//! Workflows, one module per verb. Each returns the envelope's `data` as a serde_json::Value.
pub mod apply_check;
pub mod brief;
pub mod calibrate;
pub mod check;
pub mod decide;
pub mod decision;
pub mod deliberate;
pub mod delta;
pub mod doctor;
pub mod graph;
pub mod init;
pub mod lens;
pub mod matrix;
pub mod panel;
pub mod persona;
pub mod pick;
pub mod setup;
pub mod should_run;
pub mod subject;
pub mod traits;
pub mod summary;
pub mod verdict;
use crate::domain::Refusal;
use serde_json::Value;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Store(#[from] crate::store::StoreError),
    /// a refusal of report §5 / §10; its code is the envelope's error code
    #[error(transparent)]
    Refused(#[from] Refusal),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    InvalidArguments(String),
    /// a gate that failed with data to show (check: unresolved findings)
    #[error("{0}")]
    CheckFailed(String, Value),
    #[error("{0}: {1}")]
    Io(PathBuf, std::io::Error),
    #[error("summary --save needs report_dir in personajury.toml (for example report_dir = \"docs/juries\"); nothing is written by default")]
    NoReportDir,
    /// a calibration file written for another subject, lens, lens version or run (M1)
    #[error("{0}")]
    CalibrationMismatch(String),
    /// a report directory or an MCP output path that is absolute, holds `..`, or resolves outside the root (I2)
    #[error("{0}")]
    InvalidReportDir(String),
    #[error("{0} is not implemented yet")]
    NotImplemented(&'static str),
}
impl AppError {
    /// Stable error codes.
    pub fn code(&self) -> &'static str {
        match self {
            AppError::Store(crate::store::StoreError::NotFound(_)) => "not_found",
            AppError::Store(_) => "store_error",
            AppError::Refused(r) => r.code(),
            AppError::NotFound(_) => "not_found",
            AppError::InvalidArguments(_) => "invalid_arguments",
            AppError::CheckFailed(..) => "check_failed",
            AppError::Io(..) => "io_error",
            AppError::NotImplemented(_) => "not_implemented",
            AppError::NoReportDir => "no_report_dir",
            AppError::CalibrationMismatch(_) => "calibration_mismatch",
            AppError::InvalidReportDir(_) => "invalid_report_dir",
        }
    }
    /// Data carried by a failing gate, shown in the envelope beside the error.
    pub fn data(&self) -> Option<Value> {
        match self {
            AppError::CheckFailed(_, v) => Some(v.clone()),
            AppError::Refused(r) => r.data(),
            _ => None,
        }
    }
}
pub type Result<T> = std::result::Result<T, AppError>;

/// An I/O error on a path the caller named: a missing file is `not_found`, anything else `io_error`.
pub(crate) fn io_err(p: &std::path::Path, e: std::io::Error) -> AppError {
    if e.kind() == std::io::ErrorKind::NotFound {
        AppError::NotFound(format!("no such file: {}", p.display()))
    } else {
        AppError::Io(p.to_path_buf(), e)
    }
}
/// Read a file the caller named (`--file`, `--attach`, `--diff`, `--config`).
pub(crate) fn read_text(p: &std::path::Path) -> Result<String> {
    std::fs::read_to_string(p).map_err(|e| io_err(p, e))
}
/// Refuse an id or name that is not a slug (I6); checked where it is added and again before a path join.
pub(crate) fn slug(what: &str, s: &str) -> Result<()> {
    crate::domain::check_slug(what, s).map_err(AppError::InvalidArguments)
}
impl From<rusqlite::Error> for AppError {
    fn from(e: rusqlite::Error) -> Self {
        AppError::Store(e.into())
    }
}
