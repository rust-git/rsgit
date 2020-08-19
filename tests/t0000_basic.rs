use std::fs;
use std::process::Command;

mod common;

#[test]
fn objects_dir_is_empty() {
    common::compare_git_and_rsgit(|cmd, path| {
        Command::new(cmd)
            .args(&["init", path.to_str().unwrap()])
            .output()
            .unwrap();

        common::sanitize_repo(path);

        let objects_dir = path.join(".git/objects");
        assert!(objects_dir.is_dir());
        assert_eq!(
            fs::read_dir(objects_dir)
                .unwrap()
                .filter(|x| !x.as_ref().unwrap().path().is_dir())
                .count(),
            0
        );
    });
}

#[test]
fn objects_dir_has_two_subdirs() {
    // C git has this as three subdirs, but I see only `info` and `pack`.

    common::compare_git_and_rsgit(|cmd, path| {
        Command::new(cmd)
            .args(&["init", path.to_str().unwrap()])
            .output()
            .unwrap();

        common::sanitize_repo(path);

        let objects_dir = path.join(".git/objects");
        assert!(objects_dir.is_dir());
        assert_eq!(
            fs::read_dir(objects_dir)
                .unwrap()
                .filter(|x| x.as_ref().unwrap().path().is_dir())
                .count(),
            2
        );
    });
}

// NOTE: Most of C git's t0000 is testing the test framework itself,
// which is unnecessary in this case because we're using Rust's built-in
// test framework.

// TODO: Review update-index and write-tree tests starting at line 1017.
