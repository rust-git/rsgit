use super::{parse_utils, ContentSource, ContentSourceResult};

use crate::path::{CheckPlatforms, FileMode, PathMode, PathSegment};

use std::cmp::Ordering;
use std::collections::HashSet;
use std::io::BufRead;

use unicode_normalization::UnicodeNormalization;

pub(crate) fn tree_is_valid(s: &dyn ContentSource) -> ContentSourceResult<bool> {
    tree_is_valid_with_platform_checks(
        s,
        &CheckPlatforms {
            windows: false,
            mac: false,
        },
    )
}

pub(crate) fn tree_is_valid_with_platform_checks(
    s: &dyn ContentSource,
    platforms: &CheckPlatforms,
) -> ContentSourceResult<bool> {
    let mut r = s.open()?;

    let mut previous_line: Vec<u8> = Vec::new();
    let mut this_line: Vec<u8> = Vec::new();
    let mut maybe_lingering_trees: Vec<Vec<u8>> = Vec::new();

    // If we're enforcing platform-specific naming conventions,
    // we need to retain a set of names previously seen. The same
    // name could appear twice (for example, "A", followed by "B",
    // and then "a"). This would be legal in Posix file systems.
    // That sequence would be properly sorted in git, but should be
    // disallowed when either Windows or Mac platform rules are
    // being checked. We use a HashSet to track previously-seen
    // names in that case.
    let mut lc_names = HashSet::new();
    let check_lc_names = platforms.mac || platforms.windows;

    loop {
        this_line.clear();

        if r.read_until(0, &mut this_line)? == 0 {
            // We've reached EOF: It's good.
            return Ok(true);
        }

        let this_line_slice = this_line.as_slice();
        let this_path_mode = match parse_path_mode(&this_line_slice, platforms) {
            Some(pm) => pm,
            None => {
                return Ok(false);
            }
        };

        if check_lc_names {
            if let Ok(path) = String::from_utf8(this_path_mode.path.to_vec()) {
                let mut lc_path = path.to_lowercase();
                if platforms.mac {
                    lc_path = lc_path.nfc().collect::<String>();
                }
                if lc_names.contains(&lc_path) {
                    return Ok(false);
                }
                lc_names.insert(lc_path);
            }
        }

        if !previous_line.is_empty() {
            let previous_line_slice = previous_line.as_slice();
            let previous_path_mode = parse_path_mode(&previous_line_slice, platforms).unwrap();
            // .unwrap() seems justified here since we had previously
            // parsed this successfully. Ultimately, I'd like to find a way
            // to retain the previous parsing through this next iteration,
            // but managing that lifecycle without a heap allocation seems
            // tricky.

            if this_path_mode.path == previous_path_mode.path {
                return Ok(false);
            }

            if this_path_mode.cmp(&previous_path_mode) != Ordering::Greater {
                return Ok(false);
            }

            if !maybe_lingering_trees.is_empty() {
                for i in 0..maybe_lingering_trees.len() {
                    let name_as_tree = PathMode {
                        path: &maybe_lingering_trees[i],
                        mode: FileMode::Tree,
                    };

                    match name_as_tree.cmp_same_name(&this_path_mode) {
                        Ordering::Less => {
                            maybe_lingering_trees.truncate(i);
                            break;
                        }
                        Ordering::Equal => {
                            return Ok(false);
                        }
                        Ordering::Greater => (),
                    }
                }
            }

            if previous_path_mode.cmp_same_name(&this_path_mode) == Ordering::Greater {
                maybe_lingering_trees.push(previous_path_mode.path.to_owned());
            }
        }

        let mut object_id = [0u8; 20];
        match r.read(&mut object_id) {
            Ok(20) => (),
            _ => {
                return Ok(false);
            }
        }

        if object_id.iter().all(|c| c == &0) {
            return Ok(false);
        }

        previous_line = this_line;
        this_line = Vec::new();
    }
}

