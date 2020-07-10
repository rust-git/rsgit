use std::result::Result;

extern crate thiserror;
use thiserror::Error;

/// Represents a list of bytes (typically, but not necessarily UTF-8)
/// that is a valid path in a git repo.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitPath<'a> {
    path: &'a [u8],
    checked_platforms: CheckPlatforms,
}

/// Represents a list of bytes (typically, but not necessarily UTF-8)
/// that is a valid path *segment* in a git repo. (Unlike `GitPath`,
/// a `GitPathSegment` may not contain a `/` character.)
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitPathSegment<'a> {
    path: &'a [u8],
    checked_platforms: CheckPlatforms,
}

/// Reasons why a given byte sequence can not be accepted as a git repo path.
#[derive(Debug, Eq, Error, PartialEq)]
pub enum GitPathError {
    #[error("the path is empty")]
    EmptyPath,

    #[error("the path begins with '/'")]
    AbsolutePath,

    #[error("the path ends with '/'")]
    TrailingSlash,

    #[error("the path contains adjacent '/' path separators")]
    DuplicateSlash,

    #[error("the path segment contains '/'")]
    ContainsSlash,

    #[error("the path contains a reserved name `{}`", String::from_utf8_lossy(.0))]
    ReservedName(Vec<u8>),

    #[error("the path contains a NULL character")]
    ContainsNull,

    #[error("the path contains the character `{0}`, which is not allowed on Windows")]
    ContainsInvalidWindowsCharacter(char),

    #[error("the path ends with `{0}`, which is not allowed on Windows")]
    InvalidWindowsNameEnding(char),

    #[error("the name `{}` is a reserved device name on Windows", String::from_utf8_lossy(.0))]
    ReservedWindowsDeviceName(Vec<u8>),

    #[error("the name contains Unicode characters which are ignorable")]
    ContainsIgnorableUnicodeCharacters,

    #[error("the name contains incomplete Unicode characters")]
    ContainsIncompleteUnicodeCharacters,
}

/// Which platform's file naming conventions should be checked?
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckPlatforms {
    pub windows: bool,
    pub mac: bool,
}

impl<'a> GitPath<'a> {
    /// Convert the provided byte vector to a `GitPath` struct if it is acceptable
    /// as a git path. The rules enforced here are slightly different from what
    /// is allowed in a `tree` object in that we allow `/` characters to build
    /// hierarchical paths.
    #[cfg_attr(tarpaulin, skip)]
    pub fn new(path: &'a [u8]) -> Result<GitPath<'a>, GitPathError> {
        // Argh. `cargo fmt` reformats this into a format that generates
        // "coverage" for some of the arguments below, but not all.
        GitPath::new_with_platform_checks(
            path,
            &CheckPlatforms {
                windows: false,
                mac: false,
            },
        )
    }

    /// Convert the provided byte vector to a `GitPath` struct if it is acceptable
    /// as a git path. In addition to the typical constraints enforced via `new()`,
    /// also check platform-specific rules.
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_with_platform_checks(
        path: &'a [u8],
        platforms: &CheckPlatforms,
    ) -> Result<GitPath<'a>, GitPathError> {
        // Argh. `cargo fmt` reformats this into a format that generates
        // "coverage" for some of the arguments below, but not all.
        match check_path(path, platforms) {
            Ok(()) => Ok(GitPath {
                path,
                checked_platforms: platforms.clone(),
            }),
            Err(err) => Err(err),
        }
    }

    /// Return the path.
    pub fn path(&self) -> &[u8] {
        self.path
    }

    /// Return which platforms were checked for this path.
    pub fn checked_platforms(&self) -> &CheckPlatforms {
        &self.checked_platforms
    }
}

impl<'a> GitPathSegment<'a> {
    /// Convert the provided byte vector to a `GitPathSegment` struct if it is
    /// acceptable as a git path segment. Similarly to a `tree` object, we do not
    /// allow `/` characters.
    #[cfg_attr(tarpaulin, skip)]
    pub fn new(path: &'a [u8]) -> Result<GitPathSegment<'a>, GitPathError> {
        // Argh. `cargo fmt` reformats this into a format that generates
        // "coverage" for some of the arguments below, but not all.
        GitPathSegment::new_with_platform_checks(
            path,
            &CheckPlatforms {
                windows: false,
                mac: false,
            },
        )
    }

    /// Convert the provided byte vector to a `GitPathSegment` struct if it is acceptable
    /// as a git path. In addition to the typical constraints enforced via `new()`,
    /// also check platform-specific rules.
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_with_platform_checks(
        path: &'a [u8],
        platforms: &CheckPlatforms,
    ) -> Result<GitPathSegment<'a>, GitPathError> {
        // Argh. `cargo fmt` reformats this into a format that generates
        // "coverage" for some of the arguments below, but not all.
        match check_segment(path, platforms) {
            Ok(()) => Ok(GitPathSegment {
                path,
                checked_platforms: platforms.clone(),
            }),
            Err(err) => Err(err),
        }
    }

