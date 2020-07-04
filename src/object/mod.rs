//! Represents the git concept of an "object" which is a tuple of
//! object type and binary data identified by the hash of the binary data.

use std::io::{self, BufRead, BufReader};

use sha1::{Digest, Sha1};

mod attribution;
pub use attribution::Attribution;

// TO DO: Make pub(crate)
pub mod check_commit;

mod content_source;
pub use content_source::ContentSource;

mod file_content_source;
pub use file_content_source::FileContentSource;

mod id;
pub use id::{Id, ParseIdError, ParseIdErrorKind};

mod kind;
pub use kind::Kind;

pub(crate) mod parse_utils;

/// Describes a single object stored (or about to be stored) in a git repository.
///
/// This struct is constructed, modified, and shared as a working description of
/// how to find and describe an object before it gets written to a repository.
pub struct Object {
    id: Option<Id>,
    kind: Kind,
    content_source: Box<dyn ContentSource>,
}

impl Object {
    /// Create a new Object.
    pub fn new(kind: Kind, content_source: Box<dyn ContentSource>) -> Object {
        Object {
            id: None,
            kind,
            content_source,
        }
    }

    /// Return the ID of the object, if it is known.
    #[cfg_attr(tarpaulin, skip)]
    pub fn id(&self) -> &Option<Id> {
        // Code coverage doesn't seem to see this line.
        // Not sure why, but I have independently verified it is reached.
        &self.id
    }

    /// Return the kind of the object.
    pub fn kind(&self) -> Kind {
        self.kind
    }

    /// Return the size (in bytes) of the object.
    pub fn len(&self) -> usize {
        self.content_source.len()
    }

    /// Returns true if the object is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a `BufRead` struct which can be used for reading the content.
    pub fn open<'a>(&'a self) -> io::Result<Box<dyn BufRead + 'a>> {
        let f = self.content_source.open()?;
        Ok(Box::new(BufReader::new(f)))
    }

    /// Computes the object's ID from its content, size, and type.
    ///
    /// No-op if an ID has been assigned already.
    ///
    /// This is functionally equivalent to the
    /// [`git hash-object`](https://git-scm.com/docs/git-hash-object) command
    /// without the `-w` option that would write the object to the repo.
    pub fn assign_id(&mut self) -> io::Result<()> {
        if self.id.is_none() {
            let mut hasher = Sha1::new();

            hasher.update(self.kind.to_string());
            hasher.update(b" ");

            let lstr = self.len().to_string();
            hasher.update(lstr);
            hasher.update(b"\0");

            {
                let mut reader = self.open()?;
                let mut buf = [0; 8192];
                let mut n = 1;

                while n > 0 {
                    n = reader.read(&mut buf)?;
                    if n > 0 {
                        hasher.update(&buf[..n]);
                    }
                }
            }

            let final_hash = hasher.finalize();
            let id: &[u8] = final_hash.as_slice();

            // We use unwrap here becuase hasher is guaranteed
            // to return a 20-byte slice.
            self.id = Some(Id::new(id).unwrap());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;
    use std::io::Write;
    use std::process::Command;

    extern crate tempfile;
    use tempfile::TempDir;

    #[test]
    fn empty_vec() {
        let v = vec![];
        let o = Object::new(Kind::Blob, Box::new(v));

        assert_eq!(*o.id(), None);
        assert_eq!(o.kind(), Kind::Blob);
        assert_eq!(o.kind().to_string(), "blob");
        assert_eq!(o.len(), 0);
        assert!(o.is_empty());

        let mut buf = [0; 10];
        let mut f = o.open().unwrap();

        let r = f.read(&mut buf);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), 0);

        let r = f.read(&mut buf);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), 0);
    }

    #[test]
    fn vec_with_content() {
        let v = vec![2, 3, 45, 67];
        let o = Object::new(Kind::Blob, Box::new(v));

        assert_eq!(*o.id(), None);
        assert_eq!(o.kind(), Kind::Blob);
        assert_eq!(o.len(), 4);
        assert!(!o.is_empty());

        let mut buf = [0; 3];
        let mut f = o.open().unwrap();

        let r = f.read(&mut buf);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), 3);
        assert_eq!(buf, [2, 3, 45]);

        let r = f.read(&mut buf);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), 1);
        assert_eq!(buf, [67, 3, 45]);

        let r = f.read(&mut buf);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), 0);
        assert_eq!(buf, [67, 3, 45]);
    }

    #[test]
    fn empty_str() {
        let s = "".to_string();
        let o = Object::new(Kind::Blob, Box::new(s));

        assert_eq!(*o.id(), None);
        assert_eq!(o.kind(), Kind::Blob);
        assert_eq!(o.len(), 0);
        assert!(o.is_empty());

        let mut buf = [0; 10];
        let mut f = o.open().unwrap();

        let r = f.read(&mut buf);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), 0);

        let r = f.read(&mut buf);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), 0);
    }

    #[test]
    fn str_with_content() {
        let s = "ABCD".to_string();
        let o = Object::new(Kind::Blob, Box::new(s));

        assert_eq!(*o.id(), None);
        assert_eq!(o.kind(), Kind::Blob);
        assert_eq!(o.len(), 4);
        assert!(!o.is_empty());

        let mut buf = [0; 3];
        let mut f = o.open().unwrap();

        let r = f.read(&mut buf);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), 3);
        assert_eq!(buf, [65, 66, 67]);

        let r = f.read(&mut buf);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), 1);
        assert_eq!(buf, [68, 66, 67]);

        let r = f.read(&mut buf);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), 0);
        assert_eq!(buf, [68, 66, 67]);
    }

    #[test]
    fn assign_id() {
        // $ echo 'test content' | git hash-object --stdin
        // d670460b4b4aece5915caf5c68d12f560a9fe3e4

        let mut o = Object::new(Kind::Blob, Box::new("test content\n".to_string()));
        o.assign_id().unwrap();

        assert_eq!(
            o.id().as_ref().unwrap().to_string(),
            "d670460b4b4aece5915caf5c68d12f560a9fe3e4"
        );

        // Verify that nothing changes on second assign attempt.

        o.assign_id().unwrap();

        assert_eq!(
            o.id().as_ref().unwrap().to_string(),
            "d670460b4b4aece5915caf5c68d12f560a9fe3e4"
        );
    }

    #[test]
    #[cfg_attr(tarpaulin, skip)]
    fn assign_id_from_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.as_ref().join("example");

        {
            let mut f = File::create(&path).unwrap();
            let s = "foobar".repeat(1000);
            f.write_all(s.as_bytes()).unwrap();
        }

        let path_str = path.to_str().unwrap();
        let output = Command::new("git")
            .args(&["hash-object", path_str])
            .output()
            .unwrap();

        let expected_id = String::from_utf8(output.stdout).unwrap();
        let expected_id = expected_id.trim();

        let fcs = FileContentSource::new(&path).unwrap();
        assert_eq!(fcs.len(), 6000);

        let mut o = Object::new(Kind::Blob, Box::new(fcs));
        o.assign_id().unwrap();

        assert_eq!(o.id().as_ref().unwrap().to_string(), expected_id);
    }
}
