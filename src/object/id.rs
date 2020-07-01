//! Represents the git concept of an "object" which is a tuple of
//! object type and binary data identified by the hash of the binary data.

use std::fmt::{self, Formatter, Write};
use std::str::FromStr;

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
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.__description().fmt(f)
    }
}

/// An object ID is a string that identifies an object within a repository.
/// It is stored as a 20-byte signature, but can also be represented as 40 hex digits.
#[derive(Clone, Debug, PartialEq)]
pub struct ObjectId {
    id: Vec<u8>,
}

impl ObjectId {
    // Create a new ID from a 20-byte hex slice.
    pub fn new(id: &[u8]) -> Result<ObjectId, ParseObjectIdError> {
        match id.len() {
            20 => Ok(ObjectId { id: id.to_vec() }),
            0 => Err(ParseObjectIdError {
                kind: ParseObjectIdErrorKind::Empty,
            }),
            n if n < 20 => Err(ParseObjectIdError {
                kind: ParseObjectIdErrorKind::Underflow,
            }),
            _ => Err(ParseObjectIdError {
                kind: ParseObjectIdErrorKind::Overflow,
            }),
        }
    }

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
                let byte_chunks = hex.chunks(2);

                let nybbles = byte_chunks.map(|pair| -> Result<u8, ParseObjectIdError> {
                    Ok(digit_value(pair[0])? << 4 | digit_value(pair[1])?)
                });

                let maybe_id: Result<Vec<u8>, ParseObjectIdError> = nybbles.collect();

                match maybe_id {
                    Ok(id) => {
                        if id.iter().all(|x| *x == 0) {
                            Err(ParseObjectIdError {
                                kind: ParseObjectIdErrorKind::Zero,
                            })
                        } else {
                            Ok(ObjectId { id })
                        }
                    }
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

static CHARS: &[u8] = b"0123456789abcdef";

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

#[cfg(test)]
mod id_tests {
    use super::*;

    extern crate tempfile;

    #[test]
    fn object_id_from_hex() {
        let oid =
            ObjectId::from_hex("3cd9329ac53613a0bfa198ae28f3af957e49573c".as_bytes()).unwrap();
        assert_eq!(oid.to_string(), "3cd9329ac53613a0bfa198ae28f3af957e49573c");
    }

    #[test]
    fn object_id_from_str() {
        let oid = ObjectId::from_str("3cd9329ac53613a0bfa198ae28f3af957e49573c").unwrap();
        assert_eq!(oid.to_string(), "3cd9329ac53613a0bfa198ae28f3af957e49573c");
    }

    #[test]
    fn object_id_from_empty_str() {
        let r = ObjectId::from_hex("");
        assert!(r.is_err());

        if let Err(err) = r {
            assert_eq!(err.kind(), ParseObjectIdErrorKind::Empty);
            assert_eq!(err.to_string(), "cannot parse object ID from empty string");
        }
    }

    #[test]
    fn object_id_from_invalid_str() {
        let r = ObjectId::from_hex("3cD9329ac53613a0bfa198ae28f3af957e49573c");
        assert!(r.is_err());

        if let Err(err) = r {
            assert_eq!(err.kind(), ParseObjectIdErrorKind::InvalidDigit);
            assert_eq!(err.to_string(), "non-hex digit found in string");
        }
    }

    #[test]
    fn object_id_too_long() {
        let r = ObjectId::from_hex("3cd9329ac53613a0bfa198ae28f3af957e49573c4");
        assert!(r.is_err());

        if let Err(err) = r {
            assert_eq!(err.kind(), ParseObjectIdErrorKind::Overflow);
            assert_eq!(err.to_string(), "ID too large to fit in target type");
        }
    }

    #[test]
    fn object_id_too_short() {
        let r = ObjectId::from_hex("3cd9329ac53613a0bfa198ae28f3af957e49573");
        assert!(r.is_err());

        if let Err(err) = r {
            assert_eq!(err.kind(), ParseObjectIdErrorKind::Underflow);
            assert_eq!(err.to_string(), "ID too small to fit in target type");
        }
    }

    #[test]
    fn object_id_zero() {
        let r = ObjectId::from_hex("0000000000000000000000000000000000000000");
        assert!(r.is_err());

        if let Err(err) = r {
            assert_eq!(err.kind(), ParseObjectIdErrorKind::Zero);
            assert_eq!(err.to_string(), "ID would be zero");
        }
    }
}
