use std::fmt;

/// An `Attribution` combines a person's identity (name and e-mail address)
/// with the timestamp for a particular action.
///
/// Attributions are typically associated with commits or tags in git.
///
/// The `timestamp` value is in milliseconds relative to the Unix era.
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} <{}> {} {}",
            sanitize(&self.name),
            sanitize(&self.email),
            self.timestamp / 1000,
            self.format_tz()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Attribution;

    #[test]
    fn happy_path() {
        let a = Attribution::new("A U Thor", "author@example.com", 1_142_878_501_000, 150);

        assert_eq!(a.name(), "A U Thor");
        assert_eq!(a.email(), "author@example.com");
        assert_eq!(a.timestamp(), 1_142_878_501_000);
        assert_eq!(a.tz_offset(), 150);

        assert_eq!(
            a.to_string(),
            "A U Thor <author@example.com> 1142878501 +0230"
        );
    }

    #[test]
    fn sanitize() {
        let a1 = Attribution::new(
            " A U \x0CThor ",
            " author@example.com",
            1_142_878_501_000,
            150,
        );

        assert_eq!(a1.name(), " A U \x0CThor ");
        assert_eq!(a1.email(), " author@example.com");

        assert_eq!(a1.sanitized_name(), "A U Thor");
        assert_eq!(a1.sanitized_email(), "author@example.com");

        let a2 = Attribution::new(
            " A U <Thor> ",
            " author@example.com",
            1_142_878_501_000,
            150,
        );

        assert_eq!(a2.name(), " A U <Thor> ");
        assert_eq!(a2.email(), " author@example.com");

        assert_eq!(a2.sanitized_name(), "A U Thor");
        assert_eq!(a2.sanitized_email(), "author@example.com");
    }

    #[test]
    fn format_tz() {
        let a1 = Attribution::new("A U Thor", "author@example.com", 1_142_878_501_000, 150);
        assert_eq!(a1.format_tz(), "+0230");

        let a2 = Attribution::new("A U Thor", "author@example.com", 1_142_878_501_000, 0);
        assert_eq!(a2.format_tz(), "+0000");

        let a3 = Attribution::new("A U Thor", "author@example.com", 1_142_878_501_000, -420);
        assert_eq!(a3.format_tz(), "-0700");
    }

    #[test]
    fn trims_all_whitespace() {
        let a = Attribution::new("  \u{0001} \n ", "  \u{0001} \n ", 1_142_878_501_000, 0);
        assert_eq!(a.to_string(), " <> 1142878501 +0000");
    }

    #[test]
    fn trims_other_bad_chars() {
        let a = Attribution::new(
            " Foo\r\n<Bar> ",
            " Baz>\n\u{1234}<Quux ",
            1_142_878_501_000,
            0,
        );
        assert_eq!(a.to_string(), "Foo\rBar <Baz\u{1234}Quux> 1142878501 +0000");
    }

    #[test]
    fn accepts_empty_name_and_email() {
        let a = Attribution::new("", "", 1_142_878_501_000, 0);
        assert_eq!(a.to_string(), " <> 1142878501 +0000");
    }

    #[test]
    fn accepts_gmt_minus_12_hours() {
        let a = Attribution::new("", "", 1_142_878_501_000, -720);
        assert_eq!(a.to_string(), " <> 1142878501 -1200");
    }

    #[test]
    fn accepts_gmt_plus_14_hours() {
        let a = Attribution::new("", "", 1_142_878_501_000, 840);
        assert_eq!(a.to_string(), " <> 1142878501 +1400");
    }

    #[test]
    #[should_panic(expected = "Illegal time zone offset: -721")]
    fn panics_on_illegal_negative_tz() {
        let _a = Attribution::new("", "", 1_142_878_501_000, -721);
    }

    #[test]
    #[should_panic(expected = "Illegal time zone offset: 841")]
    fn panics_on_illegal_positive_tz() {
        let _a = Attribution::new("", "", 1_142_878_501_000, 841);
    }
}
