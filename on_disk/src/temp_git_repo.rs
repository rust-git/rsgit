use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

/// A `TempGitRepo` creates a temporary, empty repo using
/// the command-line git from the host system. This is often
/// used in unit tests to compare output with comparable
/// rsgit operations.
///
/// Because this struct is intended for testing, its functions
/// panic instead of returning Result structs.
#[derive(Default)]
pub struct TempGitRepo {
    #[allow(dead_code)] // tempdir is only used for RAII
    tempdir: Option<tempfile::TempDir>,
    path: PathBuf,
}

impl TempGitRepo {
    // Create a new, sanitized repo in a temporary directory.
    // This directory will be deleted when the struct is dropped.
    pub fn new() -> TempGitRepo {
        let tempdir = tempfile::tempdir().unwrap();
        let path: PathBuf = tempdir.path().to_path_buf();

        let mut r = TempGitRepo {
            tempdir: Some(tempdir),
            path,
        };

        r.init();
        r
    }

    // Create a new, sanitized repo in the specified location.
    // WARNING: This will erase any content already at that path.
    // Use this only when you need to manually inspect the results
    // of the test run.
    pub fn new_at_path<P: Into<PathBuf>>(p: P) -> TempGitRepo {
        let path = p.into();
        fs::remove_dir_all(&path).unwrap_or(());
        fs::create_dir_all(&path).unwrap();

        let mut r = TempGitRepo {
            tempdir: None,
            path,
        };

        r.init();
        r
    }

    fn init(&mut self) {
        self.git_command(&["init"]);

        // Some older versions of git create a branches directory, but it's
        // considered deprecated. We'll remove it so folder comparisons are canonical.
        // Don't worry if it doesn't exist.

        let branches_dir = self.path.join(".git/branches");
        fs::remove_dir_all(&branches_dir).unwrap_or(());

        // Some things change too much from one version to another of git.
        // Rewrite to a canonical version so we can test against rsgit's output.

        // Clean out the hooks directory. The samples aren't essential.

        let hooks_dir = self.path.join(".git/hooks");
        fs::remove_dir_all(&hooks_dir).unwrap_or(());
        fs::create_dir_all(&hooks_dir).unwrap();

        let git_config_txt = "[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n\tlogallrefupdates = true\n";

        let git_config_path = self.path.join(".git/config");
        fs::write(git_config_path, git_config_txt).unwrap();

        let git_info_exclude_txt = "# git ls-files --others --exclude-from=.git/info/exclude\n# Lines that start with '#' are comments.\n# For a project mostly in C, the following would be a good set of\n# exclude patterns (uncomment them if you want to use them):\n# *.[oa]\n# *~\n.DS_Store\n";

        let git_info_exclude_path = self.path.join(".git/info/exclude");
        fs::write(git_info_exclude_path, git_info_exclude_txt).unwrap();
    }

    // Return the path for this repo's root (working directory).
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    // Create a command struct pointing to the root of the repo.
    pub fn command<S: AsRef<OsStr>>(&mut self, program: S) -> Command {
        let mut c = Command::new(program);
        c.current_dir(&self.path);
        c
    }

    // Run a git command and return the git repo struct for method chaining.
    // Since this is used primarily for testing purposes, panics if command fails.
    pub fn git_command<I, S>(&mut self, args: I) -> &mut TempGitRepo
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let output = self.command("git").args(args).output().unwrap();

        if !output.status.success() {
            panic!(
                "git command failed with status {:?}\n\nstdout:\n\n{}\n\nstderr:\n\n{}\n\n",
                output.status.code(),
                std::str::from_utf8(&output.stdout).unwrap(),
                std::str::from_utf8(&output.stderr).unwrap()
            );
        }

        self
    }
}

#[cfg(test)]
mod tests {
    use super::TempGitRepo;

    #[test]
    fn temp_path() {
        let path = {
            let mut r = TempGitRepo::new();
            r.git_command(&["status"]);

            let path = r.path().to_path_buf();

            let git_dir = path.join(".git");
            assert_eq!(git_dir.is_dir(), true);

            path
        };

        assert_eq!(path.as_path().is_dir(), false);
    }

    #[test]
    fn at_specific_path() {
        let temp_dir = tempfile::tempdir().unwrap();
        let repo_dir = temp_dir.into_path().join("tgr");

        assert_eq!(repo_dir.is_dir(), false);

        {
            let _r = TempGitRepo::new_at_path(&repo_dir);

            let git_dir = repo_dir.join(".git");
            assert_eq!(git_dir.is_dir(), true);
        }

        // This should be left behind for post-test inspection.
        // (Except that, in this case, because we used tempfile::tempdir()
        // behind TGR's back, it will be deleted at end of test.)

        assert_eq!(repo_dir.is_dir(), true);
    }

    #[test]
    #[should_panic(expected = "git command failed with status")]
    fn git_command_error() {
        let mut r = TempGitRepo::new();
        r.git_command(&["bogus"]);
    }
}
