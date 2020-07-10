use std::fmt::{self, Write};
use std::str::FromStr;

extern crate thiserror;
use thiserror::Error;

/// An error which can be returned when parsing a git object ID.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ParseIdError {
    /// Value being parsed is empty.
    #[error("cannot parse object ID from empty string")]
    Empty,

    /// Contains an invalid digit.
    ///
    /// Among other causes, this variant will be constructed when parsing a string that
    /// contains a letter.
    #[error("value contains invalid digit `{0}`")]
    InvalidDigit(char),

    /// ID string is too large to store in target integer type.
    #[error("value is more than 40 digits long")]
    Overflow,

    /// ID string is too small to store in target integer type.
    #[error("value is less than 40 digits long")]
    Underflow,

    /// Value was zero.
    #[error("ID would be zero")]
    Zero,
}

/// An object ID is a string that identifies an object within a repository.
/// It is stored as a 20-byte signature, but can also be represented as 40 hex digits.
#[derive(Clone, Debug, PartialEq)]
pub struct Id {
    id: Vec<u8>,
}

impl Id {
    /// Create a new ID from a 20-byte hex slice.
    ///
    /// It is an error if the slice contains anything other than 20 bytes.
    pub fn new(id: &[u8]) -> Result<Id, ParseIdError> {
        match id.len() {
            20 => Ok(Id { id: id.to_vec() }),
            0 => Err(ParseIdError::Empty),
            n if n < 20 => Err(ParseIdError::Underflow),
            _ => Err(ParseIdError::Overflow),
        }
    }

    // Returns the special all-null object ID, often used to stand-in for no object.
    // pub fn zero() -> Id {
    //     let id: Vec<u8> = [0; 20].to_vec();
    //     Id{ id }
    // }

    /// Convert a 40-character hex ID to an object ID.
    ///
    /// It is an error if the ID contains anything other than 40 lowercase hex digits.
    pub fn from_hex<T: AsRef<[u8]>>(id: T) -> Result<Id, ParseIdError> {
        let hex = id.as_ref();

        match hex.len() {
            40 => {
                let byte_chunks = hex.chunks(2);

                let nybbles = byte_chunks.map(|pair| -> Result<u8, ParseIdError> {
                    Ok(digit_value(pair[0])? << 4 | digit_value(pair[1])?)
                });

                let maybe_id: Result<Vec<u8>, ParseIdError> = nybbles.collect();

                match maybe_id {
                    Ok(id) => {
                        if id.iter().all(|x| *x == 0) {
                            Err(ParseIdError::Zero)
                        } else {
                            Ok(Id { id })
                        }
                    }
                    Err(err) => Err(err),
                }
            }
            0 => Err(ParseIdError::Empty),
            n if n < 40 => Err(ParseIdError::Underflow),
            _ => Err(ParseIdError::Overflow),
        }
    }
}

impl FromStr for Id {
    type Err = ParseIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Id::from_hex(s.as_bytes())
    }
}

static CHARS: &[u8] = b"0123456789abcdef";

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &byte in self.id.iter() {
            f.write_char(CHARS[(byte >> 4) as usize].into())?;
            f.write_char(CHARS[(byte & 0xf) as usize].into())?;
        }

        Ok(())
    }
}

fn digit_value(c: u8) -> Result<u8, ParseIdError> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        _ => Err(ParseIdError::InvalidDigit(c as char)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    extern crate tempfile;

    #[test]
    fn new() {
        let b = [
            0x3c, 0xd9, 0x32, 0x9a, 0xc5, 0x36, 0x13, 0xa0, 0xbf, 0xa1, 0x98, 0xae, 0x28, 0xf3,
            0xaf, 0x95, 0x7e, 0x49, 0x57, 0x3c,
        ];

        let oid = Id::new(&b).unwrap();
        assert_eq!(oid.to_string(), "3cd9329ac53613a0bfa198ae28f3af957e49573c");

        let b: [u8; 0] = [];
        assert_eq!(Id::new(&b).unwrap_err(), ParseIdError::Empty);

        let b: [u8; 19] = [
            0x3c, 0xd9, 0x32, 0x9a, 0xc5, 0x36, 0x13, 0xa0, 0xbf, 0xa1, 0x98, 0xae, 0x28, 0xf3,
            0xaf, 0x95, 0x7e, 0x49, 0x57,
        ];
        assert_eq!(Id::new(&b).unwrap_err(), ParseIdError::Underflow);

        let b: [u8; 21] = [
            0x3c, 0xd9, 0x32, 0x9a, 0xc5, 0x36, 0x13, 0xa0, 0xbf, 0xa1, 0x98, 0xae, 0x28, 0xf3,
            0xaf, 0x95, 0x7e, 0x49, 0x57, 0x3c, 0x3c,
        ];
        assert_eq!(Id::new(&b).unwrap_err(), ParseIdError::Overflow);
    }

    #[test]
    fn from_hex() {
        let oid = Id::from_hex("3cd9329ac53613a0bfa198ae28f3af957e49573c".as_bytes()).unwrap();
        assert_eq!(oid.to_string(), "3cd9329ac53613a0bfa198ae28f3af957e49573c");
    }

    #[test]
    fn from_str() {
        let oid = Id::from_str("3cd9329ac53613a0bfa198ae28f3af957e49573c").unwrap();
        assert_eq!(oid.to_string(), "3cd9329ac53613a0bfa198ae28f3af957e49573c");
    }

    #[test]
    fn from_empty_str() {
        let r = Id::from_hex("");
        assert!(r.is_err());

        if let Err(err) = r {
            assert_eq!(err, ParseIdError::Empty);
            assert_eq!(err.to_string(), "cannot parse object ID from empty string");
        }
    }

    #[test]
    fn from_invalid_str() {
        let r = Id::from_hex("3cD9329ac53613a0bfa198ae28f3af957e49573c");
        assert!(r.is_err());

        if let Err(err) = r {
            assert_eq!(err, ParseIdError::InvalidDigit('D'));
            assert_eq!(err.to_string(), "value contains invalid digit `D`");
        }
    }

    #[test]
    fn from_hex_too_long() {
        let r = Id::from_hex("3cd9329ac53613a0bfa198ae28f3af957e49573c4");
        assert!(r.is_err());

        if let Err(err) = r {
            assert_eq!(err, ParseIdError::Overflow);
            assert_eq!(err.to_string(), "value is more than 40 digits long");
        }
    }

    #[test]
    fn from_hex_too_short() {
        let r = Id::from_hex("3cd9329ac53613a0bfa198ae28f3af957e49573");
        assert!(r.is_err());

        if let Err(err) = r {
            assert_eq!(err, ParseIdError::Underflow);
            assert_eq!(err.to_string(), "value is less than 40 digits long");
        }
    }

    #[test]
    fn error_zero() {
        let r = Id::from_hex("0000000000000000000000000000000000000000");
        assert!(r.is_err());

        if let Err(err) = r {
            assert_eq!(err, ParseIdError::Zero);
            assert_eq!(err.to_string(), "ID would be zero");
        }
    }
}
