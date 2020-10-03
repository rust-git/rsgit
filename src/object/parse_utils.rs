use std::io::{BufRead, Result};

// Read one line from input source if possible.
pub(crate) fn read_line<B: BufRead>(b: &mut B) -> Result<Option<Vec<u8>>> {
    let mut line = Vec::new();

    if b.read_until(10, &mut line)? > 0 {
        if let Some(10) = line.last() {
            line.truncate(line.len() - 1);
        }
        Ok(Some(line))
    } else {
        Ok(None)
    }
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

pub(crate) fn attribution_is_valid(line: &[u8]) -> bool {
    // Note that this parser is intentionally more strict
    // than the one in Attribution::parse.
    if !line.contains(&b'<') {
        return false;
    }
    let (_name, line) = split_once(line, &b'<');

    if !line.contains(&b'>') {
        return false;
    }
    let (_email, line) = split_once(line, &b'>');

    if !line.starts_with(b" ") {
        return false;
    }
    let line = &line[1..];

    if !line.contains(&b' ') {
        return false;
    }
    let (time, tz) = split_once(line, &b' ');

    if time.is_empty() || !time.iter().all(|&c| is_valid_decimal_digit(c)) {
        return false;
    }

    if tz.len() != 5 {
        return false;
    }

    match tz[0] {
        b'+' => (),
        b'-' => (),
        _ => return false,
    }

    if !tz[1..].iter().all(|&c| is_valid_decimal_digit(c)) {
        return false;
    }

    let tzsign = if tz[0] == b'+' { 1 } else { -1 };

    let hh = from_decimal_digit(tz[1]) * 10 + from_decimal_digit(tz[2]);
    let mm = from_decimal_digit(tz[3]) * 10 + from_decimal_digit(tz[3]);
    if mm > 59 {
        return false;
    }

    let tz = tzsign * (hh * 60 + mm);
    tz >= -720 && tz <= 840
}

fn is_valid_decimal_digit(c: u8) -> bool {
    match c {
        b'0'..=b'9' => true,
        _ => false,
    }
}

fn from_decimal_digit(digit: u8) -> i16 {
    // We've already invoked is_valid_decimal_digit
    // on this digit, so no need to recheck.
    (digit as i16) - 48
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
    fn read_line_fn() {
        let mut c = Cursor::new(&b"abc\ndef\n");

        let line = read_line(&mut c).unwrap().unwrap();
        assert_eq!(line.as_slice(), b"abc");

        let line = read_line(&mut c).unwrap().unwrap();
        assert_eq!(line.as_slice(), b"def");

        assert!(read_line(&mut c).unwrap().is_none());

        let mut c = Cursor::new(&b"abc\n");

        let line = read_line(&mut c).unwrap().unwrap();
        assert_eq!(line.as_slice(), b"abc");

        assert!(read_line(&mut c).unwrap().is_none());

        let mut c = Cursor::new(&b"abc");

        let line = read_line(&mut c).unwrap().unwrap();
        assert_eq!(line.as_slice(), b"abc");

        assert!(read_line(&mut c).unwrap().is_none());
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
    fn attribution_is_valid_fn() {
        assert_eq!(
            attribution_is_valid(b"A. U. Thor <author@localhost> 1 +0000"),
            true
        );
        assert_eq!(
            attribution_is_valid(b"A. U. Thor <author@localhost> 1222757360 -0730"),
            true
        );
        assert_eq!(attribution_is_valid(b"<> 0 +0000"), true);

        assert_eq!(attribution_is_valid(b"b <b@c> <b@c> 0 +0000"), false);
        assert_eq!(attribution_is_valid(b"A. U. Thor <foo 1 +0000"), false);
        assert_eq!(attribution_is_valid(b"A. U. Thor foo> 1 +0000"), false);
        assert_eq!(attribution_is_valid(b"1 +0000"), false);
        assert_eq!(attribution_is_valid(b"a <b> +0000"), false);
        assert_eq!(attribution_is_valid(b"a <b> 1 ~0700"), false);
        assert_eq!(attribution_is_valid(b"a <b> 1 +0760"), false);
        assert_eq!(attribution_is_valid(b"a <b> 1 +07x0"), false);
        assert_eq!(attribution_is_valid(b"a <b> 1 +07"), false);
        assert_eq!(attribution_is_valid(b"a <b> 1 +07000"), false);
        assert_eq!(attribution_is_valid(b"a <b> 1 -1300"), false);
        assert_eq!(attribution_is_valid(b"a <b> 1 +1500"), false);
        assert_eq!(attribution_is_valid(b"a <b>"), false);
        assert_eq!(attribution_is_valid(b"a <b> z"), false);
        assert_eq!(attribution_is_valid(b"a <b> 1 z"), false);
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
