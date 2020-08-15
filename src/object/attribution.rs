use std::fmt;
use std::str::{self, FromStr};
use std::string::String;

use super::parse_utils::split_once;

/// An `Attribution` combines a person's identity (name and e-mail address)
/// with the timestamp for a particular action.
///
/// Attributions are typically associated with commits or tags in git.
///
/// The `timestamp` value is in milliseconds relative to the Unix era.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attribution {
    name: String,
    email: String,
    timestamp: i64,
    tz_offset: i16,
}

impl Attribution {
    /// Creates a new attribution.
    pub fn new(name: &str, email: &str, timestamp: i64, tz_offset: i16) -> Attribution {
        if tz_offset < -720 || tz_offset > 840 {
            panic!("Illegal time zone offset: {}", tz_offset);
        }

        Attribution {
            name: name.to_string(),
            email: email.to_string(),
            timestamp,
            tz_offset,
        }
    }

    /// Parse a name line (e.g. author, committer, tagger) into an `Attribution` struct.
    /// Returns `None` if unable to parse the line properly.
    pub fn parse(line: &[u8]) -> Option<Attribution> {
        let line = drop_last_newline(line);
        let (name, line) = split_once(line, &b'<');
        let name = drop_last_space(name);
        let name = match str::from_utf8(name) {
            Ok(name_str) => name_str.to_string(),
            _ => return None,
        };

        if !line.contains(&b'>') {
            return None;
        }

        let (email, line) = split_once(line, &b'>');
        let email = match str::from_utf8(email) {
            Ok(email_str) => email_str.to_string(),
            _ => return None,
        };

        let line = drop_last_space(line);
        let (tz_offset, line) = last_word(line);
        let tz_offset = match tz_from_str(tz_offset.as_str()) {
            Some(t) => t,
            _ => 0,
        };

        let (timestamp, _line) = last_word(line);
        let timestamp = match i64::from_str(timestamp.as_str()) {
            Ok(t) => t,
            _ => 0,
        };

        Some(Attribution {
            name,
            email,
            timestamp,
            tz_offset,
        })
    }

    /// Returns the person's human-readable name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns a sanitized version of the person's human-readable name.
    pub fn sanitized_name(&self) -> String {
        sanitize(&self.name)
    }

    /// Returns the person's email address.
    pub fn email(&self) -> &str {
        &self.email
    }

    /// Returns a sanitized version of the person's email address.
    pub fn sanitized_email(&self) -> String {
        sanitize(&self.email)
    }

    /// Returns the timestamp.
    pub fn timestamp(&self) -> i64 {
        self.timestamp
    }

    /// Returns the timezone offset (minutes relative to GMT).
    pub fn tz_offset(&self) -> i16 {
        self.tz_offset
    }

    /// Returns the timezone formatted in human readable offset from GMT.
    pub fn format_tz(&self) -> String {
        let sign = if self.tz_offset < 0 { "-" } else { "+" };

        let offset = self.tz_offset.abs();
        let hours = offset / 60;
        let min = offset % 60;

        format!("{}{:02}{:02}", sign, hours, min)
    }
}

fn drop_last_newline(s: &[u8]) -> &[u8] {
    if s.last() == Some(&10) {
        &s[0..s.len() - 1]
    } else {
        s
    }
}

fn drop_last_space(s: &[u8]) -> &[u8] {
    if s.last() == Some(&b' ') {
        &s[0..s.len() - 1]
    } else {
        s
    }
}

fn last_word(s: &[u8]) -> (String, &[u8]) {
    let s = match s.iter().position(|b| b != &b' ') {
        Some(n) => &s[n..],
        None => s,
    };

    let (word, line) = rsplit_once(s, &b' ');
    let word = match str::from_utf8(word) {
        Ok(word_str) => word_str.to_string(),
        _ => "".to_string(),
    };

    (word, line)
}

fn rsplit_once<'a>(s: &'a [u8], c: &u8) -> (&'a [u8], &'a [u8]) {
    match s.iter().rev().position(|b| b == c) {
        Some(n) => (&s[s.len() - n..], &s[0..s.len() - n - 1]),
        None => (s, &[]),
    }
}

