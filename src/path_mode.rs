use std::cmp::{self, Ordering};

use crate::file_mode::FileMode;

/// Represents the tuple of git path (an uninterpreted sequence of bytes,
/// not necessarily UTF-8) and git file mode. Used for comparisons.
#[derive(Eq, PartialEq)]
pub struct PathMode<'a> {
    pub path: &'a [u8],
    pub mode: FileMode,
}

impl<'a> PathMode<'a> {
    /// Compare two paths, checking for identical name.
    ///
    /// Unlike `cmp()`, this function returns `Equal` when the paths have
    /// the same characters in their names, even if the mode differs. It is
    /// intended for use in validation routines detecting duplicate entries.
    ///
    /// Trees are sorted as though the last character is `/`, even if
    /// no such character exists.
    ///
    /// ### Return Value
    ///
    /// * `Less` if no duplicate name could exist. (All possible occurrences
    ///   of `self` sort before `other` and no conflict can happen.)
    /// * `Equal` if the paths have the same name. (A conflict exists
    ///   between the two paths for this reason.)
    /// * `Greater` if `other`'s path should still be checked by caller.
    ///   (It is possible for a duplicate occurrence of `self` to appear
    ///   later, after `other`. Callers should continue to examine candidates
    ///   for `other` until the method returns one of the other return values.
    pub fn cmp_same_name(&self, other: &PathMode) -> Ordering {
        let self_as_tree = PathMode {
            path: &self.path,
            mode: FileMode::Tree,
        };
        core_compare(&self_as_tree, other)
    }
}

impl<'a> Ord for PathMode<'a> {
    fn cmp(&self, other: &PathMode) -> Ordering {
        match core_compare(&self, &other) {
            Ordering::Equal => mode_compare(self.mode, other.mode),
            x => x,
        }
    }
}