    /// Return the path.
    pub fn path(&self) -> &[u8] {
        self.path
    }

    /// Return which platforms were checked for this path.
    pub fn checked_platforms(&self) -> &CheckPlatforms {
        &self.checked_platforms
    }
}

fn check_path(path: &[u8], platforms: &CheckPlatforms) -> Result<(), GitPathError> {
    if path.is_empty() {
        Err(GitPathError::EmptyPath)
    } else if path.starts_with(b"/") {
        Err(GitPathError::AbsolutePath)
    } else if path.ends_with(b"/") {
        Err(GitPathError::TrailingSlash)
    } else {
        for segment in path.split(|c| *c == 47) {
            match check_segment(segment, platforms) {
                Err(GitPathError::EmptyPath) => Err(GitPathError::DuplicateSlash),
                x => x,
            }?;
        }
        Ok(())
    }
}

fn check_segment(segment: &[u8], platforms: &CheckPlatforms) -> Result<(), GitPathError> {
    if segment.is_empty() {
        Err(GitPathError::EmptyPath)
    } else if segment.contains(&0) {
        Err(GitPathError::ContainsNull)
    } else if segment.contains(&47) {
        Err(GitPathError::ContainsSlash)
    } else {
        check_git_reserved_name(segment)?;
        check_windows_git_name(segment)?;

        if platforms.windows {
            check_windows_special_characters(segment)?;
            check_windows_segment_ending(segment)?;
            check_windows_device_name(segment)?;
        }

        if platforms.mac {
            check_git_path_with_mac_ignorables(segment)?;
            check_truncated_utf8_for_mac(segment)?
        }

        Ok(())
    }
}

fn check_git_reserved_name(segment: &[u8]) -> Result<(), GitPathError> {
    let reserved = match segment {
        b"." => true,
        b".." => true,
        b".git" => true,
        _ => is_normalized_git(segment),
    };

    if reserved {
        Err(GitPathError::ReservedName(segment.to_owned()))
    } else {
        Ok(())
    }
}

fn is_normalized_git(segment: &[u8]) -> bool {
    if segment.len() < 4 {
        return false;
    }

    if segment[0] != b'.' {
        return false;
    }

    if segment[1] != b'G' && segment[1] != b'g' {
        return false;
    }

    if segment[2] != b'I' && segment[2] != b'i' {
        return false;
    }

    if segment[3] != b'T' && segment[3] != b't' {
        return false;
    }

    match &segment[4..] {
        b"" => true,
        b" " => true,
        b"." => true,
        b". " => true,
        b" ." => true,
        b" . " => true,
        _ => false,
    }
}