fn tz_from_str(s: &str) -> Option<i16> {
    let s = s.as_bytes();

    if s.len() != 5 {
        return None;
    }

    let sign: i16 = match &s[0..1] {
        b"+" => 1,
        b"-" => -1,
        _ => {
            return None;
        }
    };

    let hh = from_digit(s[1]) * 10 + from_digit(s[2]);
    let mm = from_digit(s[3]) * 10 + from_digit(s[3]);
    Some(sign * (hh * 60 + mm))
}

fn from_digit(digit: u8) -> i16 {
    if digit >= 48 && digit <= 57 {
        (digit as i16) - 48
    } else {
        0
    }
}

fn sanitize(s: &str) -> String {
    let mut result = String::new();
    for c in s.trim().chars() {
        // Remove control characters except for CR and angle brackets.
        match c as u32 {
            0..=12 => (),
            14..=31 => (),
            60 | 62 => (),
            _ => result.push(c as char),
        }
    }
    result
}

impl fmt::Display for Attribution {
    // Skipping this function because, for some reason, the line
    // containing "f," doesn't register as reached. There is definitely
    // test coverage for the entire function.
    #[cfg(not(tarpaulin_include))]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} <{}> {} {}",
            sanitize(&self.name),
            sanitize(&self.email),
            self.timestamp,
            self.format_tz()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Attribution;

    #[test]
    fn happy_path() {
        let a = Attribution::new("A U Thor", "author@example.com", 1_142_878_501, 150);

        assert_eq!(a.name(), "A U Thor");
        assert_eq!(a.email(), "author@example.com");
        assert_eq!(a.timestamp(), 1_142_878_501);
        assert_eq!(a.tz_offset(), 150);

        assert_eq!(
            a.to_string(),
            "A U Thor <author@example.com> 1142878501 +0230"
        );
    }

    #[test]
    fn parse_legal_cases() {
        let a = Attribution::parse(b"Me <me@example.com> 1234567890 -0700\n").unwrap();
        assert_eq!(a.name(), "Me");
        assert_eq!(a.email(), "me@example.com");
        assert_eq!(a.timestamp(), 1234567890);
        assert_eq!(a.tz_offset(), -420);

        let a = Attribution::parse(b" Me <me@example.com> 1234567890 -0700\n").unwrap();
        assert_eq!(a.name(), " Me");
        assert_eq!(a.email(), "me@example.com");
        assert_eq!(a.timestamp(), 1234567890);
        assert_eq!(a.tz_offset(), -420);

        let a = Attribution::parse(b"A U Thor <author@example.com> 1234567890 -0700").unwrap();
        assert_eq!(a.name(), "A U Thor");
        assert_eq!(a.email(), "author@example.com");
        assert_eq!(a.timestamp(), 1234567890);
        assert_eq!(a.tz_offset(), -420);

        let a = Attribution::parse(b"A U Thor<author@example.com> 1234567890 -0700").unwrap();
        assert_eq!(a.name(), "A U Thor");
        assert_eq!(a.email(), "author@example.com");
        assert_eq!(a.timestamp(), 1234567890);
        assert_eq!(a.tz_offset(), -420);

        let a = Attribution::parse(b"A U Thor<author@example.com>1234567890 +0700").unwrap();
        assert_eq!(a.name(), "A U Thor");
        assert_eq!(a.email(), "author@example.com");
        assert_eq!(a.timestamp(), 1234567890);
        assert_eq!(a.tz_offset(), 420);

        let a = Attribution::parse(b" A U Thor   < author@example.com > 1234567890 -0700").unwrap();
        assert_eq!(a.name(), " A U Thor  ");
        assert_eq!(a.email(), " author@example.com ");
        assert_eq!(a.timestamp(), 1234567890);
        assert_eq!(a.tz_offset(), -420);

        let a = Attribution::parse(b"A U Thor<author@example.com>1234567890 -0700").unwrap();
        assert_eq!(a.name(), "A U Thor");
        assert_eq!(a.email(), "author@example.com");
        assert_eq!(a.timestamp(), 1234567890);
        assert_eq!(a.tz_offset(), -420);
    }

