use std::io::{BufRead, Result};

pub(crate) fn read_line(b: &mut dyn BufRead, line: &mut Vec<u8>) -> Result<usize> {
    line.clear();

    b.read_until(10, line)?;

    if let Some(last) = line.last() {
        if last == &10 {
            line.truncate(line.len() - 1);
        }
    }

    Ok(line.len())
}

pub(crate) fn header<'a>(line: &'a [u8], name: &[u8]) -> Option<&'a [u8]> {
    if line.contains(&b' ') {
        let (maybe_name, value) = split_once(line, &b' ');
        if maybe_name == name {
            Some(value)
        } else {
            None
        }
    } else {
        None
    }
}

pub(crate) fn split_once<'a>(s: &'a [u8], c: &u8) -> (&'a [u8], &'a [u8]) {
    match s.iter().position(|b| b == c) {
        Some(n) => (&s[0..n], &s[n + 1..]),
        None => (s, &[]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Cursor;

    #[test]
    fn read_line() {
        let mut c = Cursor::new(&b"abc\ndef\n");
        let mut line: Vec<u8> = Vec::new();

        assert_eq!(super::read_line(&mut c, &mut line).unwrap(), 3);
        assert_eq!(line.as_slice(), b"abc");

        let mut c = Cursor::new(&b"abc\n");
        let mut line: Vec<u8> = Vec::new();

        assert_eq!(super::read_line(&mut c, &mut line).unwrap(), 3);
        assert_eq!(line.as_slice(), b"abc");

        let mut c = Cursor::new(&b"abc");
        let mut line: Vec<u8> = Vec::new();

        assert_eq!(super::read_line(&mut c, &mut line).unwrap(), 3);
        assert_eq!(line.as_slice(), b"abc");
    }

    #[test]
    fn header_fn() {
        assert_eq!(header(b"tagger abc", b"tagger").unwrap(), b"abc");
        assert_eq!(header(b"tagger ", b"tagger").unwrap(), b"");

        assert_eq!(header(b"taggex abc", b"tagger"), None);
        assert_eq!(header(b"tagger", b"tagger"), None);
        assert_eq!(header(b"taggerx abc", b"tagger"), None);
    }
}
