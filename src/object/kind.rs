use std::fmt::{self, Display, Formatter};

/// Describes the fundamental git object type (blob, tree, commit, or tag).
/// We use the word `kind` here to avoid conflict with the Rust reserved word `type`.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Kind {
    Blob,
    Tree,
    Commit,
    Tag,
}

impl Display for Kind {
    #[cfg(not(tarpaulin_include))]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // Code coverage doesn't see the "match self" line.
        // Not sure why, but I have independently verified it is reached.
        match self {
            Kind::Blob => write!(f, "blob"),
            Kind::Tree => write!(f, "tree"),
            Kind::Commit => write!(f, "commit"),
            Kind::Tag => write!(f, "tag"),
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
    }
}