    #[test]
    fn parse_fuzzy_cases() {
        let a = Attribution::parse(
            b"A U Thor <author@example.com>,  C O. Miter <comiter@example.com> 1234567890 -0700",
        )
        .unwrap();
        assert_eq!(a.name(), "A U Thor");
        assert_eq!(a.email(), "author@example.com");
        assert_eq!(a.timestamp(), 1234567890);
        assert_eq!(a.tz_offset(), -420);

        let a = Attribution::parse(b"A U Thor <author@example.com> and others 1234567890 -0700")
            .unwrap();
        assert_eq!(a.name(), "A U Thor");
        assert_eq!(a.email(), "author@example.com");
        assert_eq!(a.timestamp(), 1234567890);
        assert_eq!(a.tz_offset(), -420);

        let a = Attribution::parse(b"A U Thor <author@example.com> and others 1234567890 ~0700")
            .unwrap();
        assert_eq!(a.name(), "A U Thor");
        assert_eq!(a.email(), "author@example.com");
        assert_eq!(a.timestamp(), 1234567890);
        assert_eq!(a.tz_offset(), 0);
    }

    #[test]
    fn parse_bad_utf8() {
        assert_eq!(
            Attribution::parse(b"M\xE2 <me@example.com> 1234567890 -0700"),
            None
        );

        assert_eq!(
            Attribution::parse(b"Me <me@e\x88ample.com> 1234567890 -0700"),
            None
        );

        let a = Attribution::parse(b"A U Thor <author@example.com> and others 1234567890 -07z0")
            .unwrap();
        assert_eq!(a.name(), "A U Thor");
        assert_eq!(a.email(), "author@example.com");
        assert_eq!(a.timestamp(), 1234567890);
        assert_eq!(a.tz_offset(), -420);
        // undefined behavior; this is "reasonable" I guess

        let a = Attribution::parse(b"A U Thor<author@example.com>1234567890 -0\xA700").unwrap();
        assert_eq!(a.name(), "A U Thor");
        assert_eq!(a.email(), "author@example.com");
        assert_eq!(a.timestamp(), 1234567890);
        assert_eq!(a.tz_offset(), 0);
        // Should be zero'd out because TZ isn't valid UTF8.
    }

    #[test]
    fn parse_incomplete_cases() {
        let a = Attribution::parse(b"Me <> 1234567890 -0700").unwrap();
        assert_eq!(a.name(), "Me");
        assert_eq!(a.email(), "");
        assert_eq!(a.timestamp(), 1234567890);
        assert_eq!(a.tz_offset(), -420);

        let a = Attribution::parse(b" <me@example.com> 1234567890 -0700").unwrap();
        assert_eq!(a.name(), "");
        assert_eq!(a.email(), "me@example.com");
        assert_eq!(a.timestamp(), 1234567890);
        assert_eq!(a.tz_offset(), -420);

        let a = Attribution::parse(b" <> 1234567890 -0700").unwrap();
        assert_eq!(a.name(), "");
        assert_eq!(a.email(), "");
        assert_eq!(a.timestamp(), 1234567890);
        assert_eq!(a.tz_offset(), -420);

        let a = Attribution::parse(b"<>").unwrap();
        assert_eq!(a.name(), "");
        assert_eq!(a.email(), "");
        assert_eq!(a.timestamp(), 0);
        assert_eq!(a.tz_offset(), 0);

        let a = Attribution::parse(b" <>").unwrap();
        assert_eq!(a.name(), "");
        assert_eq!(a.email(), "");
        assert_eq!(a.timestamp(), 0);
        assert_eq!(a.tz_offset(), 0);

        let a = Attribution::parse(b"<me@example.com>").unwrap();
        assert_eq!(a.name(), "");
        assert_eq!(a.email(), "me@example.com");
        assert_eq!(a.timestamp(), 0);
        assert_eq!(a.tz_offset(), 0);

        let a = Attribution::parse(b" <me@example.com>").unwrap();
        assert_eq!(a.name(), "");
        assert_eq!(a.email(), "me@example.com");
        assert_eq!(a.timestamp(), 0);
        assert_eq!(a.tz_offset(), 0);

        let a = Attribution::parse(b"Me <>").unwrap();
        assert_eq!(a.name(), "Me");
        assert_eq!(a.email(), "");
        assert_eq!(a.timestamp(), 0);
        assert_eq!(a.tz_offset(), 0);

        let a = Attribution::parse(b"Me <me@example.com>").unwrap();
        assert_eq!(a.name(), "Me");
        assert_eq!(a.email(), "me@example.com");
        assert_eq!(a.timestamp(), 0);
        assert_eq!(a.tz_offset(), 0);

        let a = Attribution::parse(b"Me <me@example.com> 1234567890").unwrap();
        assert_eq!(a.name(), "Me");
        assert_eq!(a.email(), "me@example.com");
        assert_eq!(a.timestamp(), 0);
        assert_eq!(a.tz_offset(), 0);

        let a = Attribution::parse(b"Me <me@example.com> 1234567890 ").unwrap();
        assert_eq!(a.name(), "Me");
        assert_eq!(a.email(), "me@example.com");
        assert_eq!(a.timestamp(), 0);
        assert_eq!(a.tz_offset(), 0);
    }