fn check_windows_git_name(segment: &[u8]) -> Result<(), GitPathError> {
    if segment.len() == 5 {
        let mut segment_lc: [u8; 5] = [0u8; 5];
        segment_lc.clone_from_slice(segment);
        segment_lc.make_ascii_lowercase();
        if &segment_lc == b"git~1" {
            Err(GitPathError::ReservedName(segment.to_owned()))
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

fn check_windows_special_characters(segment: &[u8]) -> Result<(), GitPathError> {
    for c in segment {
        let invalid = match c {
            b'"' => true,
            b'*' => true,
            b':' => true,
            b'<' => true,
            b'>' => true,
            b'?' => true,
            b'\\' => true,
            b'|' => true,
            0..=31 => true,
            _ => false,
        };

        if invalid {
            return Err(GitPathError::ContainsInvalidWindowsCharacter(*c as char));
        }
    }

    Ok(())
}

fn check_windows_segment_ending(segment: &[u8]) -> Result<(), GitPathError> {
    if segment.ends_with(b".") {
        Err(GitPathError::InvalidWindowsNameEnding('.'))
    } else if segment.ends_with(b" ") {
        Err(GitPathError::InvalidWindowsNameEnding(' '))
    } else {
        Ok(())
    }
}

#[cfg_attr(tarpaulin, skip)]
fn check_windows_device_name(segment: &[u8]) -> Result<(), GitPathError> {
    // Coverage skip justification: We have to cover the `None` case,
    // but we know it will never happen because we earlier test for
    // and bail out if the segment name is empty.
    match segment.split(|c| *c == b'.').next() {
        Some(before_dot) => {
            let illegal = match before_dot.len() {
                3 => {
                    let mut name: [u8; 3] = [0; 3];
                    name.clone_from_slice(before_dot);
                    name.make_ascii_lowercase();
                    match &name {
                        b"aux" => true,
                        b"con" => true,
                        b"nul" => true,
                        b"prn" => true,
                        _ => false,
                    }
                }
                4 => {
                    let mut name: [u8; 3] = [0; 3];
                    name.clone_from_slice(&before_dot[0..3]);
                    name.make_ascii_lowercase();
                    let illegal = match &name {
                        b"com" => true,
                        b"lpt" => true,
                        _ => false,
                    };
                    let digit = before_dot[3];
                    illegal && digit >= b'1' && digit <= b'9'
                }
                _ => false,
            };

            if illegal {
                Err(GitPathError::ReservedWindowsDeviceName(segment.to_owned()))
            } else {
                Ok(())
            }
        }
        _ => Ok(()),
    }
}

fn check_git_path_with_mac_ignorables(segment: &[u8]) -> Result<(), GitPathError> {
    if match_mac_hfs_path(segment, b".git") {
        Err(GitPathError::ContainsIgnorableUnicodeCharacters)
    } else {
        Ok(())
    }
}

fn check_truncated_utf8_for_mac(segment: &[u8]) -> Result<(), GitPathError> {
    let tail3 = &segment[0.max(segment.len() - 2)..];
    if tail3.contains(&0xE2) || tail3.contains(&0xEF) {
        Err(GitPathError::ContainsIncompleteUnicodeCharacters)
    } else {
        Ok(())
    }
}

fn match_mac_hfs_path(segment: &[u8], m: &[u8]) -> bool {
    if segment.is_empty() && m.is_empty() {
        true
    } else if segment.is_empty() {
        false
    } else {
        if segment.len() >= 3 {
            let ignorable_char = match segment[0..3] {
                // U+200C 0xe2808c ZERO WIDTH NON-JOINER
                [0xE2, 0x80, 0x8C] => true,

                // U+200D 0xe2808d ZERO WIDTH JOINER
                [0xE2, 0x80, 0x8D] => true,

                // U+200E 0xe2808e LEFT-TO-RIGHT MARK
                [0xE2, 0x80, 0x8E] => true,

                // U+200F 0xe2808f RIGHT-TO-LEFT MARK
                [0xE2, 0x80, 0x8F] => true,

                // U+202A 0xe280aa LEFT-TO-RIGHT EMBEDDING
                [0xE2, 0x80, 0xAA] => true,

                // U+202B 0xe280ab RIGHT-TO-LEFT EMBEDDING
                [0xE2, 0x80, 0xAB] => true,

                // U+202C 0xe280ac POP DIRECTIONAL FORMATTING
                [0xE2, 0x80, 0xAC] => true,

                // U+202D 0xe280ad LEFT-TO-RIGHT OVERRIDE
                [0xE2, 0x80, 0xAD] => true,

                // U+202E 0xe280ae RIGHT-TO-LEFT OVERRIDE
                [0xE2, 0x80, 0xAE] => true,

                // U+206A 0xe281aa INHIBIT SYMMETRIC SWAPPING
                [0xE2, 0x81, 0xAA] => true,

                // U+206B 0xe281ab ACTIVATE SYMMETRIC SWAPPING
                [0xE2, 0x81, 0xAB] => true,

                // U+206C 0xe281ac INHIBIT ARABIC FORM SHAPING
                [0xE2, 0x81, 0xAC] => true,

                // U+206D 0xe281ad ACTIVATE ARABIC FORM SHAPING
                [0xE2, 0x81, 0xAD] => true,

                // U+206E 0xe281ae NATIONAL DIGIT SHAPES
                [0xE2, 0x81, 0xAE] => true,

                // U+206F 0xe281af NOMINAL DIGIT SHAPES
                [0xE2, 0x81, 0xAF] => true,

                // U+FEFF 0xefbbbf BYTE ORDER MARK
                [0xEF, 0xBB, 0xBF] => true,

                _ => false,
            };

            if ignorable_char {
                return match_mac_hfs_path(&segment[3..], m);
            }
        }

        if m.is_empty() || segment.first() != m.first() {
            false
        } else {
            match_mac_hfs_path(&segment[1..], &m[1..])
        }
    }
}

#[cfg(test)]
mod path_tests {
    use super::*;

    #[test]
    fn basic_case() {
        // No platform-specific checks.
        assert_eq!(GitPath::new(b"").unwrap_err(), GitPathError::EmptyPath);

        let a = GitPath::new(b"a").unwrap();
        assert_eq!(a.path(), b"a");
        assert_eq!(
            a.checked_platforms(),
            &CheckPlatforms {
                mac: false,
                windows: false
            }
        );

        let a = GitPath::new(b"a/b").unwrap();
        assert_eq!(a.path(), b"a/b");

        assert_eq!(
            GitPath::new(b"a//b").unwrap_err(),
            GitPathError::DuplicateSlash
        );

        assert_eq!(GitPath::new(b"/a").unwrap_err(), GitPathError::AbsolutePath);

        assert_eq!(
            GitPath::new(b"a\0b").unwrap_err(),
            GitPathError::ContainsNull
        );

        let a = GitPath::new(b"ab/cd/ef").unwrap();
        assert_eq!(a.path(), b"ab/cd/ef");

        assert_eq!(
            GitPath::new(b"ab/cd//ef").unwrap_err(),
            GitPathError::DuplicateSlash
        );

        assert_eq!(
            GitPath::new(b"a/").unwrap_err(),
            GitPathError::TrailingSlash
        );
        assert_eq!(
            GitPath::new(b"ab/cd/ef/").unwrap_err(),
            GitPathError::TrailingSlash
        );
    }

    const GIT_RESERVED_NAMES: [&[u8]; 11] = [
        b".", b"..", b".git", b".git.", b".git ", b".git. ", b".git . ", b".Git", b".gIt", b".giT",
        b".giT.",
    ];

    const ALMOST_GIT_RESERVED_NAMES: [&[u8]; 6] = [
        b".gxt",
        b".git..",
        b".gitfoobar",
        b".gitfoo bar",
        b".gitfoobar.",
        b".gitfoobar..",
    ];

    #[test]
    fn git_reserved_names() {
        for name in &GIT_RESERVED_NAMES {
            assert_eq!(
                GitPath::new(name).unwrap_err(),
                GitPathError::ReservedName(name.to_vec())
            );
        }

        for name in &ALMOST_GIT_RESERVED_NAMES {
            let a = GitPath::new(name).unwrap();
            assert_eq!(&a.path(), name);
        }
    }

    const WINDOWS_GIT_NAMES: [&[u8]; 2] = [b"GIT~1", b"GiT~1"];
    const ALMOST_WINDOWS_GIT_NAMES: [&[u8]; 2] = [b"GIT~11", b"GIT~2"];

    #[test]
    fn windows_variations_on_dot_git_name() {
        // This constraint applies to all platforms, since a ".git"-like name
        // on *any* platform will cause problems when moving to Windows.
        for name in &WINDOWS_GIT_NAMES {
            assert_eq!(
                GitPath::new(name).unwrap_err(),
                GitPathError::ReservedName(name.to_vec())
            );
        }

        for name in &ALMOST_WINDOWS_GIT_NAMES {
            let a = GitPath::new(name).unwrap();
            assert_eq!(&a.path(), name);
        }
    }

    const INVALID_WINDOWS_PATHS: [&[u8]; 14] = [
        b"\"",
        b"*",
        b":",
        b"<",
        b">",
        b"?",
        b"\\",
        b"|",
        &[1],
        &[2],
        &[3],
        &[4],
        &[7],
        &[31],
    ];

    #[test]
    fn invalid_windows_characters() {
        for name in &INVALID_WINDOWS_PATHS {
            let a = GitPath::new(name).unwrap();
            assert_eq!(&a.path(), name);

            let c = *(name.first().unwrap()) as char;

            assert_eq!(
                GitPath::new_with_platform_checks(
                    name,
                    &CheckPlatforms {
                        windows: true,
                        mac: false
                    }
                )
                .unwrap_err(),
                GitPathError::ContainsInvalidWindowsCharacter(c)
            );
        }

        for n in &INVALID_WINDOWS_PATHS {
            let mut name: Vec<u8> = Vec::new();
            name.push(b'a');
            name.extend_from_slice(n);
            name.push(b'b');

            let a = GitPath::new(&name).unwrap();
            assert_eq!(a.path(), name.as_slice());

            let c = *(n.first().unwrap()) as char;

            assert_eq!(
                GitPath::new_with_platform_checks(
                    &name,
                    &CheckPlatforms {
                        windows: true,
                        mac: false
                    }
                )
                .unwrap_err(),
                GitPathError::ContainsInvalidWindowsCharacter(c)
            );
        }

        let a = GitPath::new_with_platform_checks(
            b"ab/cd/ef",
            &CheckPlatforms {
                windows: true,
                mac: false,
            },
        )
        .unwrap();
        assert_eq!(a.path(), b"ab/cd/ef");
        assert_eq!(
            a.checked_platforms(),
            &CheckPlatforms {
                mac: false,
                windows: true
            }
        )
    }

    #[test]
    #[cfg_attr(tarpaulin, skip)]
    fn invalid_windows_name_ending() {
        let name = b"abc.";
        let a = GitPath::new(name).unwrap();
        assert_eq!(&a.path(), name);

        assert_eq!(
            GitPath::new_with_platform_checks(
                name,
                &CheckPlatforms {
                    windows: true,
                    mac: false
                }
            )
            .unwrap_err(),
            GitPathError::InvalidWindowsNameEnding('.')
        );

        let name = b"abc ";
        let a = GitPath::new(name).unwrap();
        assert_eq!(&a.path(), name);

        assert_eq!(
            GitPath::new_with_platform_checks(
                name,
                &CheckPlatforms {
                    windows: true,
                    mac: false
                }
            )
            .unwrap_err(),
            GitPathError::InvalidWindowsNameEnding(' ')
        );
    }

    const WINDOWS_DEVICE_NAMES: [&[u8]; 8] = [
        b"aux", b"con", b"com1", b"com7", b"lpt1", b"lpt3", b"nul", b"prn",
    ];
    const ALMOST_WINDOWS_DEVICE_NAMES: [&[u8]; 9] = [
        b"aub", b"con1", b"com", b"lpt", b"nul3", b"prn8", b"co", b"com12", b"com0",
    ];

    #[test]
    fn invalid_windows_device_names() {
        for name in &WINDOWS_DEVICE_NAMES {
            let a = GitPath::new(name).unwrap();
            assert_eq!(&a.path(), name);

            assert_eq!(
                GitPath::new_with_platform_checks(
                    name,
                    &CheckPlatforms {
                        windows: true,
                        mac: false
                    }
                )
                .unwrap_err(),
                GitPathError::ReservedWindowsDeviceName(name.to_vec())
            );
        }

        for name in &ALMOST_WINDOWS_DEVICE_NAMES {
            let a = GitPath::new(name).unwrap();
            assert_eq!(&a.path(), name);

            let a = GitPath::new_with_platform_checks(
                &name,
                &CheckPlatforms {
                    windows: true,
                    mac: false,
                },
            )
            .unwrap();

            assert_eq!(&a.path(), name);
        }
    }

    const MAC_HFS_GIT_NAMES: [&str; 16] = [
        ".gi\u{200C}t",
        ".gi\u{200D}t",
        ".gi\u{200E}t",
        ".gi\u{200F}t",
        ".gi\u{202A}t",
        ".gi\u{202B}t",
        ".gi\u{202C}t",
        ".gi\u{202D}t",
        ".gi\u{202E}t",
        ".gi\u{206A}t",
        "\u{206B}.git",
        "\u{206C}.git",
        "\u{206D}.git",
        "\u{206E}.git",
        "\u{206F}.git",
        ".git\u{FEFF}",
    ];

    const ALMOST_MAC_HFS_GIT_NAMES: [&str; 3] = [".gi", ".git\u{200C}x", ".kit\u{200C}"];

    #[test]
    fn mac_variations_on_dot_git_name() {
        for name in &MAC_HFS_GIT_NAMES {
            let name = name.as_bytes();

            let a = GitPath::new(name).unwrap();
            assert_eq!(&a.path(), &name);

            assert_eq!(
                GitPath::new_with_platform_checks(
                    name,
                    &CheckPlatforms {
                        windows: false,
                        mac: true
                    }
                )
                .unwrap_err(),
                GitPathError::ContainsIgnorableUnicodeCharacters
            );
        }

        for name in &ALMOST_MAC_HFS_GIT_NAMES {
            let name = name.as_bytes();

            let a = GitPath::new(name).unwrap();
            assert_eq!(&a.path(), &name);

            let a = GitPath::new_with_platform_checks(
                name,
                &CheckPlatforms {
                    windows: false,
                    mac: true,
                },
            )
            .unwrap();
            assert_eq!(&a.path(), &name);
            assert_eq!(
                a.checked_platforms(),
                &CheckPlatforms {
                    mac: true,
                    windows: false
                }
            )
        }
    }

    #[test]
    fn mac_badly_formed_utf8() {
        assert_eq!(
            GitPath::new_with_platform_checks(
                &[97, 98, 0xE2, 0x80],
                &CheckPlatforms {
                    mac: true,
                    windows: false
                }
            )
            .unwrap_err(),
            GitPathError::ContainsIncompleteUnicodeCharacters
        );

        assert_eq!(
            GitPath::new_with_platform_checks(
                &[97, 98, 0xEF, 0x80],
                &CheckPlatforms {
                    mac: true,
                    windows: false
                }
            )
            .unwrap_err(),
            GitPathError::ContainsIncompleteUnicodeCharacters
        );

        let name = &[97, 98, 0xE2, 0x80, 0xAE];
        let a = GitPath::new_with_platform_checks(
            name,
            &CheckPlatforms {
                windows: false,
                mac: true,
            },
        )
        .unwrap();

        assert_eq!(&a.path(), &name);
        assert_eq!(
            a.checked_platforms(),
            &CheckPlatforms {
                mac: true,
                windows: false
            }
        );

        let bad_name = b".git\xEF";
        let a = GitPath::new(bad_name).unwrap();

        assert_eq!(&a.path(), bad_name);
        assert_eq!(
            a.checked_platforms(),
            &CheckPlatforms {
                mac: false,
                windows: false
            }
        );

        assert_eq!(
            GitPath::new_with_platform_checks(
                bad_name,
                &CheckPlatforms {
                    mac: true,
                    windows: false
                }
            )
            .unwrap_err(),
            GitPathError::ContainsIncompleteUnicodeCharacters
        );

        let bad_name = b".git\xE2\xAB";
        let a = GitPath::new(bad_name).unwrap();

        assert_eq!(&a.path(), bad_name);
        assert_eq!(
            a.checked_platforms(),
            &CheckPlatforms {
                mac: false,
                windows: false
            }
        );

        assert_eq!(
            GitPath::new_with_platform_checks(
                bad_name,
                &CheckPlatforms {
                    mac: true,
                    windows: false
                }
            )
            .unwrap_err(),
            GitPathError::ContainsIncompleteUnicodeCharacters
        );
    }
}

#[cfg(test)]
mod path_segment_tests {
    use super::*;

    #[test]
    #[cfg_attr(tarpaulin, skip)]
    fn basic_case() {
        // No platform-specific checks.
        assert_eq!(
            GitPathSegment::new(b"").unwrap_err(),
            GitPathError::EmptyPath
        );

        let a = GitPathSegment::new(b"a").unwrap();
        assert_eq!(a.path(), b"a");
        assert_eq!(
            a.checked_platforms(),
            &CheckPlatforms {
                mac: false,
                windows: false
            }
        );

        assert_eq!(
            GitPathSegment::new(b"a/b").unwrap_err(),
            GitPathError::ContainsSlash
        );

        assert_eq!(
            GitPathSegment::new(b"a//b").unwrap_err(),
            GitPathError::ContainsSlash
        );

        assert_eq!(
            GitPathSegment::new(b"/a").unwrap_err(),
            GitPathError::ContainsSlash
        );

        assert_eq!(
            GitPathSegment::new(b"a\0b").unwrap_err(),
            GitPathError::ContainsNull
        );

        assert_eq!(
            GitPathSegment::new(b"a/").unwrap_err(),
            GitPathError::ContainsSlash
        );
        assert_eq!(
            GitPathSegment::new(b"ab/cd/ef/").unwrap_err(),
            GitPathError::ContainsSlash
        );
    }

    const GIT_RESERVED_NAMES: [&[u8]; 11] = [
        b".", b"..", b".git", b".git.", b".git ", b".git. ", b".git . ", b".Git", b".gIt", b".giT",
        b".giT.",
    ];

    const ALMOST_GIT_RESERVED_NAMES: [&[u8]; 6] = [
        b".gxt",
        b".git..",
        b".gitfoobar",
        b".gitfoo bar",
        b".gitfoobar.",
        b".gitfoobar..",
    ];

    #[test]
    fn git_reserved_names() {
        for name in &GIT_RESERVED_NAMES {
            assert_eq!(
                GitPathSegment::new(name).unwrap_err(),
                GitPathError::ReservedName(name.to_vec())
            );
        }

        for name in &ALMOST_GIT_RESERVED_NAMES {
            let a = GitPathSegment::new(name).unwrap();
            assert_eq!(&a.path(), name);
        }
    }

    const WINDOWS_GIT_NAMES: [&[u8]; 2] = [b"GIT~1", b"GiT~1"];
    const ALMOST_WINDOWS_GIT_NAMES: [&[u8]; 2] = [b"GIT~11", b"GIT~2"];

    #[test]
    fn windows_variations_on_dot_git_name() {
        // This constraint applies to all platforms, since a ".git"-like name
        // on *any* platform will cause problems when moving to Windows.
        for name in &WINDOWS_GIT_NAMES {
            assert_eq!(
                GitPathSegment::new(name).unwrap_err(),
                GitPathError::ReservedName(name.to_vec())
            );
        }

        for name in &ALMOST_WINDOWS_GIT_NAMES {
            let a = GitPathSegment::new(name).unwrap();
            assert_eq!(&a.path(), name);
        }
    }

    const INVALID_WINDOWS_PATHS: [&[u8]; 14] = [
        b"\"",
        b"*",
        b":",
        b"<",
        b">",
        b"?",
        b"\\",
        b"|",
        &[1],
        &[2],
        &[3],
        &[4],
        &[7],
        &[31],
    ];

    #[test]
    fn invalid_windows_characters() {
        for name in &INVALID_WINDOWS_PATHS {
            let a = GitPathSegment::new(name).unwrap();
            assert_eq!(&a.path(), name);

            let c = *(name.first().unwrap()) as char;

            assert_eq!(
                GitPathSegment::new_with_platform_checks(
                    name,
                    &CheckPlatforms {
                        windows: true,
                        mac: false
                    }
                )
                .unwrap_err(),
                GitPathError::ContainsInvalidWindowsCharacter(c)
            );
        }

        for n in &INVALID_WINDOWS_PATHS {
            let mut name: Vec<u8> = Vec::new();
            name.push(b'a');
            name.extend_from_slice(n);
            name.push(b'b');

            let a = GitPathSegment::new(&name).unwrap();
            assert_eq!(a.path(), name.as_slice());

            let c = *(n.first().unwrap()) as char;

            assert_eq!(
                GitPathSegment::new_with_platform_checks(
                    &name,
                    &CheckPlatforms {
                        windows: true,
                        mac: false
                    }
                )
                .unwrap_err(),
                GitPathError::ContainsInvalidWindowsCharacter(c)
            );
        }
    }

    #[test]
    #[cfg_attr(tarpaulin, skip)]
    fn invalid_windows_name_ending() {
        let name = b"abc.";
        let a = GitPathSegment::new(name).unwrap();
        assert_eq!(&a.path(), name);

        assert_eq!(
            GitPathSegment::new_with_platform_checks(
                name,
                &CheckPlatforms {
                    windows: true,
                    mac: false
                }
            )
            .unwrap_err(),
            GitPathError::InvalidWindowsNameEnding('.')
        );

        let name = b"abc ";
        let a = GitPathSegment::new(name).unwrap();
        assert_eq!(&a.path(), name);

        assert_eq!(
            GitPathSegment::new_with_platform_checks(
                name,
                &CheckPlatforms {
                    windows: true,
                    mac: false
                }
            )
            .unwrap_err(),
            GitPathError::InvalidWindowsNameEnding(' ')
        );
    }

    const WINDOWS_DEVICE_NAMES: [&[u8]; 8] = [
        b"aux", b"con", b"com1", b"com7", b"lpt1", b"lpt3", b"nul", b"prn",
    ];
    const ALMOST_WINDOWS_DEVICE_NAMES: [&[u8]; 9] = [
        b"aub", b"con1", b"com", b"lpt", b"nul3", b"prn8", b"co", b"com12", b"com0",
    ];

    #[test]
    fn invalid_windows_device_names() {
        for name in &WINDOWS_DEVICE_NAMES {
            let a = GitPathSegment::new(name).unwrap();
            assert_eq!(&a.path(), name);

            assert_eq!(
                GitPathSegment::new_with_platform_checks(
                    name,
                    &CheckPlatforms {
                        windows: true,
                        mac: false
                    }
                )
                .unwrap_err(),
                GitPathError::ReservedWindowsDeviceName(name.to_vec())
            );
        }

        for name in &ALMOST_WINDOWS_DEVICE_NAMES {
            let a = GitPathSegment::new(name).unwrap();
            assert_eq!(&a.path(), name);

            let a = GitPathSegment::new_with_platform_checks(
                &name,
                &CheckPlatforms {
                    windows: true,
                    mac: false,
                },
            )
            .unwrap();

            assert_eq!(&a.path(), name);
        }
    }

    const MAC_HFS_GIT_NAMES: [&str; 16] = [
        ".gi\u{200C}t",
        ".gi\u{200D}t",
        ".gi\u{200E}t",
        ".gi\u{200F}t",
        ".gi\u{202A}t",
        ".gi\u{202B}t",
        ".gi\u{202C}t",
        ".gi\u{202D}t",
        ".gi\u{202E}t",
        ".gi\u{206A}t",
        "\u{206B}.git",
        "\u{206C}.git",
        "\u{206D}.git",
        "\u{206E}.git",
        "\u{206F}.git",
        ".git\u{FEFF}",
    ];

    const ALMOST_MAC_HFS_GIT_NAMES: [&str; 3] = [".gi", ".git\u{200C}x", ".kit\u{200C}"];

    #[test]
    fn mac_variations_on_dot_git_name() {
        for name in &MAC_HFS_GIT_NAMES {
            let name = name.as_bytes();

            let a = GitPathSegment::new(name).unwrap();
            assert_eq!(&a.path(), &name);

            assert_eq!(
                GitPathSegment::new_with_platform_checks(
                    name,
                    &CheckPlatforms {
                        windows: false,
                        mac: true
                    }
                )
                .unwrap_err(),
                GitPathError::ContainsIgnorableUnicodeCharacters
            );
        }

        for name in &ALMOST_MAC_HFS_GIT_NAMES {
            let name = name.as_bytes();

            let a = GitPathSegment::new(name).unwrap();
            assert_eq!(&a.path(), &name);

            let a = GitPathSegment::new_with_platform_checks(
                name,
                &CheckPlatforms {
                    windows: false,
                    mac: true,
                },
            )
            .unwrap();
            assert_eq!(&a.path(), &name);
            assert_eq!(
                a.checked_platforms(),
                &CheckPlatforms {
                    mac: true,
                    windows: false
                }
            )
        }
    }

    #[test]
    fn mac_badly_formed_utf8() {
        assert_eq!(
            GitPathSegment::new_with_platform_checks(
                &[97, 98, 0xE2, 0x80],
                &CheckPlatforms {
                    mac: true,
                    windows: false
                }
            )
            .unwrap_err(),
            GitPathError::ContainsIncompleteUnicodeCharacters
        );

        assert_eq!(
            GitPathSegment::new_with_platform_checks(
                &[97, 98, 0xEF, 0x80],
                &CheckPlatforms {
                    mac: true,
                    windows: false
                }
            )
            .unwrap_err(),
            GitPathError::ContainsIncompleteUnicodeCharacters
        );

        let name = &[97, 98, 0xE2, 0x80, 0xAE];
        let a = GitPathSegment::new_with_platform_checks(
            name,
            &CheckPlatforms {
                windows: false,
                mac: true,
            },
        )
        .unwrap();

        assert_eq!(&a.path(), &name);
        assert_eq!(
            a.checked_platforms(),
            &CheckPlatforms {
                mac: true,
                windows: false
            }
        );

        let bad_name = b".git\xEF";
        let a = GitPathSegment::new(bad_name).unwrap();

        assert_eq!(&a.path(), bad_name);
        assert_eq!(
            a.checked_platforms(),
            &CheckPlatforms {
                mac: false,
                windows: false
            }
        );

        assert_eq!(
            GitPathSegment::new_with_platform_checks(
                bad_name,
                &CheckPlatforms {
                    mac: true,
                    windows: false
                }
            )
            .unwrap_err(),
            GitPathError::ContainsIncompleteUnicodeCharacters
        );

        let bad_name = b".git\xE2\xAB";
        let a = GitPathSegment::new(bad_name).unwrap();

        assert_eq!(&a.path(), bad_name);
        assert_eq!(
            a.checked_platforms(),
            &CheckPlatforms {
                mac: false,
                windows: false
            }
        );

        assert_eq!(
            GitPathSegment::new_with_platform_checks(
                bad_name,
                &CheckPlatforms {
                    mac: true,
                    windows: false
                }
            )
            .unwrap_err(),
            GitPathError::ContainsIncompleteUnicodeCharacters
        );
    }
}
