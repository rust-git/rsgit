/// Describes the file type as represented on disk.
///
/// Git uses a variation on the Unix file permissions flags to denote a file's
/// intended type on disk. The following values are recognized:
///
/// * `0o100644` - normal file
/// * `0o100755` - executable file
/// * `0o120000` - symbolic link
/// * `0o040000` - tree (subdirectory)
/// * `0o160000` - submodule (aka gitlink)
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FileMode {
    Normal,
    Executable,
    SymbolicLink,
    Tree,
    Submodule,
}

impl FileMode {
    /// Convert from git file-mode integer to `FileMode` enum.
    ///
    /// Returns `None` if the value is not one of the recognized constants.
    pub fn from_value(value: u32) -> Option<FileMode> {
        match value {
            0o100644 => Some(FileMode::Normal),
            0o100755 => Some(FileMode::Executable),
            0o120000 => Some(FileMode::SymbolicLink),
            0o040000 => Some(FileMode::Tree),
            0o160000 => Some(FileMode::Submodule),
            _ => None,
        }
    }

    /// Convert from `FileMode` enum to git file-mode integer.
    pub fn to_value(self) -> u32 {
        match self {
            FileMode::Normal => 0o100644,
            FileMode::Executable => 0o100755,
            FileMode::SymbolicLink => 0o120000,
            FileMode::Tree => 0o040000,
            FileMode::Submodule => 0o160000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_value() {
        assert_eq!(FileMode::from_value(0o100644).unwrap(), FileMode::Normal);
        assert_eq!(
            FileMode::from_value(0o100755).unwrap(),
            FileMode::Executable
        );
        assert_eq!(
            FileMode::from_value(0o120000).unwrap(),
            FileMode::SymbolicLink
        );
        assert_eq!(FileMode::from_value(0o040000).unwrap(), FileMode::Tree);
        assert_eq!(FileMode::from_value(0o160000).unwrap(), FileMode::Submodule);
        assert!(FileMode::from_value(0o160001).is_none());
        assert!(FileMode::from_value(0).is_none());
        assert!(FileMode::from_value(0x100643).is_none());
    }

    #[test]
    fn to_value() {
        assert_eq!(FileMode::to_value(FileMode::Normal), 0o100644);
        assert_eq!(FileMode::to_value(FileMode::Executable), 0o100755);
        assert_eq!(FileMode::to_value(FileMode::SymbolicLink), 0o120000);
        assert_eq!(FileMode::to_value(FileMode::Tree), 0o040000);
        assert_eq!(FileMode::to_value(FileMode::Submodule), 0o160000);
    }
}
