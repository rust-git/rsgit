use std::io::Write;

use super::super::*;

use crate::TempGitRepo;

use rsgit_core::object::{Kind, Object};

use tempfile::{tempdir, NamedTempFile};

const TEST_CONTENT: &[u8; 13] = b"test content\n";

#[test]
fn matches_command_line_git() {
    let mut test_file = NamedTempFile::new().unwrap();
    test_file.write_all(TEST_CONTENT).unwrap();

    let mut tgr = TempGitRepo::new();
    let output = tgr
        .command("git")
        .args(&["hash-object", "-w", test_file.path().to_str().unwrap()])
        .output()
        .unwrap();

    let expected_output = b"d670460b4b4aece5915caf5c68d12f560a9fe3e4\n".to_vec();

    assert!(output.status.success());
    assert_eq!(output.stdout, expected_output);

    let rsgit_temp = tempdir().unwrap();
    let r_path = rsgit_temp.path();
    let mut r = OnDiskRepo::init(r_path).unwrap();

    let o = Object::new(&Kind::Blob, Box::new(TEST_CONTENT.to_vec())).unwrap();
    r.put_loose_object(&o).unwrap();

    assert!(!dir_diff::is_different(tgr.path(), r_path).unwrap());
}

#[test]
fn matches_command_line_git_large_file() {
    let mut test_file = NamedTempFile::new().unwrap();
    let test_content = "foobar".repeat(1000);
    let test_content = test_content.as_bytes();
    test_file.write_all(&test_content).unwrap();

    let mut tgr = TempGitRepo::new();
    let output = tgr
        .command("git")
        .args(&["hash-object", "-w", test_file.path().to_str().unwrap()])
        .output()
        .unwrap();

    let mut object_id = String::from_utf8(output.stdout).unwrap();
    object_id.truncate(40);

    let rsgit_temp = tempdir().unwrap();
    let r_path = rsgit_temp.path();
    let mut r = OnDiskRepo::init(r_path).unwrap();

    let o = Object::new(&Kind::Blob, Box::new(test_content.to_vec())).unwrap();
    assert_eq!(object_id, o.id().to_string());
    r.put_loose_object(&o).unwrap();

    assert!(!dir_diff::is_different(tgr.path(), r_path).unwrap());
}

#[test]
fn error_cant_create_objects_dir() {
    let rsgit_temp = tempdir().unwrap();
    let r_path = rsgit_temp.path();
    let mut r = OnDiskRepo::init(r_path).unwrap();

    let objects_dir = r_path.join(".git/objects/d6");
    fs::write(&objects_dir, "sand in the gears").unwrap();

    let o = Object::new(&Kind::Blob, Box::new(TEST_CONTENT.to_vec())).unwrap();
    let err = r.put_loose_object(&o).unwrap_err();

    match err {
        Error::IoError(err) => assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists),
        _ => panic!("Unexpected error {:?}", err),
    }
}

#[test]
fn error_object_exists() {
    let rsgit_temp = tempdir().unwrap();
    let r_path = rsgit_temp.path();
    let mut r = OnDiskRepo::init(r_path).unwrap();

    let mut object_path = r_path.join(".git/objects/d6");
    fs::create_dir(&object_path).unwrap();

    object_path.push("70460b4b4aece5915caf5c68d12f560a9fe3e4");
    fs::write(&object_path, "sand in the gears").unwrap();

    let o = Object::new(&Kind::Blob, Box::new(TEST_CONTENT.to_vec())).unwrap();
    let err = r.put_loose_object(&o).unwrap_err();

    match err {
        Error::IoError(err) => assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists),
        _ => panic!("Unexpected error {:?}", err),
    }
}
