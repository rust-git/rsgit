//! Represents the git concept of an "object" which is a tuple of
//! object type and binary data identified by the hash of the binary data.

use sha1::{Digest, Sha1};

use crate::path::CheckPlatforms;

mod attribution;
pub use attribution::Attribution;

mod check_commit;
mod check_tag;
mod check_tree;

mod content_source;
pub use content_source::{ContentSource, ContentSourceOpenResult, ContentSourceResult};

mod file_content_source;
pub use file_content_source::FileContentSource;

mod id;
pub use id::{Id, ParseIdError};

mod kind;
pub use kind::Kind;

pub(crate) mod parse_utils;

/// Describes a single object stored (or about to be stored) in a git repository.
///
/// This struct is constructed, modified, and shared as a working description of
/// how to find and describe an object before it gets written to a repository.
pub struct Object {
    id: Id,
    kind: Kind,
    content_source: Box<dyn ContentSource>,
}

impl Object {
    /// Create a new Object.
    ///
    /// Calculates the object's ID.
    #[cfg(not(tarpaulin_include))]
    pub fn new(kind: Kind, content_source: Box<dyn ContentSource>) -> ContentSourceResult<Object> {
        Ok(Object {
            id: assign_id(kind, content_source.as_ref())?,
            kind,
            content_source,
        })
    }

