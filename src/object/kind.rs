use std::fmt::{self, Display, Formatter};

/// Describes the fundamental git object type (blob, tree, commit, or tag).
///
/// We use the word `kind` here to avoid conflict with the Rust reserved word `type`.
///
/// Though not widely supported or used, you can also store objects of other
/// made-up types by using the `--literally` flag, so this type supports storing
/// other values, albeit less efficiently than it does the built-in types.
#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    Blob,
    Tree,
    Commit,
    Tag,
    Other(Vec<u8>),
}

impl Display for Kind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Kind::Blob => write!(f, "blob"),
            Kind::Tree => write!(f, "tree"),
            Kind::Commit => write!(f, "commit"),
            Kind::Tag => write!(f, "tag"),
            Kind::Other(name) => write!(f, "{}", String::from_utf8_lossy(name)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_string() {
        let k = Kind::Blob;
        assert_eq!(k.to_string(), "blob");

        let k = Kind::Commit;
        assert_eq!(k.to_string(), "commit");

        let k = Kind::Tree;
        assert_eq!(k.to_string(), "tree");

        let k = Kind::Tag;
        assert_eq!(k.to_string(), "tag");

        let k = Kind::Other(b"arbitrary".to_vec());
        assert_eq!(k.to_string(), "arbitrary");
    }
}
