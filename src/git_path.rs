use std::result::Result;

/// Represents a list of bytes (typically, but not necessarily UTF-8)
/// that is a valid path in a git repo.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitPath<'a> {
    path: &'a [u8],
    // windows: bool,
    // mac: bool,
}

/// Reasons why a given byte sequence can not be accepted as a git repo path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GitPathError {
    EmptyPath,
    AbsolutePath,
    TrailingSlash,
    DuplicateSlash,
    ContainsNull,
}

impl<'a> GitPath<'a> {
    /// Convert the provided byte vector to a `GitPath` struct if it is acceptable
    /// as a git path. The rules enforced here are slightly different from what
    /// is allowed in a `tree` object in that we allow `/` characters to build
    /// hierarchical paths.
    pub fn new(path: &'a [u8]) -> Result<GitPath<'a>, GitPathError> {
        GitPath::new_with_platform_checks(path, false, false)
    }

    /// Convert the provided byte vector to a `GitPath` struct if it is acceptable
    /// as a git path. In addition to the typical constraints enforced via `new()`,
    /// also check platform-specific rules.
    pub fn new_with_platform_checks(
        path: &'a [u8],
        windows: bool,
        mac: bool,
    ) -> Result<GitPath<'a>, GitPathError> {
        match check_path(path, windows, mac) {
            Ok(()) => Ok(GitPath { path }),
            Err(err) => Err(err),
        }
    }

    // Return the path.
    pub fn path(&self) -> &[u8] {
        self.path
    }
}

fn check_path(path: &[u8], windows: bool, mac: bool) -> Result<(), GitPathError> {
    if path.is_empty() {
        Err(GitPathError::EmptyPath)
    } else if path.starts_with(b"/") {
        Err(GitPathError::AbsolutePath)
    } else if path.ends_with(b"/") {
        Err(GitPathError::TrailingSlash)
    } else {
        for segment in path.split(|c| *c == 47) {
            match check_segment(segment, windows, mac) {
                Err(GitPathError::EmptyPath) => Err(GitPathError::DuplicateSlash),
                x => x,
            }?;
        }
        Ok(())
    }
}

fn check_segment(segment: &[u8], _windows: bool, _mac: bool) -> Result<(), GitPathError> {
    if segment.is_empty() {
        Err(GitPathError::EmptyPath)
    } else if segment.contains(&0) {
        Err(GitPathError::ContainsNull)
    } else {
        // TO DO: Way more to check here.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_case() {
        // No platform-specific checks.
        assert_eq!(GitPath::new(b"").unwrap_err(), GitPathError::EmptyPath);

        let a = GitPath::new(b"a").unwrap();
        assert_eq!(a.path(), b"a");

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
}