impl<'a> PartialOrd for PathMode<'a> {
    fn partial_cmp(&self, other: &PathMode) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn core_compare(left: &PathMode, right: &PathMode) -> Ordering {
    if left.path == right.path {
        Ordering::Equal
    } else {
        let l = cmp::min(left.path.len(), right.path.len());

        let lprefix = &left.path[..l];
        let rprefix = &right.path[..l];

        match lprefix.cmp(rprefix) {
            Ordering::Equal => (),
            non_eq => return non_eq,
        }

        let lsuffix = suffix_or_slash(&left.path[l..], left.mode);
        let rsuffix = suffix_or_slash(&right.path[l..], right.mode);

        lsuffix.cmp(rsuffix)
    }
}

const EMPTY: [u8; 0] = [];
const SLASH: [u8; 1] = [47];

fn suffix_or_slash(suffix: &[u8], mode: FileMode) -> &[u8] {
    if suffix.len() > 0 || mode != FileMode::Tree {
        suffix
    } else {
        &SLASH
    }
}

fn mode_compare(m1: FileMode, m2: FileMode) -> Ordering {
    if m1 == FileMode::Submodule || m2 == FileMode::Submodule {
        Ordering::Equal
    } else {
        let lsuffix = suffix_or_slash(&EMPTY, m1);
        let rsuffix = suffix_or_slash(&EMPTY, m2);
        lsuffix.cmp(rsuffix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmp_simple_case() {
        let l = PathMode {
            path: b"abc",
            mode: FileMode::Normal,
        };
        let r = PathMode {
            path: b"def",
            mode: FileMode::Normal,
        };
        assert_eq!(l.cmp(&r), Ordering::Less);

        let r = PathMode {
            path: b"aba",
            mode: FileMode::Normal,
        };
        assert_eq!(l.cmp(&r), Ordering::Greater);
    }

    #[test]
    fn cmp_lengths_mismatch() {
        let l = PathMode {
            path: b"abc",
            mode: FileMode::Normal,
        };
        let r = PathMode {
            path: b"ab",
            mode: FileMode::Normal,
        };
        assert_eq!(l.cmp(&r), Ordering::Greater);

        let l = PathMode {
            path: b"ab",
            mode: FileMode::Normal,
        };
        let r = PathMode {
            path: b"aba",
            mode: FileMode::Normal,
        };
        assert_eq!(l.cmp(&r), Ordering::Less);
    }

    #[test]
    fn cmp_implied_tree() {
        let l = PathMode {
            path: b"ab/",
            mode: FileMode::Tree,
        };
        let r = PathMode {
            path: b"ab",
            mode: FileMode::Tree,
        };
        assert_eq!(l.cmp(&r), Ordering::Equal);

        let l = PathMode {
            path: b"ab",
            mode: FileMode::Tree,
        };
        let r = PathMode {
            path: b"ab/",
            mode: FileMode::Tree,
        };
        assert_eq!(l.cmp(&r), Ordering::Equal);
    }

    #[test]
    fn cmp_exact_match() {
        let l = PathMode {
            path: b"abc",
            mode: FileMode::Normal,
        };
        let r = PathMode {
            path: b"abc",
            mode: FileMode::Normal,
        };
        assert_eq!(l.cmp(&r), Ordering::Equal);
    }

    #[test]
    fn cmp_match_except_file_mode() {
        let l = PathMode {
            path: b"abc",
            mode: FileMode::Tree,
        };
        let r = PathMode {
            path: b"abc",
            mode: FileMode::Normal,
        };
        assert_eq!(l.cmp(&r), Ordering::Greater);

        let l = PathMode {
            path: b"abc",
            mode: FileMode::Normal,
        };
        let r = PathMode {
            path: b"abc",
            mode: FileMode::Tree,
        };
        assert_eq!(l.cmp(&r), Ordering::Less);
    }

    #[test]
    fn cmp_gitlink_exception() {
        let l = PathMode {
            path: b"abc",
            mode: FileMode::Tree,
        };
        let r = PathMode {
            path: b"abc",
            mode: FileMode::Submodule,
        };
        assert_eq!(l.cmp(&r), Ordering::Equal);

        let l = PathMode {
            path: b"abc",
            mode: FileMode::Submodule,
        };
        let r = PathMode {
            path: b"abc",
            mode: FileMode::Tree,
        };
        assert_eq!(l.cmp(&r), Ordering::Equal);
    }

    #[test]
    fn partial_cmp_simple_case() {
        let l = PathMode {
            path: b"abc",
            mode: FileMode::Normal,
        };
        let r = PathMode {
            path: b"def",
            mode: FileMode::Normal,
        };
        assert_eq!(l.partial_cmp(&r).unwrap(), Ordering::Less);

        let r = PathMode {
            path: b"aba",
            mode: FileMode::Normal,
        };
        assert_eq!(l.partial_cmp(&r).unwrap(), Ordering::Greater);
    }

    #[test]
    fn partial_cmp_lengths_mismatch() {
        let l = PathMode {
            path: b"abc",
            mode: FileMode::Normal,
        };
        let r = PathMode {
            path: b"ab",
            mode: FileMode::Normal,
        };
        assert_eq!(l.partial_cmp(&r).unwrap(), Ordering::Greater);

        let l = PathMode {
            path: b"ab",
            mode: FileMode::Normal,
        };
        let r = PathMode {
            path: b"aba",
            mode: FileMode::Normal,
        };
        assert_eq!(l.partial_cmp(&r).unwrap(), Ordering::Less);
    }

    #[test]
    fn partial_cmp_implied_tree() {
        let l = PathMode {
            path: b"ab/",
            mode: FileMode::Tree,
        };
        let r = PathMode {
            path: b"ab",
            mode: FileMode::Tree,
        };
        assert_eq!(l.partial_cmp(&r).unwrap(), Ordering::Equal);

        let l = PathMode {
            path: b"ab",
            mode: FileMode::Tree,
        };
        let r = PathMode {
            path: b"ab/",
            mode: FileMode::Tree,
        };
        assert_eq!(l.partial_cmp(&r).unwrap(), Ordering::Equal);
    }

    #[test]
    fn partial_cmp_exact_match() {
        let l = PathMode {
            path: b"abc",
            mode: FileMode::Normal,
        };
        let r = PathMode {
            path: b"abc",
            mode: FileMode::Normal,
        };
        assert_eq!(l.partial_cmp(&r).unwrap(), Ordering::Equal);
    }

    #[test]
    fn partial_cmp_match_except_file_mode() {
        let l = PathMode {
            path: b"abc",
            mode: FileMode::Tree,
        };
        let r = PathMode {
            path: b"abc",
            mode: FileMode::Normal,
        };
        assert_eq!(l.partial_cmp(&r).unwrap(), Ordering::Greater);

        let l = PathMode {
            path: b"abc",
            mode: FileMode::Normal,
        };
        let r = PathMode {
            path: b"abc",
            mode: FileMode::Tree,
        };
        assert_eq!(l.partial_cmp(&r).unwrap(), Ordering::Less);
    }

    #[test]
    fn partial_cmp_gitlink_exception() {
        let l = PathMode {
            path: b"abc",
            mode: FileMode::Tree,
        };
        let r = PathMode {
            path: b"abc",
            mode: FileMode::Submodule,
        };
        assert_eq!(l.partial_cmp(&r).unwrap(), Ordering::Equal);

        let l = PathMode {
            path: b"abc",
            mode: FileMode::Submodule,
        };
        let r = PathMode {
            path: b"abc",
            mode: FileMode::Tree,
        };
        assert_eq!(l.partial_cmp(&r).unwrap(), Ordering::Equal);
    }

    #[test]
    fn cmp_same_name_simple_case() {
        let l = PathMode {
            path: b"abc",
            mode: FileMode::Normal,
        };
        let r = PathMode {
            path: b"def",
            mode: FileMode::Normal,
        };
        assert_eq!(l.cmp_same_name(&r), Ordering::Less);

        let r = PathMode {
            path: b"aba",
            mode: FileMode::Normal,
        };
        assert_eq!(l.cmp_same_name(&r), Ordering::Greater);
    }

    #[test]
    fn cmp_same_name_lengths_mismatch() {
        let l = PathMode {
            path: b"abc",
            mode: FileMode::Normal,
        };
        let r = PathMode {
            path: b"ab",
            mode: FileMode::Normal,
        };
        assert_eq!(l.cmp_same_name(&r), Ordering::Greater);

        let l = PathMode {
            path: b"ab",
            mode: FileMode::Normal,
        };
        let r = PathMode {
            path: b"aba",
            mode: FileMode::Normal,
        };
        assert_eq!(l.cmp_same_name(&r), Ordering::Less);
    }

    #[test]
    fn cmp_same_name_implied_tree() {
        let l = PathMode {
            path: b"ab/",
            mode: FileMode::Tree,
        };
        let r = PathMode {
            path: b"ab",
            mode: FileMode::Tree,
        };
        assert_eq!(l.cmp_same_name(&r), Ordering::Equal);

        let l = PathMode {
            path: b"ab",
            mode: FileMode::Tree,
        };
        let r = PathMode {
            path: b"ab/",
            mode: FileMode::Tree,
        };
        assert_eq!(l.cmp_same_name(&r), Ordering::Equal);
    }

    #[test]
    fn cmp_same_name_match_except_file_mode() {
        let l = PathMode {
            path: b"abc",
            mode: FileMode::Tree,
        };
        let r = PathMode {
            path: b"abc",
            mode: FileMode::Normal,
        };
        assert_eq!(l.cmp_same_name(&r), Ordering::Equal);

        let l = PathMode {
            path: b"abc",
            mode: FileMode::Normal,
        };
        let r = PathMode {
            path: b"abc",
            mode: FileMode::Tree,
        };
        assert_eq!(l.cmp_same_name(&r), Ordering::Equal);
    }

    #[test]
    fn cmp_same_name_exact_match() {
        let l = PathMode {
            path: b"abc",
            mode: FileMode::Tree,
        };
        let r = PathMode {
            path: b"abc",
            mode: FileMode::Tree,
        };
        assert_eq!(l.cmp_same_name(&r), Ordering::Equal);
    }

    #[test]
    fn cmp_same_name_gitlink_exception() {
        let l = PathMode {
            path: b"abc",
            mode: FileMode::Tree,
        };
        let r = PathMode {
            path: b"abc",
            mode: FileMode::Submodule,
        };
        assert_eq!(l.cmp_same_name(&r), Ordering::Equal);

        let l = PathMode {
            path: b"abc",
            mode: FileMode::Submodule,
        };
        let r = PathMode {
            path: b"abc",
            mode: FileMode::Tree,
        };
        assert_eq!(l.cmp_same_name(&r), Ordering::Equal);
    }
}