    /// Return the ID of the object.
    #[cfg(not(tarpaulin_include))]
    pub fn id(&self) -> &Id {
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
    pub fn open(&self) -> ContentSourceOpenResult {
        self.content_source.open()
    }

    /// Returns true if the content of the object is valid for the type.
    #[cfg(not(tarpaulin_include))]
    pub fn is_valid(&self) -> ContentSourceResult<bool> {
        // The match line is seen as executable but not covered.
        // Does not compute.
        match self.kind {
            Kind::Blob => Ok(true),
            Kind::Commit => check_commit::commit_is_valid(self.content_source.as_ref()),
            Kind::Tag => check_tag::tag_is_valid(self.content_source.as_ref()),
            Kind::Tree => check_tree::tree_is_valid(self.content_source.as_ref()),
        }
    }

    /// Returns true if the content of the object is valid for the type
    /// and the given platform's file system(s).
    #[cfg(not(tarpaulin_include))]
    pub fn is_valid_with_platform_checks(
        &self,
        platforms: &CheckPlatforms,
    ) -> ContentSourceResult<bool> {
        // The match and platforms line are seen as executable but not covered.
        // Does not compute.
        match self.kind {
            Kind::Blob => Ok(true),
            Kind::Commit => check_commit::commit_is_valid(self.content_source.as_ref()),
            Kind::Tag => check_tag::tag_is_valid(self.content_source.as_ref()),
            Kind::Tree => check_tree::tree_is_valid_with_platform_checks(
                self.content_source.as_ref(),
                platforms,
            ),
        }
    }
}

fn assign_id(kind: Kind, content_source: &dyn ContentSource) -> ContentSourceResult<Id> {
    let mut hasher = Sha1::new();

    hasher.update(kind.to_string());
    hasher.update(b" ");

    let lstr = content_source.len().to_string();
    hasher.update(lstr);
    hasher.update(b"\0");

    {
        let mut reader = content_source.open()?;
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
    Ok(Id::new(id).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;
    use std::io::Write;
    use std::process::Command;

    use tempfile::TempDir;

    #[test]
    fn empty_vec() {
        let v = vec![];
        let o = Object::new(Kind::Blob, Box::new(v)).unwrap();

        assert_eq!(
            o.id().to_string(),
            "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
        );
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
        let o = Object::new(Kind::Blob, Box::new(v)).unwrap();

        assert_eq!(
            o.id().to_string(),
            "87cffd12aa440e20847f516da27af986eacda0b9"
        );
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
        let o = Object::new(Kind::Blob, Box::new(s)).unwrap();

        assert_eq!(
            o.id().to_string(),
            "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
        );
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
        let o = Object::new(Kind::Blob, Box::new(s)).unwrap();

        assert_eq!(
            o.id().to_string(),
            "a6bddc4a144046eddf2296a9a4c23d8fae600b15"
        );
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
    fn id_matches_git_hash_object() {
        // $ echo 'test content' | git hash-object --stdin
        // d670460b4b4aece5915caf5c68d12f560a9fe3e4

        let o = Object::new(Kind::Blob, Box::new("test content\n".to_string())).unwrap();
        assert_eq!(
            o.id().to_string(),
            "d670460b4b4aece5915caf5c68d12f560a9fe3e4"
        );
    }

    #[test]
    #[cfg(not(tarpaulin_include))]
    fn assign_id_from_file_matches_git_hash_object() {
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

        let o = Object::new(Kind::Blob, Box::new(fcs)).unwrap();
        assert_eq!(o.id().to_string(), expected_id);
    }

    #[test]
    fn check_blob_valid() {
        let cs = "no such thing as an invalid blob".to_string();

        let o = Object::new(Kind::Blob, Box::new(cs)).unwrap();
        assert_eq!(o.is_valid().unwrap(), true);
    }

    #[test]
    fn check_commit_valid_no_parent() {
        let cs = "tree be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  author A. U. Thor <author@localhost> 1 +0000\n\
                  committer A. U. Thor <author@localhost> 1 +0000\n"
            .to_string();

        let o = Object::new(Kind::Commit, Box::new(cs)).unwrap();
        assert_eq!(o.is_valid().unwrap(), true);
    }

    #[test]
    fn check_commit_valid_blank_author() {
        let cs = "tree be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  author <> 0 +0000\n\
                committer <> 0 +0000\n"
            .to_string();

        let o = Object::new(Kind::Commit, Box::new(cs)).unwrap();
        assert_eq!(o.is_valid().unwrap(), true);
    }

    #[test]
    fn check_commit_invalid_corrupt_attribution() {
        let cs = "tree be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  author <> 0 +0000\n\
                  committer b <b@c> <b@c> 0 +0000\n"
            .to_string();

        let o = Object::new(Kind::Commit, Box::new(cs)).unwrap();
        assert_eq!(o.is_valid().unwrap(), false);
    }

    #[test]
    fn check_tag_valid() {
        let cs = "object be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  type commit\n\
                  tag test-tag\n\
                  tagger A. U. Thor <tagger@localhost> 1 +0000\n"
            .to_string();

        let o = Object::new(Kind::Tag, Box::new(cs)).unwrap();
        assert_eq!(o.is_valid().unwrap(), true);
    }

    #[test]
    fn check_tag_invalid_object() {
        let cs = "object\tbe9bfa841874ccc9f2ef7c48d0c76226f89b7189\n".to_string();

        let o = Object::new(Kind::Tag, Box::new(cs)).unwrap();
        assert_eq!(o.is_valid().unwrap(), false);
    }

    const PLACEHOLDER_OBJECT_ID: &str =
        "\0\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10\x11\x12\x13";

    fn entry(mode_name_str: &str) -> String {
        entry_with_object_id(mode_name_str, PLACEHOLDER_OBJECT_ID)
    }

    fn entry_with_object_id(mode_name_str: &str, object_id: &str) -> String {
        let mut r = String::new();
        r.push_str(mode_name_str);
        r.push('\0');

        assert_eq!(object_id.len(), 20);
        r.push_str(object_id);
        r
    }

    #[test]
    fn check_tree_valid_tree_one_entry() {
        let cs = entry("100644 regular-file");

        let o = Object::new(Kind::Tree, Box::new(cs)).unwrap();
        assert_eq!(o.is_valid().unwrap(), true);
    }

    #[test]
    fn check_tree_invalid_null_object_id() {
        let cs = entry_with_object_id(
            "100644 regular-file",
            "\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        );

        let o = Object::new(Kind::Tree, Box::new(cs)).unwrap();
        assert_eq!(o.is_valid().unwrap(), false);
    }

    #[test]
    fn platform_check_blob_valid() {
        let cs = "no such thing as an invalid blob".to_string();

        let o = Object::new(Kind::Blob, Box::new(cs)).unwrap();
        assert_eq!(
            o.is_valid_with_platform_checks(&CheckPlatforms {
                windows: false,
                mac: true
            })
            .unwrap(),
            true
        );
    }

    #[test]
    fn platform_check_commit_valid_no_parent() {
        let cs = "tree be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  author A. U. Thor <author@localhost> 1 +0000\n\
                  committer A. U. Thor <author@localhost> 1 +0000\n"
            .to_string();

        let o = Object::new(Kind::Commit, Box::new(cs)).unwrap();
        assert_eq!(
            o.is_valid_with_platform_checks(&CheckPlatforms {
                windows: false,
                mac: true
            })
            .unwrap(),
            true
        );
    }

    #[test]
    fn platform_check_commit_valid_blank_author() {
        let cs = "tree be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  author <> 0 +0000\n\
                committer <> 0 +0000\n"
            .to_string();

        let o = Object::new(Kind::Commit, Box::new(cs)).unwrap();
        assert_eq!(
            o.is_valid_with_platform_checks(&CheckPlatforms {
                windows: false,
                mac: true
            })
            .unwrap(),
            true
        );
    }

    #[test]
    fn platform_check_commit_invalid_corrupt_attribution() {
        let cs = "tree be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  author <> 0 +0000\n\
                  committer b <b@c> <b@c> 0 +0000\n"
            .to_string();

        let o = Object::new(Kind::Commit, Box::new(cs)).unwrap();
        assert_eq!(
            o.is_valid_with_platform_checks(&CheckPlatforms {
                windows: false,
                mac: true
            })
            .unwrap(),
            false
        );
    }

    #[test]
    fn platform_check_tag_valid() {
        let cs = "object be9bfa841874ccc9f2ef7c48d0c76226f89b7189\n\
                  type commit\n\
                  tag test-tag\n\
                  tagger A. U. Thor <tagger@localhost> 1 +0000\n"
            .to_string();

        let o = Object::new(Kind::Tag, Box::new(cs)).unwrap();
        assert_eq!(
            o.is_valid_with_platform_checks(&CheckPlatforms {
                windows: false,
                mac: true
            })
            .unwrap(),
            true
        );
    }

    #[test]
    fn platform_check_tag_invalid_object() {
        let cs = "object\tbe9bfa841874ccc9f2ef7c48d0c76226f89b7189\n".to_string();

        let o = Object::new(Kind::Tag, Box::new(cs)).unwrap();
        assert_eq!(
            o.is_valid_with_platform_checks(&CheckPlatforms {
                windows: false,
                mac: true
            })
            .unwrap(),
            false
        );
    }

    #[test]
    fn platform_check_tree_valid_tree_one_entry() {
        let cs = entry("100644 regular-file");

        let o = Object::new(Kind::Tree, Box::new(cs)).unwrap();
        assert_eq!(
            o.is_valid_with_platform_checks(&CheckPlatforms {
                windows: false,
                mac: false
            })
            .unwrap(),
            true
        );
    }

    #[test]
    fn platform_check_tree_invalid_null_object_id() {
        let cs = entry_with_object_id(
            "100644 regular-file",
            "\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        );

        let o = Object::new(Kind::Tree, Box::new(cs)).unwrap();
        assert_eq!(
            o.is_valid_with_platform_checks(&CheckPlatforms {
                windows: false,
                mac: false
            })
            .unwrap(),
            false
        );
    }

    #[test]
    fn platform_check_tree_windows_dot_at_end_of_name() {
        let cs = entry(&"100644 test.".to_string());

        let o = Object::new(Kind::Tree, Box::new(cs)).unwrap();
        assert_eq!(
            o.is_valid_with_platform_checks(&CheckPlatforms {
                windows: true,
                mac: false
            })
            .unwrap(),
            false
        );
    }
}
