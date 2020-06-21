use std::io::Error;
use std::io::Read;
use std::result::Result;
use std::vec::Vec;

/// Trait used for reading git object content from various sources.
pub trait ContentSource<'a> {
    // TO DO: Rework this as async at some point? I'm not ready for that yet.
    // https://github.com/rust-git/rsgit/issues/18

    /// Returns the length (in bytes) of the content.
    fn len(&self) -> usize;

    // Returns true if the content is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    // Returns a `Read` struct which can be used for reading the content.
    fn open(&'a self) -> Box<dyn Read + 'a>;
}

struct ByteSliceReader<'a> {
    v: &'a [u8],
    len: usize,
    offset: usize,
}

impl<'a> ByteSliceReader<'a> {
    fn new<'x>(v: &'x [u8]) -> ByteSliceReader<'x> {
      ByteSliceReader {
            v,
            len: v.len(),
            offset: 0,
        }
    }
}

impl Read for ByteSliceReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let len = self.len;
        if self.offset < len {
            let copy_len = (len - self.offset).min(buf.len());
            let copy_from = &(self.v)[self.offset..(self.offset + copy_len)];
            buf[0..copy_len].copy_from_slice(&copy_from);
            self.offset += copy_len;
            Ok(copy_len)
        } else {
            Ok(0)
        }
    }
}

impl<'a> ContentSource<'a> for Vec<u8> {
    fn len(&self) -> usize {
        self.len()
    }

    fn open(&'a self) -> Box<dyn Read + 'a> {
        Box::new(ByteSliceReader::new(self))
    }
}

impl<'a> ContentSource<'a> for str {
    fn len(&self) -> usize {
        self.len()
    }

    fn open(&'a self) -> Box<dyn Read + 'a> {
        Box::new(ByteSliceReader::new(self.as_bytes()))
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
        let mut f = v.open();

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
        let mut f = v.open();

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
    fn empty_sr() {
        let s = "";

        let l = ContentSource::len(s);
        assert_eq!(l, 0);

        assert!(ContentSource::is_empty(s));

        let mut buf = [0; 10];
        let mut f = s.open();

        let r = f.read(&mut buf);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), 0);

        let r = f.read(&mut buf);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), 0);
    }

    #[test]
    fn str_with_content() {
        let s = "ABCD";

        let l = ContentSource::len(s);
        assert_eq!(l, 4);

        assert!(!ContentSource::is_empty(s));

        let mut buf = [0; 3];
        let mut f = s.open();

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
