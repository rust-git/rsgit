use std::io::{self, Cursor, Error, ErrorKind, Read};

use super::{ContentSource, ContentSourceOpenResult};

/// Implements [`ContentSource`] to read content from
/// an arbitrary [`Read`] struct (often `stdin`).
///
/// This is not intended for heavy-weight production use.
/// Since rsgit isn't optimized to perform single-pass reading
/// and verification of content source, it buffers the [`Read`]
/// content so as to allow multiple reads.
///
/// For safety, it will fail if the source content exceeds an
/// arbitrary limit (currently 20MB).
///
/// [`ContentSource`]: trait.ContentSource.html
/// [`Read`]: https://doc.rust-lang.org/std/io/trait.Read.html
pub struct ReadContentSource {
    content: Vec<u8>,
}

const MAX_SIZE: usize = 20 * 1024 * 1024;

impl ReadContentSource {
    /// Create a `ReadContentSource` for an arbitrary [`Read`] struct.
    ///
    /// [`Read`]: https://doc.rust-lang.org/std/io/trait.Read.html
    pub fn new<R: Read>(r: R) -> io::Result<ReadContentSource> {
        let mut content: Vec<u8> = Vec::new();

        let mut take = r.take(MAX_SIZE as u64 + 1);
        let size = take.read_to_end(&mut content)?;
        if size > MAX_SIZE {
            Err(Error::new(
                ErrorKind::InvalidData,
                format!("read beyond {} byte limit", MAX_SIZE),
            ))
        } else {
            Ok(ReadContentSource { content })
        }
    }
}

impl ContentSource for ReadContentSource {
    fn len(&self) -> usize {
        self.content.len()
    }

    fn open(&self) -> ContentSourceOpenResult {
        Ok(Box::new(Cursor::new(&self.content)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::{Cursor, Result};

    #[test]
    fn happy_path() {
        let tc: Vec<u8> = b"example".to_vec();
        let rcs = ReadContentSource::new(Cursor::new(tc)).unwrap();

        assert_eq!(rcs.len(), 7);

        let mut r = rcs.open().unwrap();
        let mut buf = [0; 20];
        assert_eq!(r.read(&mut buf).unwrap(), 7);

        assert_eq!(&buf[..7], b"example");

        let mut r = rcs.open().unwrap();
        let mut buf = [0; 20];
        assert_eq!(r.read(&mut buf).unwrap(), 7);

        assert_eq!(&buf[..7], b"example");
    }

    struct InfiniteRead {}

    impl Read for InfiniteRead {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            // Yes, this is evil!
            Ok(buf.len())
        }
    }

    #[test]
    fn infinite_read_stream() {
        let evil = InfiniteRead {};
        let res = ReadContentSource::new(evil);
        assert!(res.is_err());
    }
}
