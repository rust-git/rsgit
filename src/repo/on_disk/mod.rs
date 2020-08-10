//! A repository that stores content on the local file system.
//!
//! **IMPORTANT NOTE:** This is intended as a reference implementation largely
//! for testing purposes and may not necessarily handle all of the edge cases that
//! the traditional `git` command-line interface will handle.
//!
//! That said, it does intentionally use the same `.git` folder format as
//! command-line git so that results may be compared for similar operations.

use std::fs;
use std::path::{Path, PathBuf};

use super::{Error, Repo, Result};

/// Implementation of `rsgit::Repo` that stores content on the local file system.
///
/// _IMPORTANT NOTE:_ This is intended as a reference implementation largely
/// for testing purposes and may not necessarily handle all of the edge cases that
/// the traditional `git` command-line interface will handle.
///
/// That said, it does intentionally use the same `.git` folder format as command-line
/// `git` so that results may be compared for similar operations.
#[derive(Debug)]
pub struct OnDisk {
    #[allow(dead_code)] // TEMPORARY: Remove once we start consuming this.
    work_dir: PathBuf,

    #[allow(dead_code)] // TEMPORARY: Remove once we start consuming this.
    git_dir: PathBuf,
}

impl OnDisk {
    /// Create an on-disk git repository.
    ///
    /// `work_dir` should be the top-level working directory. A `.git` directory should
    /// exist at this path. Use `init` function to create an empty on-disk repository if
    /// necessary.
    pub fn new(work_dir: &Path) -> Result<Self> {
        let work_dir = work_dir.to_path_buf();
        if !work_dir.exists() {
            return Err(Error::WorkDirDoesntExist(work_dir));
        }

        let git_dir = work_dir.join(".git");
        if !git_dir.exists() {
            return Err(Error::GitDirDoesntExist(git_dir));
        }

        Ok(OnDisk { work_dir, git_dir })
    }

    /// Creates a new, empty git repository on the local file system.
    ///
    /// Analogous to [`git init`](https://git-scm.com/docs/git-init).
    pub fn init(work_dir: &Path) -> Result<Self> {
        let git_dir = work_dir.join(".git");
        if git_dir.exists() {
            return Err(Error::GitDirShouldntExist(git_dir));
        }

        fs::create_dir_all(&git_dir)?;

        create_config(&git_dir)?;
        create_description(&git_dir)?;
        create_head(&git_dir)?;
        create_hooks_dir(&git_dir)?;
        create_info_dir(&git_dir)?;
        create_objects_dir(&git_dir)?;
        create_refs_dir(&git_dir)?;

        Ok(OnDisk {
            work_dir: work_dir.to_path_buf(),
            git_dir,
        })
    }

    /// Return the working directory for this repo.
    pub fn work_dir(&self) -> &Path {
        self.work_dir.as_path()
    }

    /// Return the path to the `.git` directory.
    pub fn git_dir(&self) -> &Path {
        self.git_dir.as_path()
    }
}

impl Repo for OnDisk {}

fn create_config(git_dir: &Path) -> Result<()> {
    let config_path = git_dir.join("config");
    let config_txt = "[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n\tlogallrefupdates = true\n";

    fs::write(config_path, config_txt).map_err(|e| e.into())
}

fn create_description(git_dir: &Path) -> Result<()> {
    let desc_path = git_dir.join("description");
    let desc_txt = "Unnamed repository; edit this file 'description' to name the repository.\n";

    fs::write(desc_path, desc_txt).map_err(|e| e.into())
}

fn create_head(git_dir: &Path) -> Result<()> {
    let head_path = git_dir.join("HEAD");
    let head_txt = "ref: refs/heads/master\n";

    fs::write(head_path, head_txt).map_err(|e| e.into())
}

fn create_hooks_dir(git_dir: &Path) -> Result<()> {
    let hooks_dir = git_dir.join("hooks");
    fs::create_dir_all(&hooks_dir).map_err(|e| e.into())

    // NOTE: Intentionally not including the sample files.
}

fn create_info_dir(git_dir: &Path) -> Result<()> {
    let info_dir = git_dir.join("info");
    fs::create_dir_all(&info_dir)?;

    let exclude_path = info_dir.join("exclude");
    let exclude_txt = "# git ls-files --others --exclude-from=.git/info/exclude\n# Lines that start with '#' are comments.\n# For a project mostly in C, the following would be a good set of\n# exclude patterns (uncomment them if you want to use them):\n# *.[oa]\n# *~\n.DS_Store\n";

    fs::write(exclude_path, exclude_txt).map_err(|e| e.into())
}

fn create_objects_dir(git_dir: &Path) -> Result<()> {
    let info_dir = git_dir.join("objects/info");
    fs::create_dir_all(&info_dir)?;

    let pack_dir = git_dir.join("objects/pack");
    fs::create_dir_all(&pack_dir).map_err(|e| e.into())
}

fn create_refs_dir(git_dir: &Path) -> Result<()> {
    let heads_dir = git_dir.join("refs/heads");
    fs::create_dir_all(&heads_dir)?;

    let tags_dir = git_dir.join("refs/tags");
    fs::create_dir_all(&tags_dir).map_err(|e| e.into())
}

#[cfg(test)]
mod tests;
