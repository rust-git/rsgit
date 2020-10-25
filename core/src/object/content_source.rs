use std::{
    io::{BufRead, Cursor},
    vec::Vec,
};

/// Result type for operations which depend on [`ContentSource.open()`].
/// Since [`ContentSource`] may wrap arbitrary sources,
/// it could return any arbitrary error type.
///
/// [`ContentSource`]: trait.ContentSource.html
/// [`ContentSource.open()`]: trait.ContentSource.html#tymethod.open
pub type ContentSourceResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Result type for [`ContentSource.open()`] call.
/// Since [`ContentSource`] may wrap arbitrary sources,
/// it could return any arbitrary error type.
///
/// [`ContentSource`]: trait.ContentSource.html
/// [`ContentSource.open()`]: trait.ContentSource.html#tymethod.open
pub type ContentSourceOpenResult<'a> = ContentSourceResult<Box<dyn BufRead + 'a>>;

/// Trait used for reading git object content from various sources.
pub trait ContentSource {
    // TO DO: Rework this as async at some point? I'm not ready for that yet.
    // https://github.com/rust-git/rsgit/issues/18

    /// Returns the length (in bytes) of the content.
    fn len(&self) -> usize;

    /// Returns true if the content is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a [`BufRead`] struct which can be used for reading the content.
    ///
    /// [`BufRead`]: https://doc.rust-lang.org/nightly/std/io/trait.BufRead.html
    fn open(&self) -> ContentSourceOpenResult;
}

impl ContentSource for Vec<u8> {
    fn len(&self) -> usize {
        self.len()
    }

    fn open(&self) -> ContentSourceOpenResult {
        Ok(Box::new(Cursor::new(self)))
    }
}

impl ContentSource for String {
    fn len(&self) -> usize {
        self.len()
    }

    fn open(&self) -> ContentSourceOpenResult {
        Ok(Box::new(Cursor::new(self.as_bytes())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_vec() {
        let v = vec![];

        let l = ContentSource::len(&v);
        assert_eq!(l, 0);

        assert!(ContentSource::is_empty(&v));

        let mut buf = [0; 10];
        let mut f = v.open().unwrap();

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

        let l = ContentSource::len(&v);
        assert_eq!(l, 4);

        assert!(!ContentSource::is_empty(&v));

        let mut buf = [0; 3];
        let mut f = v.open().unwrap();

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

        let l = ContentSource::len(&s);
        assert_eq!(l, 0);

        assert!(ContentSource::is_empty(&s));

        let mut buf = [0; 10];
        let mut f = s.open().unwrap();

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

        let l = ContentSource::len(&s);
        assert_eq!(l, 4);

        assert!(!ContentSource::is_empty(&s));

        let mut buf = [0; 3];
        let mut f = s.open().unwrap();

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
