//! A repository that stores content on the local file system.
//!
//! **IMPORTANT NOTE:** This is intended as a reference implementation largely
//! for testing purposes and may not necessarily handle all of the edge cases that
//! the traditional `git` command-line interface will handle.
//!
//! That said, it does intentionally use the same `.git` folder format as
//! command-line git so that results may be compared for similar operations.

mod init;
pub use init::init;
