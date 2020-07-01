//! Represents the git concept of an "object" which is a tuple of
//! object type and binary data identified by the hash of the binary data.

use std::fmt::{self, Display, Formatter};

/// Describes the fundamental git object type (blob, tree, commit, or tag).
/// We use the word `kind` here to avoid conflict with the Rust reserved word `type`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ObjectKind {
    Blob,
    Tree,
    Commit,
    Tag,
}

impl Display for ObjectKind {
    #[cfg_attr(tarpaulin, skip)]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // Code coverage doesn't see the "match self" line.
        // Not sure why, but I have independently verified it is reached.
        match self {
            ObjectKind::Blob => write!(f, "blob"),
            ObjectKind::Tree => write!(f, "tree"),
            ObjectKind::Commit => write!(f, "commit"),
            ObjectKind::Tag => write!(f, "tag"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_string() {
        let k = ObjectKind::Blob;
        assert_eq!(k.to_string(), "blob");

        let k = ObjectKind::Commit;
        assert_eq!(k.to_string(), "commit");

        let k = ObjectKind::Tree;
        assert_eq!(k.to_string(), "tree");

        let k = ObjectKind::Tag;
        assert_eq!(k.to_string(), "tag");
    }
}
