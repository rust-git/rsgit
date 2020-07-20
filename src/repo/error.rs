use std::path::PathBuf;

extern crate thiserror;
use thiserror::Error;

/// Describes the potential error conditions that might arise from rsgit `Repo` operations.
#[derive(Debug, Error)]
pub enum Error {
    #[error("work_dir doesn't exist `{0}`")]
    WorkDirDoesntExist(PathBuf),

    #[error("git_dir doesn't exist `{0}`")]
    GitDirDoesntExist(PathBuf),

    #[error("git_dir shouldn't exist `{0}`")]
    GitDirShouldntExist(PathBuf),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    OtherError(#[from] Box<dyn std::error::Error>),
}

/// A specialized `Result` type for rsgit `Repo` operations.
pub type Result<T> = std::result::Result<T, Error>;
