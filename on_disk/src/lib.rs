//! This crate provides a git repository that stores content on the local file system.
//!
//! **IMPORTANT NOTE:** This is intended as a reference implementation largely
//! for testing purposes and may not necessarily handle all of the edge cases that
//! the traditional `git` command-line interface will handle.
//!
//! That said, it does intentionally use the same `.git` folder format as
//! command-line git so that results may be compared for similar operations.

#![deny(warnings)]

mod on_disk_repo;
pub use on_disk_repo::OnDiskRepo;

mod temp_git_repo;
pub use temp_git_repo::TempGitRepo;