    #[test]
    fn parse_malformed_cases() {
        assert!(Attribution::parse(b"Me me@example.com> 1234567890 -0700").is_none());
        assert!(Attribution::parse(b"Me <me@example.com 1234567890 -0700").is_none());
    }

    #[test]
    fn sanitize() {
        let a1 = Attribution::new(" A U \x0CThor ", " author@example.com", 1_142_878_501, 150);

        assert_eq!(a1.name(), " A U \x0CThor ");
        assert_eq!(a1.email(), " author@example.com");

        assert_eq!(a1.sanitized_name(), "A U Thor");
        assert_eq!(a1.sanitized_email(), "author@example.com");

        let a2 = Attribution::new(" A U <Thor> ", " author@example.com", 1_142_878_501, 150);

        assert_eq!(a2.name(), " A U <Thor> ");
        assert_eq!(a2.email(), " author@example.com");

        assert_eq!(a2.sanitized_name(), "A U Thor");
        assert_eq!(a2.sanitized_email(), "author@example.com");
    }

    #[test]
    fn format_tz() {
        let a1 = Attribution::new("A U Thor", "author@example.com", 1_142_878_501, 150);
        assert_eq!(a1.format_tz(), "+0230");

        let a2 = Attribution::new("A U Thor", "author@example.com", 1_142_878_501, 0);
        assert_eq!(a2.format_tz(), "+0000");

        let a3 = Attribution::new("A U Thor", "author@example.com", 1_142_878_501, -420);
        assert_eq!(a3.format_tz(), "-0700");
    }

    #[test]
    fn trims_all_whitespace() {
        let a = Attribution::new("  \u{0001} \n ", "  \u{0001} \n ", 1_142_878_501, 0);
        assert_eq!(a.to_string(), " <> 1142878501 +0000");
    }

    #[test]
    fn trims_other_bad_chars() {
        let a = Attribution::new(" Foo\r\n<Bar> ", " Baz>\n\u{1234}<Quux ", 1_142_878_501, 0);
        assert_eq!(a.to_string(), "Foo\rBar <Baz\u{1234}Quux> 1142878501 +0000");
    }

    #[test]
    fn accepts_empty_name_and_email() {
        let a = Attribution::new("", "", 1_142_878_501, 0);
        assert_eq!(a.to_string(), " <> 1142878501 +0000");
    }

    #[test]
    fn accepts_gmt_minus_12_hours() {
        let a = Attribution::new("", "", 1_142_878_501, -720);
        assert_eq!(a.to_string(), " <> 1142878501 -1200");
    }

    #[test]
    fn accepts_gmt_plus_14_hours() {
        let a = Attribution::new("", "", 1_142_878_501, 840);
        assert_eq!(a.to_string(), " <> 1142878501 +1400");
    }

    #[test]
    #[should_panic(expected = "Illegal time zone offset: -721")]
    fn panics_on_illegal_negative_tz() {
        let _a = Attribution::new("", "", 1_142_878_501, -721);
    }

    #[test]
    #[should_panic(expected = "Illegal time zone offset: 841")]
    fn panics_on_illegal_positive_tz() {
        let _a = Attribution::new("", "", 1_142_878_501, 841);
    }
}
