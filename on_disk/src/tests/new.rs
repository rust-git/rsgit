use std::fs;

use super::super::*;

use crate::TempGitRepo;

#[test]
fn happy_path() {
    let tgr = TempGitRepo::new();
    let work_dir = tgr.path();
    let git_dir = work_dir.join(".git");
    let r = OnDisk::new(&work_dir).unwrap();
    assert_eq!(r.work_dir(), work_dir);
    assert_eq!(r.git_dir(), git_dir.as_path());
}

#[test]
fn error_no_work_dir() {
    let tgr = TempGitRepo::new();
    let work_dir = tgr.path().join("bogus");
    let err = OnDisk::new(&work_dir).unwrap_err();
    if let Error::WorkDirDoesntExist(_) = err {
        // expected
    } else {
        panic!("wrong error: {:?}", err);
    }
}

#[test]
fn error_no_git_dir() {
    let tempdir = tempfile::tempdir().unwrap();
    let work_dir = tempdir.path();
    let err = OnDisk::new(&work_dir).unwrap_err();
    if let Error::GitDirDoesntExist(_) = err {
        // expected
    } else {
        panic!("wrong error: {:?}", err);
    }
}

#[test]
fn matches_command_line_git() {
    let tgr = TempGitRepo::new();
    let c_path = tgr.path();

    let r_path = tempfile::tempdir().unwrap();
    OnDisk::init(r_path.path()).unwrap();

    assert!(!dir_diff::is_different(c_path, r_path.path()).unwrap());
}

#[test]
fn err_if_git_dir_exists() {
    let r_path = tempfile::tempdir().unwrap();
    let git_dir = r_path.path().join(".git");
    fs::create_dir_all(&git_dir).unwrap();

    let err = OnDisk::init(r_path.path()).unwrap_err();
    if let Error::GitDirShouldntExist(_) = err {
        // expected case
    } else {
        panic!("wrong error: {:?}", err);
    }
}
