use std::path::PathBuf;
use thiserror::Error;

/// Possible errors when running a bulk rename.
#[derive(Debug, Error)]
pub enum Error {
    /// The provided path is not a directory.
    #[error("path is not a directory")]
    NotDirError,
    /// The provided regular expression is invalid.
    #[error("invalid regex: {0}")]
    RegexError(#[from] regex::Error),
    /// A generic I/O error occurred during renaming.
    #[error("I/O error at {path}: {source}")]
    IoError {
        path: PathBuf,
        source: std::io::Error,
    },
}
