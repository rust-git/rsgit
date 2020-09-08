//! Represents an abstract git repository.
//!
//! ## Design Goals
//!
//! Rsgit intends to allow repositories to be stored in multiple different mechanisms.
//! While it includes built-in support for local on-disk repositories
//! (see [`OnDisk`]), you could envision repositories stored entirely
//! in memory, or on a remote file system or database.
//! 
//! [`OnDisk`]: struct.OnDisk.html

use crate::object::Object;

mod error;
pub use error::{Error, Result};

mod on_disk;
pub use on_disk::OnDisk;

/// A struct that implements the `Repo` trait represents a particular mechanism
/// for storing and accessing a git repo.
///
/// The required methods on this trait represent the most primitive operations
/// which must be defined for a given storage architecture. Consider the
/// information stored in a typical `.git` directory in a local repository. You will
/// be building an alternative to that storage mechanism.
///
/// The provided methods on this trait represent the common "porcelain" and "plumbing"
/// operations for a git repo, regardless of its storage mechanism.
pub trait Repo {
    /// Writes a loose object to the repository.
    ///
    /// This is analogous to [`git hash-object -w`].
    /// 
    /// [`git hash-object -w`]: https://git-scm.com/docs/git-hash-object#Documentation/git-hash-object.txt--w
    fn put_loose_object(&mut self, object: &Object) -> Result<()>;
}
