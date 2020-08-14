use std::convert::AsRef;
use std::fs::{self, File};
use std::io::{self, BufReader, Error, ErrorKind};
use std::path::{Path, PathBuf};

use super::{ContentSource, ContentSourceOpenResult};

/// Implements `ContentSource` to read content from a file on disk.
pub struct FileContentSource {
    path: PathBuf,
    len: usize,
}

impl FileContentSource {
    /// Create a `FileContentSource` for a file that exists
    /// already on disk.
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<FileContentSource> {
        let m = fs::metadata(&path)?;
        if !m.is_file() {
            return Err(Error::new(ErrorKind::NotFound, "not a single file"));
        }

        Ok(FileContentSource {
            len: m.len() as usize,
            path: path.as_ref().to_owned(),
        })
    }
}

impl ContentSource for FileContentSource {
    fn len(&self) -> usize {
        self.len
    }

    fn open(&self) -> ContentSourceOpenResult {
        let f = File::open(&self.path)?;
        Ok(Box::new(BufReader::new(f)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::{ErrorKind, Write};

    use tempfile::TempDir;

    #[test]
    fn existing_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.as_ref().join("example");

        {
            let mut f = File::create(&path).unwrap();
            f.write_all(b"example").unwrap();
        }

        let fcs = FileContentSource::new(&path).unwrap();
        assert_eq!(fcs.len(), 7);

        let mut r = fcs.open().unwrap();
        let mut buf = [0; 20];
        assert_eq!(r.read(&mut buf).unwrap(), 7);

        assert_eq!(&buf[..7], b"example");
    }

    #[test]
    fn not_existing_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.as_ref().join("example");

        let fcs = FileContentSource::new(&path);
        assert!(fcs.is_err());

        let err = fcs.err().unwrap();
        assert_eq!(err.kind(), ErrorKind::NotFound);
    }

    #[test]
    fn existing_dir() {
        let dir = TempDir::new().unwrap();
        let path = dir.as_ref().join("example");
        fs::create_dir_all(&path).unwrap();

        let fcs = FileContentSource::new(&path);
        assert!(fcs.is_err());

        let err = fcs.err().unwrap();
        assert_eq!(err.kind(), ErrorKind::NotFound);
        assert_eq!(err.to_string(), "not a single file");
    }
}
