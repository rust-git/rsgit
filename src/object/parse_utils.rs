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

pub(crate) fn object_id_is_valid(name: &[u8]) -> bool {
    if name.len() == 40 {
        name.iter().all(|&c| is_valid_hex_digit(c))
    } else {
        false
    }
}

fn is_valid_hex_digit(c: u8) -> bool {
    match c {
        b'0'..=b'9' => true,
        b'a'..=b'f' => true,
        _ => false,
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

    #[test]
    fn object_id_is_valid_fn() {
        assert_eq!(
            object_id_is_valid(b"0123456789012345678901234567890123456789"),
            true
        );
        assert_eq!(
            object_id_is_valid(b"abcdef6789012345678901234567890123456789"),
            true
        );
        assert_eq!(
            object_id_is_valid(b"abcdefg789012345678901234567890123456789"),
            false
        );
        assert_eq!(
            object_id_is_valid(b"Abcdef6789012345678901234567890123456789"),
            false
        );
        assert_eq!(
            object_id_is_valid(b"0123456789/12345678901234567890123456789"),
            false
        );
        assert_eq!(
            object_id_is_valid(b"0123456789:12345678901234567890123456789"),
            false
        );
        assert_eq!(
            object_id_is_valid(b"012345678901234567890123456789012345678"),
            false
        );
        assert_eq!(
            object_id_is_valid(b"01234567890123456789012345678901234567890"),
            false
        );
    }
}
