use std::fs;
use std::io::{self, Result};
use std::path::Path;

/// Creates a new, empty git repository on the local file system.
///
/// Analogous to [`git init`](https://git-scm.com/docs/git-init).
pub fn init(work_dir: &Path) -> Result<()> {
    let git_dir = work_dir.join(".git");
    if git_dir.exists() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "work_dir/.git should not exist",
        ));
    }

    fs::create_dir_all(&git_dir)?;

    create_config(&git_dir)?;
    create_description(&git_dir)?;
    create_head(&git_dir)?;
    create_hooks_dir(&git_dir)?;
    create_info_dir(&git_dir)?;
    create_objects_dir(&git_dir)?;
    create_refs_dir(&git_dir)?;

    Ok(())
}

fn create_config(git_dir: &Path) -> Result<()> {
    let config_path = git_dir.join("config");
    let config_txt = "[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n\tlogallrefupdates = true\n";

    fs::write(config_path, config_txt)
}

fn create_description(git_dir: &Path) -> Result<()> {
    let desc_path = git_dir.join("description");
    let desc_txt = "Unnamed repository; edit this file 'description' to name the repository.\n";

    fs::write(desc_path, desc_txt)
}

fn create_head(git_dir: &Path) -> Result<()> {
    let head_path = git_dir.join("HEAD");
    let head_txt = "ref: refs/heads/master\n";

    fs::write(head_path, head_txt)
}

fn create_hooks_dir(git_dir: &Path) -> Result<()> {
    let hooks_dir = git_dir.join("hooks");
    fs::create_dir_all(&hooks_dir)

    // NOTE: Intentionally not including the sample files.
}

fn create_info_dir(git_dir: &Path) -> Result<()> {
    let info_dir = git_dir.join("info");
    fs::create_dir_all(&info_dir)?;

    let exclude_path = info_dir.join("exclude");
    let exclude_txt = "# git ls-files --others --exclude-from=.git/info/exclude\n# Lines that start with '#' are comments.\n# For a project mostly in C, the following would be a good set of\n# exclude patterns (uncomment them if you want to use them):\n# *.[oa]\n# *~\n.DS_Store\n";

    fs::write(exclude_path, exclude_txt)
}

fn create_objects_dir(git_dir: &Path) -> Result<()> {
    let info_dir = git_dir.join("objects/info");
    fs::create_dir_all(&info_dir)?;

    let pack_dir = git_dir.join("objects/pack");
    fs::create_dir_all(&pack_dir)
}

fn create_refs_dir(git_dir: &Path) -> Result<()> {
    let heads_dir = git_dir.join("refs/heads");
    fs::create_dir_all(&heads_dir)?;

    let tags_dir = git_dir.join("refs/tags");
    fs::create_dir_all(&tags_dir)
}

#[cfg(test)]
mod tests {
    extern crate dir_diff;
    extern crate tempfile;

    use std::fs;
    use std::io;

    use crate::test_support::TempGitRepo;

    #[test]
    fn matches_command_line_git() {
        let tgr = TempGitRepo::new();
        let c_path = tgr.path();

        let r_path = tempfile::tempdir().unwrap();
        super::init(r_path.path()).unwrap();

        assert_eq!(
            dir_diff::is_different(c_path, r_path.path()).unwrap(),
            false
        );
    }

    #[test]
    fn err_if_git_dir_exists() {
        let r_path = tempfile::tempdir().unwrap();
        let git_dir = r_path.path().join(".git");
        fs::create_dir_all(&git_dir).unwrap();

        let err = super::init(r_path.path()).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::Other);
        assert_eq!(format!("{}", err), "work_dir/.git should not exist");
    }
}
