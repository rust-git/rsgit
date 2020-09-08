use std::path::PathBuf;

use thiserror::Error;

/// Describes the potential error conditions that might arise from rsgit [`Repo`] operations.
///
/// [`Repo`]: trait.Repo.html
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

/// A specialized [`Result`] type for rsgit [`Repo`] operations.
///
/// [`Repo`]: trait.Repo.html
/// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
pub type Result<T> = std::result::Result<T, Error>;