fn parse_path_mode<'a>(line: &'a &[u8], platforms: &CheckPlatforms) -> Option<PathMode<'a>> {
    if !line.contains(&b' ') {
        return None;
    }

    let (file_mode, path) = parse_utils::split_once(line, &b' ');
    if file_mode.starts_with(b"0") {
        return None;
    }

    let file_mode = match FileMode::from_octal_slice(file_mode) {
        Some(m) => m,
        None => return None,
    };

    if !path.ends_with(&[0]) {
        return None;
    }

    let (path, _) = parse_utils::split_once(path, &0);
    if PathSegment::new_with_platform_checks(path, platforms).is_err() {
        return None;
    }

    Some(PathMode {
        path,
        mode: file_mode,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_empty_tree() {
        let cs = "".to_string();
        assert_eq!(tree_is_valid(&cs).unwrap(), true);
    }

    const PLACEHOLDER_OBJECT_ID: &str =
        "\0\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10\x11\x12\x13";

    fn entry(mode_name_str: &str) -> String {
        entry_with_object_id(mode_name_str, PLACEHOLDER_OBJECT_ID)
    }

    fn entry_with_object_id(mode_name_str: &str, object_id: &str) -> String {
        let mut r = String::new();
        r.push_str(mode_name_str);
        r.push('\0');

        assert_eq!(object_id.len(), 20);
        r.push_str(object_id);
        r
    }

    #[test]
    fn valid_tree_one_entry() {
        let cs = entry("100644 regular-file");
        assert_eq!(tree_is_valid(&cs).unwrap(), true);

        let cs = entry("100755 executable");
        assert_eq!(tree_is_valid(&cs).unwrap(), true);

        let cs = entry("40000 tree");
        assert_eq!(tree_is_valid(&cs).unwrap(), true);

        let cs = entry("120000 symlink");
        assert_eq!(tree_is_valid(&cs).unwrap(), true);

        let cs = entry("160000 git link");
        assert_eq!(tree_is_valid(&cs).unwrap(), true);

        let cs = entry("100644 .gitmodules");
        assert_eq!(tree_is_valid(&cs).unwrap(), true);
    }

    #[test]
    fn invalid_null_object_id() {
        let cs = entry_with_object_id(
            "100644 regular-file",
            "\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        );
        assert_eq!(tree_is_valid(&cs).unwrap(), false);
    }

    fn check_one_name(name: &str) {
        let mut mode_name = "100644 ".to_string();
        mode_name.push_str(name);

        let cs = entry(&mode_name);
        assert_eq!(tree_is_valid(&cs).unwrap(), true);
    }

    #[test]
    fn valid_posix_names() {
        check_one_name("a<b>c:d|e");
        check_one_name("test ");
        check_one_name("test.");
        check_one_name("NUL");
    }

    #[test]
    fn valid_sorting() {
        let mut cs = entry("100644 fooaaa");
        cs.push_str(&entry("100755 foobar"));
        assert_eq!(tree_is_valid(&cs).unwrap(), true);

        let mut cs = entry("100755 fooaaa");
        cs.push_str(&entry("100644 foobar"));
        assert_eq!(tree_is_valid(&cs).unwrap(), true);

        let mut cs = entry("40000 a");
        cs.push_str(&entry("100644 b"));
        assert_eq!(tree_is_valid(&cs).unwrap(), true);

        let mut cs = entry("100644 a");
        cs.push_str(&entry("40000 b"));
        assert_eq!(tree_is_valid(&cs).unwrap(), true);

        let mut cs = entry("100644 a.c");
        cs.push_str(&entry("40000 a"));
        cs.push_str(&entry("100644 a0c"));
        assert_eq!(tree_is_valid(&cs).unwrap(), true);

        let mut cs = entry("40000 a");
        cs.push_str(&entry("100644 apple"));
        assert_eq!(tree_is_valid(&cs).unwrap(), true);

        let mut cs = entry("40000 an orang");
        cs.push_str(&entry("40000 an orange"));
        assert_eq!(tree_is_valid(&cs).unwrap(), true);

        let mut cs = entry("100644 a");
        cs.push_str(&entry("100644 a0c"));
        cs.push_str(&entry("100644 b"));
        assert_eq!(tree_is_valid(&cs).unwrap(), true);
    }

    #[test]
    fn invalid_truncated_in_mode() {
        let cs = "1006".to_string();
        assert_eq!(tree_is_valid(&cs).unwrap(), false);
    }

    #[test]
    fn invalid_mode_starts_with_zero() {
        let cs = entry("0 a");
        assert_eq!(tree_is_valid(&cs).unwrap(), false);

        let cs = entry("0100644 a");
        assert_eq!(tree_is_valid(&cs).unwrap(), false);

        let cs = entry("040000 a");
        assert_eq!(tree_is_valid(&cs).unwrap(), false);
    }

    #[test]
    fn invalid_mode_not_octal() {
        let cs = entry("8 a");
        assert_eq!(tree_is_valid(&cs).unwrap(), false);

        let cs = entry("Z a");
        assert_eq!(tree_is_valid(&cs).unwrap(), false);
    }

    #[test]
    fn invalid_mode_not_supported() {
        let cs = entry("1 a");
        assert_eq!(tree_is_valid(&cs).unwrap(), false);

        let cs = entry("170000 a");
        assert_eq!(tree_is_valid(&cs).unwrap(), false);
    }

    #[test]
    fn invalid_name_contains_slash() {
        let cs = entry("100644 a/b");
        assert_eq!(tree_is_valid(&cs).unwrap(), false);
    }

    #[test]
    fn invalid_name_is_empty() {
        let cs = entry("100644 ");
        assert_eq!(tree_is_valid(&cs).unwrap(), false);
    }

    #[test]
    fn invalid_reserved_dot_names() {
        let cs = entry("100644 .");
        assert_eq!(tree_is_valid(&cs).unwrap(), false);

        let cs = entry("100644 ..");
        assert_eq!(tree_is_valid(&cs).unwrap(), false);

        let cs = entry("100644 .git");
        assert_eq!(tree_is_valid(&cs).unwrap(), false);

        let cs = entry("100644 .GiT");
        assert_eq!(tree_is_valid(&cs).unwrap(), false);
    }

    const MAC_HFS_GIT_NAMES: [&str; 17] = [
        ".git\u{200C}",
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

    #[test]
    fn invalid_mac_hfs_git_name() {
        for name in MAC_HFS_GIT_NAMES.iter() {
            let mut mode_name = "100644 ".to_string();
            mode_name.push_str(name);

            let cs = entry(&mode_name);
            assert_eq!(tree_is_valid(&cs).unwrap(), true);

            assert_eq!(
                tree_is_valid_with_platform_checks(
                    &cs,
                    &CheckPlatforms {
                        windows: false,
                        mac: true
                    }
                )
                .unwrap(),
                false
            );
        }
    }

    #[test]
    fn invalid_mac_hfs_git_corrupt_utf8() {
        let mut cs: Vec<u8> = Vec::new();
        cs.extend_from_slice(b"100644 .git\xEF\0\0\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10\x11\x12\x13");

        // This is fine on basic Posix.
        assert_eq!(tree_is_valid(&cs).unwrap(), true);

        // But should be rejected on Mac.
        assert_eq!(
            tree_is_valid_with_platform_checks(
                &cs,
                &CheckPlatforms {
                    windows: false,
                    mac: true
                }
            )
            .unwrap(),
            false
        );

        let mut cs: Vec<u8> = Vec::new();
        cs.extend_from_slice(b"100644 .git\xE2\xAB\0\0\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10\x11\x12\x13");

        // This is fine on basic Posix.
        assert_eq!(tree_is_valid(&cs).unwrap(), true);

        // But should be rejected on Mac.
        assert_eq!(
            tree_is_valid_with_platform_checks(
                &cs,
                &CheckPlatforms {
                    windows: false,
                    mac: true
                }
            )
            .unwrap(),
            false
        );
    }

    #[test]
    fn valid_not_mac_hfs_git() {
        let cs = entry("100644 .git\u{200C}x");
        assert_eq!(
            tree_is_valid_with_platform_checks(
                &cs,
                &CheckPlatforms {
                    windows: false,
                    mac: true
                }
            )
            .unwrap(),
            true
        );

        let cs = entry("100644 .kit\u{200C}");
        assert_eq!(
            tree_is_valid_with_platform_checks(
                &cs,
                &CheckPlatforms {
                    windows: false,
                    mac: true
                }
            )
            .unwrap(),
            true
        );
    }

    const GIT_RESERVED_NAMES: [&str; 11] = [
        ".", "..", ".git", ".git.", ".git ", ".git. ", ".git . ", ".Git", ".gIt", ".giT", ".giT.",
    ];

    #[test]
    fn invalid_reserved_names() {
        for name in GIT_RESERVED_NAMES.iter() {
            let mut mode_name = "100644 ".to_string();
            mode_name.push_str(name);

            let cs = entry(&mode_name);
            assert_eq!(tree_is_valid(&cs).unwrap(), false);
        }
    }

    const ALMOST_GIT_RESERVED_NAMES: [&str; 5] = [
        ".git..",
        ".gitfoobar",
        ".git foobar",
        ".gitfoo bar",
        ".gitfoobar..",
    ];

    #[test]
    fn valid_almost_dot_git() {
        for name in ALMOST_GIT_RESERVED_NAMES.iter() {
            let mut mode_name = "100644 ".to_string();
            mode_name.push_str(name);

            let cs = entry(&mode_name);
            assert_eq!(tree_is_valid(&cs).unwrap(), true);
        }
    }

    const BAD_DOT_GIT_TILDE_NAMES: [&str; 2] = ["GIT~1", "GiT~1"];

    #[test]
    fn invalid_git_tilde() {
        for name in BAD_DOT_GIT_TILDE_NAMES.iter() {
            let mut mode_name = "100644 ".to_string();
            mode_name.push_str(name);

            let cs = entry(&mode_name);
            assert_eq!(tree_is_valid(&cs).unwrap(), false);
        }
    }

    #[test]
    fn invalid_git_tilde_11() {
        check_one_name("GIT~11");
    }

    #[test]
    fn invalid_tree_missing_null_terminator() {
        let cs = "100644 b".to_string();
        assert_eq!(tree_is_valid(&cs).unwrap(), false);
    }

    #[test]
    fn invalid_tree_partial_object_id() {
        let cs = "100644 b\0\x01\x02".to_string();
        assert_eq!(tree_is_valid(&cs).unwrap(), false);
    }

    fn quick_tree(mode_name_1: &str, mode_name_2: &str) -> String {
        let mut s = String::new();
        s.push_str(&entry(&mode_name_1));
        s.push_str(&entry(&mode_name_2));
        s
    }

    #[test]
    fn invalid_bad_sorting() {
        let cs = quick_tree("100644 foobar", "100644 fooaaa");
        assert_eq!(tree_is_valid(&cs).unwrap(), false);

        let cs = quick_tree("40000 a", "100644 a.c");
        assert_eq!(tree_is_valid(&cs).unwrap(), false);

        let cs = quick_tree("100644 a0c", "40000 a");
        assert_eq!(tree_is_valid(&cs).unwrap(), false);
    }

    #[test]
    fn invalid_duplicate_names() {
        let cs = quick_tree("100644 a", "100644 a");
        assert_eq!(tree_is_valid(&cs).unwrap(), false);

        let cs = quick_tree("40000 a", "40000 a");
        assert_eq!(tree_is_valid(&cs).unwrap(), false);

        let cs = quick_tree("100644 a", "100755 a");
        assert_eq!(tree_is_valid(&cs).unwrap(), false);

        let cs = quick_tree("100644 a", "40000 a");
        assert_eq!(tree_is_valid(&cs).unwrap(), false);
    }

    #[test]
    fn invalid_duplicate_names_case_insensitive() {
        let mut cs = String::new();
        cs.push_str(&entry("100644 A"));
        cs.push_str(&entry("100644 A.c"));
        cs.push_str(&entry("100644 A.d"));
        cs.push_str(&entry("100644 A.e"));
        cs.push_str(&entry("40000 a"));

        assert_eq!(
            tree_is_valid_with_platform_checks(
                &cs,
                &CheckPlatforms {
                    windows: true,
                    mac: false
                }
            )
            .unwrap(),
            false
        );

        assert_eq!(
            tree_is_valid_with_platform_checks(
                &cs,
                &CheckPlatforms {
                    windows: false,
                    mac: true
                }
            )
            .unwrap(),
            false
        );
    }

    #[test]
    fn invalid_tree_sorting() {
        let mut s = String::new();
        s.push_str(&entry("100644 a"));
        s.push_str(&entry("100644 a.c"));
        s.push_str(&entry("100644 a.d"));
        s.push_str(&entry("100644 a.e"));
        s.push_str(&entry("40000 a"));
        s.push_str(&entry("100644 zoo"));

        assert_eq!(tree_is_valid(&s).unwrap(), false);
    }

    #[test]
    fn invalid_tree_sorting_2() {
        let mut s = String::new();
        s.push_str(&entry("100644 a"));
        s.push_str(&entry("100644 a.c"));
        s.push_str(&entry("100644 a.d"));
        s.push_str(&entry("100644 a.d.b"));
        s.push_str(&entry("100644 a.d.x"));
        s.push_str(&entry("40000 a.d"));
        s.push_str(&entry("100644 a.e"));
        s.push_str(&entry("40000 b"));
        s.push_str(&entry("100644 zoo"));

        assert_eq!(tree_is_valid(&s).unwrap(), false);
    }

    #[test]
    fn invalid_tree_sorting_3() {
        let mut s = String::new();
        s.push_str(&entry("100644 a"));
        s.push_str(&entry("100644 a.c"));
        s.push_str(&entry("100644 a.d"));
        s.push_str(&entry("100644 a.d.b"));
        s.push_str(&entry("100644 a.d.x"));
        s.push_str(&entry("40000 a.d.y"));
        s.push_str(&entry("100644 a.e"));
        s.push_str(&entry("40000 a"));
        s.push_str(&entry("100644 zoo"));

        assert_eq!(tree_is_valid(&s).unwrap(), false);
    }

    #[test]
    fn case_folding() {
        let cs = quick_tree("100644 A", "100644 a");
        assert_eq!(tree_is_valid(&cs).unwrap(), true);

        assert_eq!(
            tree_is_valid_with_platform_checks(
                &cs,
                &CheckPlatforms {
                    windows: true,
                    mac: false
                }
            )
            .unwrap(),
            false
        );

        assert_eq!(
            tree_is_valid_with_platform_checks(
                &cs,
                &CheckPlatforms {
                    windows: false,
                    mac: true
                }
            )
            .unwrap(),
            false
        );
    }

    #[test]
    fn invalid_mac_denormalized_names() {
        let cs = quick_tree("100644 \u{0065}\u{0301}", "100644 \u{00e9}");
        assert_eq!(tree_is_valid(&cs).unwrap(), true);

        assert_eq!(
            tree_is_valid_with_platform_checks(
                &cs,
                &CheckPlatforms {
                    windows: true,
                    mac: false
                }
            )
            .unwrap(),
            true
        );

        assert_eq!(
            tree_is_valid_with_platform_checks(
                &cs,
                &CheckPlatforms {
                    windows: false,
                    mac: true
                }
            )
            .unwrap(),
            false
        );
    }

    #[test]
    fn windows_space_at_end_of_name() {
        let cs = entry(&"100644 test ".to_string());
        assert_eq!(tree_is_valid(&cs).unwrap(), true);

        assert_eq!(
            tree_is_valid_with_platform_checks(
                &cs,
                &CheckPlatforms {
                    windows: true,
                    mac: false
                }
            )
            .unwrap(),
            false
        );

        assert_eq!(
            tree_is_valid_with_platform_checks(
                &cs,
                &CheckPlatforms {
                    windows: false,
                    mac: true
                }
            )
            .unwrap(),
            true
        );
    }

    #[test]
    fn windows_dot_at_end_of_name() {
        let cs = entry(&"100644 test.".to_string());
        assert_eq!(tree_is_valid(&cs).unwrap(), true);

        assert_eq!(
            tree_is_valid_with_platform_checks(
                &cs,
                &CheckPlatforms {
                    windows: true,
                    mac: false
                }
            )
            .unwrap(),
            false
        );

        assert_eq!(
            tree_is_valid_with_platform_checks(
                &cs,
                &CheckPlatforms {
                    windows: false,
                    mac: true
                }
            )
            .unwrap(),
            true
        );
    }

    const WINDOWS_DEVICE_NAMES: [&str; 22] = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];

    #[test]
    fn invalid_device_names_on_windows() {
        for name in WINDOWS_DEVICE_NAMES.iter() {
            let mut mode_name = "100644 ".to_string();
            mode_name.push_str(name);

            let cs = entry(&mode_name);
            assert_eq!(tree_is_valid(&cs).unwrap(), true);

            assert_eq!(
                tree_is_valid_with_platform_checks(
                    &cs,
                    &CheckPlatforms {
                        windows: true,
                        mac: false
                    }
                )
                .unwrap(),
                false
            );

            assert_eq!(
                tree_is_valid_with_platform_checks(
                    &cs,
                    &CheckPlatforms {
                        windows: false,
                        mac: true
                    }
                )
                .unwrap(),
                true
            );
        }
    }

    const INVALID_WINDOWS_CHARS: [u8; 39] = [
        b'<', b'>', b':', b'\"', b'\\', b'|', b'?', b'*', 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12,
        13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
    ];

    #[test]
    fn invalid_characters_on_windows() {
        for c in INVALID_WINDOWS_CHARS.iter() {
            let mut mode_name = "100644 te".to_string();
            mode_name.push(*c as char);
            mode_name.push_str(&"st");

            let cs = entry(&mode_name);
            assert_eq!(tree_is_valid(&cs).unwrap(), true);

            assert_eq!(
                tree_is_valid_with_platform_checks(
                    &cs,
                    &CheckPlatforms {
                        windows: true,
                        mac: false
                    }
                )
                .unwrap(),
                false
            );

            assert_eq!(
                tree_is_valid_with_platform_checks(
                    &cs,
                    &CheckPlatforms {
                        windows: false,
                        mac: true
                    }
                )
                .unwrap(),
                true
            );
        }
    }
}
