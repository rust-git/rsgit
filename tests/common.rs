use std::{env::set_current_dir, ffi::OsStr, fs, path::Path};

use assert_cmd::cargo;

type GitOp = fn(&OsStr, &Path);

#[allow(dead_code)]
pub fn compare_git_and_rsgit(op: GitOp) {
    let c_temp = tempfile::tempdir().unwrap();
    let c_dir = c_temp.path();
    sanitize_repo(c_dir);

    let cgit = OsStr::new("git");
    set_current_dir(c_dir).unwrap();
    op(&cgit, &c_dir);

    let r_temp = tempfile::tempdir().unwrap();
    let r_dir = r_temp.path();
    let rsgit = cargo::cargo_bin("rsgit");
    let rsgit = rsgit.as_os_str();
    set_current_dir(r_dir).unwrap();
    op(rsgit, &r_dir);

    assert!(!dir_diff::is_different(c_dir, r_dir).unwrap());
}

#[allow(dead_code)]
pub fn compare_git_and_rsgit_in(op: GitOp, path: &str) {
    // Use this when a test fails.
    // Set `path` to some common directory (~/Desktop, for example)
    // and then you can diff the `cgit` and `rsgit` directories in that directory
    // to understand the failure.

    let path = Path::new(path);

    let c_dir = path.join("cgit");
    let c_dir = c_dir.as_path();
    if c_dir.exists() {
        fs::remove_dir_all(&c_dir).unwrap();
    }
    fs::create_dir_all(&c_dir).unwrap();
    sanitize_repo(&c_dir);

    let cgit = OsStr::new("git");
    set_current_dir(&c_dir).unwrap();
    op(&cgit, &c_dir);

    let r_dir = path.join("rsgit");
    let r_dir = r_dir.as_path();
    if r_dir.exists() {
        fs::remove_dir_all(&r_dir).unwrap();
    }
    fs::create_dir_all(&r_dir).unwrap();
    let rsgit = cargo::cargo_bin("rsgit");
    let rsgit = rsgit.as_os_str();
    set_current_dir(&r_dir).unwrap();
    op(rsgit, &r_dir);

    if dir_diff::is_different(c_dir, r_dir).unwrap() {
        panic!(
            "Directories differ:\n\n  c git: {}\n  rsgit: {}\n\n",
            c_dir.display(),
            r_dir.display()
        );
    }
}

pub fn sanitize_repo(path: &Path) {
    // Some older versions of git create a branches directory, but it's
    // considered deprecated. We'll remove it so folder comparisons are canonical.
    // Don't worry if it doesn't exist.

    let branches_dir = path.join(".git/branches");
    fs::remove_dir_all(&branches_dir).unwrap_or(());

    // Some things change too much from one version to another of git.
    // Rewrite to a canonical version so we can test against rsgit's output.

    // Clean out the hooks directory. The samples aren't essential.

    let hooks_dir = path.join(".git/hooks");
    fs::remove_dir_all(&hooks_dir).unwrap_or(());
    fs::create_dir_all(&hooks_dir).unwrap();

    let git_config_txt = "[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n\tlogallrefupdates = true\n";

    let git_config_path = path.join(".git/config");
    fs::write(git_config_path, git_config_txt).unwrap();

    let git_info_exclude_txt = "# git ls-files --others --exclude-from=.git/info/exclude\n# Lines that start with '#' are comments.\n# For a project mostly in C, the following would be a good set of\n# exclude patterns (uncomment them if you want to use them):\n# *.[oa]\n# *~\n.DS_Store\n";

    let git_info_path = path.join(".git/info");
    fs::create_dir_all(git_info_path).unwrap();

    let git_info_exclude_path = path.join(".git/info/exclude");
    fs::write(git_info_exclude_path, git_info_exclude_txt).unwrap();
}
