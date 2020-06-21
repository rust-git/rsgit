//! Represents the git concept of an "object" which is a tuple of
//! object type and binary data identified by the hash of the binary data.

use std::fmt;
use std::fmt::Write;
use std::io::Read;
use std::str::FromStr;

use crate::content_source::ContentSource;

/// An error which can be returned when parsing a git object ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseObjectIdError {
    kind: ParseObjectIdErrorKind,
}

/// Enum to store the various types of errors that can cause parsing an object ID to fail.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ParseObjectIdErrorKind {
    /// Value being parsed is empty.
    Empty,

    /// Contains an invalid digit.
    ///
    /// Among other causes, this variant will be constructed when parsing a string that
    /// contains a letter.
    InvalidDigit,

    /// ID string is too large to store in target integer type.
    Overflow,

    /// ID string is too small to store in target integer type.
    Underflow,

    /// Value was Zero
    ///
    /// This variant will be emitted when the parsing string has a value of zero, which
    /// would be illegal for non-zero types.
    Zero,
}

impl ParseObjectIdError {
    /// Returns the detailed cause of parsing an integer failing.
    pub fn kind(&self) -> ParseObjectIdErrorKind {
        self.kind
    }

    #[doc(hidden)]
    pub fn __description(&self) -> &str {
        match self.kind {
            ParseObjectIdErrorKind::Empty => "cannot parse object ID from empty string",
            ParseObjectIdErrorKind::InvalidDigit => "non-hex digit found in string",
            ParseObjectIdErrorKind::Overflow => "ID too large to fit in target type",
            ParseObjectIdErrorKind::Underflow => "ID too small to fit in target type",
            ParseObjectIdErrorKind::Zero => "ID would be zero",
        }
    }
}

impl fmt::Display for ParseObjectIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.__description().fmt(f)
    }
}

/// An object ID is a string that identifies an object within a repository.
/// It is stored as a 20-byte signature, but can also be represented as 40 hex digits.
#[derive(Clone, PartialEq)]
pub struct ObjectId {
    id: Vec<u8>,
}

impl ObjectId {
    // Returns the special all-null object ID, often used to stand-in for no object.
    // pub fn zero() -> ObjectId {
    //     let id: Vec<u8> = [0; 20].to_vec();
    //     ObjectId{ id }
    // }

    /// Convert a 40-character hex ID to an object ID.
    ///
    /// It is an error if the ID contains anything other than 40 lowercase hex digits.
    pub fn from_hex<T: AsRef<[u8]>>(id: T) -> Result<ObjectId, ParseObjectIdError> {
        let hex = id.as_ref();

        match hex.len() {
            40 => {
                let maybe_id: Result<Vec<u8>, ParseObjectIdError> = hex
                    .chunks(2)
                    .map(|pair| -> Result<u8, ParseObjectIdError> {
                        Ok(digit_value(pair[0])? << 4 | digit_value(pair[1])?)
                    })
                    .collect();

                match maybe_id {
                    Ok(id) => Ok(ObjectId { id }),
                    Err(err) => Err(err),
                }
            }
            0 => Err(ParseObjectIdError {
                kind: ParseObjectIdErrorKind::Empty,
            }),
            n if n < 40 => Err(ParseObjectIdError {
                kind: ParseObjectIdErrorKind::Underflow,
            }),
            _ => Err(ParseObjectIdError {
                kind: ParseObjectIdErrorKind::Overflow,
            }),
        }
    }
}

impl FromStr for ObjectId {
    type Err = ParseObjectIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ObjectId::from_hex(s.as_bytes())
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        static CHARS: &'static [u8] = b"0123456789abcdef";

        for &byte in self.id.iter() {
            f.write_char(CHARS[(byte >> 4) as usize].into())?;
            f.write_char(CHARS[(byte & 0xf) as usize].into())?;
        }

        Ok(())
    }
}

fn digit_value(c: u8) -> Result<u8, ParseObjectIdError> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        _ => Err(ParseObjectIdError {
            kind: ParseObjectIdErrorKind::InvalidDigit,
        }),
    }
}

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
    fn object_id_from_str() {
        let oid = ObjectId::from_hex("3cd9329ac53613a0bfa198ae28f3af957e49573c").unwrap();
        assert_eq!(oid.to_string(), "3cd9329ac53613a0bfa198ae28f3af957e49573c");
    }

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
