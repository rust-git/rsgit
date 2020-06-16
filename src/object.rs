//! Represents the git concept of an "object" which is a tuple of
//! object type and binary data identified by the hash of the binary data.

pub enum ObjectType {
    Blob,
    Tree,
    Commit,
    Tag,
}
