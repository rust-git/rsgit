use std::{env, path::Path};

use rsgit_core::repo::Result;
use rsgit_on_disk::OnDiskRepo;

// Discover a git repo starting from the given path.
//
// Git comes with many configuration options and algorithms
// for finding a repo. Some of these may eventually be
// replicated here, which is why this function should be
// called for repo discovery.
//
// For now, however, this handles only the most simple case
// where there is a `.git` directory nested within the
// given path.
//
// Returns a `Result` with `rsgit_core::repo::OnDiskRepo` or
// `rsgit_core::repo::Error` if no such repo exists.
pub fn from_path<P: AsRef<Path>>(path: P) -> Result<OnDiskRepo> {
    // TO DO: Look in other places for repo.
    // https://github.com/rust-git/rsgit/issues/80
    OnDiskRepo::new(path)
}

// Discover a git repo starting from the current working directory.
//
// Git comes with many configuration options and algorithms
// for finding a repo. Some of these may eventually be
// replicated here, which is why this function should be
// called for repo discovery.
//
// For now, however, this handles only the most simple case
// where there is a `.git` directory nested within the
// given path.
//
// Returns a `Result` with `rsgit_on_disk::OnDiskRepo` or
// `rsgit_core::repo::Error` if no such repo exists.
#[cfg(not(tarpaulin_include))]
pub fn from_current_dir() -> Result<OnDiskRepo> {
    // This function is excluded from code coverage because we can't
    // be sure of the execution environment while testing. So we keep
    // it as simple as possible.
    let path = env::current_dir()?;
    from_path(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    use rsgit_core::repo::Error;

    use rsgit_on_disk::TempGitRepo;

    #[test]
    fn simple_case() {
        let tgr = TempGitRepo::new();
        let path = tgr.path();
        let repo = from_path(path).unwrap();
        assert_eq!(repo.work_dir(), path);
    }

    #[test]
    fn work_dir_doesnt_exist() {
        let temp_dir = tempfile::tempdir().unwrap();
        let mut path = temp_dir.path().to_path_buf();
        path.push("nope");

        let err = from_path(&path).unwrap_err();
        if let Error::WorkDirDoesntExist(err_path) = err {
            assert_eq!(err_path, path);
        } else {
            panic!("Unexpected error response: {:?}", err);
        }
    }

    #[test]
    fn git_dir_doesnt_exist() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path();

        let mut git_dir = path.to_path_buf();
        git_dir.push(".git"); // but we don't create it

        let err = from_path(&path).unwrap_err();
        if let Error::GitDirDoesntExist(err_path) = err {
            assert_eq!(err_path, git_dir.as_path());
        } else {
            panic!("Unexpected error response: {:?}", err);
        }
    }
}
