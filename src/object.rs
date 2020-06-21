//! Represents the git concept of an "object" which is a tuple of
//! object type and binary data identified by the hash of the binary data.

use std::io::Read;

use crate::content_source::ContentSource;

/// Describes the fundamental git object type (blob, tree, commit, or tag).
/// We use the word `kind` here to avoid conflict with the Rust reserved word `type`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ObjectKind {
    Blob,
    Tree,
    Commit,
    Tag,
}

/// Describes a single object stored (or about to be stored) in a git repository.
///
/// This struct is constructed, modified, and shared as a working description of
/// how to find and describe an object before it gets written to a repository.
pub struct Object {
    kind: ObjectKind,
    content_source: Box<dyn ContentSource>,
}

impl Object {
    /// Create a new Object.
    pub fn new(kind: ObjectKind, content_source: Box<dyn ContentSource>) -> Object {
        Object {
            kind,
            content_source,
        }
    }

    /// Return the kind of the object.
    pub fn kind(&self) -> ObjectKind {
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

    /// Returns a `Read` struct which can be used for reading the content.
    pub fn open<'a>(&'a self) -> Box<dyn Read + 'a> {
        self.content_source.open()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_vec() {
        let v = vec![];
        let o = Object::new(ObjectKind::Blob, Box::new(v));

        assert_eq!(o.kind(), ObjectKind::Blob);
        assert_eq!(o.len(), 0);
        assert!(o.is_empty());

        let mut buf = [0; 10];
        let mut f = o.open();

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
        let o = Object::new(ObjectKind::Blob, Box::new(v));

        assert_eq!(o.kind(), ObjectKind::Blob);
        assert_eq!(o.len(), 4);
        assert!(!o.is_empty());

        let mut buf = [0; 3];
        let mut f = o.open();

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
        let o = Object::new(ObjectKind::Blob, Box::new(s));

        assert_eq!(o.kind(), ObjectKind::Blob);
        assert_eq!(o.len(), 0);
        assert!(o.is_empty());

        let mut buf = [0; 10];
        let mut f = o.open();

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
        let o = Object::new(ObjectKind::Blob, Box::new(s));

        assert_eq!(o.kind(), ObjectKind::Blob);
        assert_eq!(o.len(), 4);
        assert!(!o.is_empty());

        let mut buf = [0; 3];
        let mut f = o.open();

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
}
