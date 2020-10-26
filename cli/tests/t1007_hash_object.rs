use std::{
    fs::File,
    io::Write,
    process::{Command, Stdio},
};

mod common;

const HELLO_CONTENT: &[u8; 11] = b"Hello World";
const HELLO_SHA1: &[u8; 40] = b"5e1c309dae7f45e0f39b1bf3ac3cd9db12e7d689";

// --- Argument checking

#[test]
fn error_multiple_stdin_args() {
    common::compare_git_and_rsgit(|cmd, path| {
        common::init_empty_repo(path);

        let test_content: Vec<u8> = b"test content\n".to_vec();

        let mut proc = Command::new(cmd)
            .current_dir(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(&["hash-object", "--stdin", "--stdin"])
            .spawn()
            .unwrap();

        {
            let stdin = proc.stdin.as_mut().unwrap();
            stdin.write_all(&test_content).unwrap();
        }

        let output = proc.wait_with_output();
        assert!(!output.unwrap().status.success());
    });
}

// TODO: --stdin-paths option is not currently supported.
// Add tests for that case if that is ever added.

// TODO: --no-filters option is not currently supported.
// Add tests for that case if that is ever added.

// --- Behavior

#[test]
fn hash_file_without_writing() {
    common::compare_git_and_rsgit(|cmd, path| {
        common::init_empty_repo(path);

        let hello_path = path.join("hello");

        {
            let mut f = File::create(&hello_path).unwrap();
            f.write_all(HELLO_CONTENT).unwrap();
        }

        let hello_path_str = hello_path.to_str().unwrap();

        let output = Command::new(cmd)
            .current_dir(path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(&["hash-object", hello_path_str])
            .output()
            .unwrap();

        let mut expected_output = HELLO_SHA1.to_vec();
        expected_output.push(10);

        // TODO: Verify that blob does not exist.
        // Needs implementation of cat-file.

        assert_eq!(output.stdout, expected_output);
    });
}

#[test]
fn hash_file_without_writing_from_stdin() {
    common::compare_git_and_rsgit(|cmd, path| {
        common::init_empty_repo(path);

        let mut proc = Command::new(cmd)
            .current_dir(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(&["hash-object", "--stdin"])
            .spawn()
            .unwrap();

        {
            let stdin = proc.stdin.as_mut().unwrap();
            stdin.write_all(HELLO_CONTENT).unwrap();
        }

        let stdout = proc.wait_with_output().unwrap().stdout;

        let mut expected_output = HELLO_SHA1.to_vec();
        expected_output.push(10);

        // TODO: Verify that blob does not exist.
        // Needs implementation of cat-file.

        assert_eq!(stdout, expected_output);
    });
}

#[test]
fn hash_file_and_write_to_database() {
    common::compare_git_and_rsgit(|cmd, path| {
        common::init_empty_repo(path);

        let hello_path = path.join("hello");

        {
            let mut f = File::create(&hello_path).unwrap();
            f.write_all(HELLO_CONTENT).unwrap();
        }

        let hello_path_str = hello_path.to_str().unwrap();

        let output = Command::new(cmd)
            .current_dir(path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(&["hash-object", "-w", hello_path_str])
            .output()
            .unwrap();

        let mut expected_output = HELLO_SHA1.to_vec();
        expected_output.push(10);

        // TODO: Verify that blob exists.
        // Needs implementation of cat-file.

        assert_eq!(output.stdout, expected_output);
    });
}

// TODO: Add test cases for combinations of --stdin and file inputs.
// Not currently supported in rsgit.

// TODO: Add test cases for .gitattributes and CR/LF conversions.
// Not currently supported in rsgit.

// TODO: Add test cases for type filtering based on path.
// Not currently supported in rsgit.

// TODO: Add test cases for .gitattributes in subdirectories.
// Not currently supported in rsgit.

// TODO: Add test cases for type filtering in subdirectory.
// Not currently supported in rsgit.

// TODO: Add test cases for --no-filters option.
// Not currently supported in rsgit.

// TODO: Add test cases for --no-filters in combination with --stdin-paths option.
// Not currently supported in rsgit.

#[test]
fn hash_file_write_std_to_db() {
    common::compare_git_and_rsgit(|cmd, path| {
        common::init_empty_repo(path);

        let mut proc = Command::new(cmd)
            .current_dir(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(&["hash-object", "--stdin", "-w"])
            .spawn()
            .unwrap();

        {
            let stdin = proc.stdin.as_mut().unwrap();
            stdin.write_all(HELLO_CONTENT).unwrap();
        }

        let stdout = proc.wait_with_output().unwrap().stdout;

        let mut expected_output = HELLO_SHA1.to_vec();
        expected_output.push(10);

        // TODO: Verify that blob exists.
        // Needs implementation of cat-file.

        assert_eq!(stdout, expected_output);
    });
}

#[test]
fn hash_file_write_std_to_db_args_swapped() {
    // Same as previous test but with -w and --stdin args in opposite order.
    common::compare_git_and_rsgit(|cmd, path| {
        common::init_empty_repo(path);

        let mut proc = Command::new(cmd)
            .current_dir(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(&["hash-object", "-w", "--stdin"])
            .spawn()
            .unwrap();

        {
            let stdin = proc.stdin.as_mut().unwrap();
            stdin.write_all(HELLO_CONTENT).unwrap();
        }

        let stdout = proc.wait_with_output().unwrap().stdout;

        let mut expected_output = HELLO_SHA1.to_vec();
        expected_output.push(10);

        // TODO: Verify that blob exists.
        // Needs implementation of cat-file.

        assert_eq!(stdout, expected_output);
    });
}

// TODO: Add tests for hashing multiple files at same time.
// Not currently supported in rsgit.

// TODO: Add tests for hashing multiple files based on --stdin-paths at same time.
// Not currently supported in rsgit.

#[test]
fn error_malformed_tree() {
    common::compare_git_and_rsgit(|cmd, path| {
        common::init_empty_repo(path);

        let malformed_tree_path = path.join("malformed-tree");

        {
            let mut f = File::create(&malformed_tree_path).unwrap();
            f.write_all(b"abc").unwrap();
        }

        let malformed_tree_path_str = malformed_tree_path.to_str().unwrap();

        let proc = Command::new(cmd)
            .current_dir(path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(&["hash-object", "-t", "tree", malformed_tree_path_str])
            .spawn()
            .unwrap();

        let output = proc.wait_with_output();
        assert!(!output.unwrap().status.success());
    });
}

#[test]
fn error_malformed_mode_in_tree() {
    common::compare_git_and_rsgit(|cmd, path| {
        common::init_empty_repo(path);

        let malformed_tree_path = path.join("tree-with-malformed-mode");

        {
            let mut f = File::create(&malformed_tree_path).unwrap();
            f.write_all(b"9100644 \0\x01\x02\x03\x04\x05\x06\x07\x08\x09\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x20").unwrap();
        }

        let malformed_tree_path_str = malformed_tree_path.to_str().unwrap();

        let proc = Command::new(cmd)
            .current_dir(path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(&["hash-object", "-t", "tree", malformed_tree_path_str])
            .spawn()
            .unwrap();

        let output = proc.wait_with_output();
        assert!(!output.unwrap().status.success());
    });
}

#[test]
fn error_empty_filename_in_tree() {
    common::compare_git_and_rsgit(|cmd, path| {
        common::init_empty_repo(path);

        let malformed_tree_path = path.join("tree-with-malformed-mode");

        {
            let mut f = File::create(&malformed_tree_path).unwrap();
            f.write_all(b"100644 \0\x01\x02\x03\x04\x05\x06\x07\x08\x09\x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x20").unwrap();
        }

        let malformed_tree_path_str = malformed_tree_path.to_str().unwrap();

        let proc = Command::new(cmd)
            .current_dir(path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(&["hash-object", "-t", "tree", malformed_tree_path_str])
            .spawn()
            .unwrap();

        let output = proc.wait_with_output();
        assert!(!output.unwrap().status.success());
    });
}

#[test]
fn error_corrupt_commit() {
    common::compare_git_and_rsgit(|cmd, path| {
        common::init_empty_repo(path);

        let mut proc = Command::new(cmd)
            .current_dir(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(&["hash-object", "-t", "commit", "--stdin"])
            .spawn()
            .unwrap();

        {
            let stdin = proc.stdin.as_mut().unwrap();
            stdin.write_all(b"").unwrap();
        }

        let output = proc.wait_with_output();
        assert!(!output.unwrap().status.success());
    });
}

#[test]
fn error_corrupt_tag() {
    common::compare_git_and_rsgit(|cmd, path| {
        common::init_empty_repo(path);

        let mut proc = Command::new(cmd)
            .current_dir(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(&["hash-object", "-t", "tag", "--stdin"])
            .spawn()
            .unwrap();

        {
            let stdin = proc.stdin.as_mut().unwrap();
            stdin.write_all(b"").unwrap();
        }

        let output = proc.wait_with_output();
        assert!(!output.unwrap().status.success());
    });
}

#[test]
fn error_invalid_type_name() {
    common::compare_git_and_rsgit(|cmd, path| {
        common::init_empty_repo(path);

        let mut proc = Command::new(cmd)
            .current_dir(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(&["hash-object", "-t", "bogus", "--stdin"])
            .spawn()
            .unwrap();

        {
            let stdin = proc.stdin.as_mut().unwrap();
            stdin.write_all(b"").unwrap();
        }

        let output = proc.wait_with_output();
        assert!(!output.unwrap().status.success());
    });
}

#[test]
fn error_truncated_type_name() {
    common::compare_git_and_rsgit(|cmd, path| {
        common::init_empty_repo(path);

        let mut proc = Command::new(cmd)
            .current_dir(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(&["hash-object", "-t", "bl", "--stdin"])
            .spawn()
            .unwrap();

        {
            let stdin = proc.stdin.as_mut().unwrap();
            stdin.write_all(b"").unwrap();
        }

        let output = proc.wait_with_output();
        assert!(!output.unwrap().status.success());
    });
}

#[test]
fn literally() {
    common::compare_git_and_rsgit(|cmd, path| {
        common::init_empty_repo(path);

        let mut proc = Command::new(cmd)
            .current_dir(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(&["hash-object", "-t", "1234567890", "--literally", "--stdin"])
            .spawn()
            .unwrap();

        {
            let stdin = proc.stdin.as_mut().unwrap();
            stdin.write_all(b"example").unwrap();
        }

        let output = proc.wait_with_output().unwrap();
        println!(
            "--- stderr\n\n{}\n",
            String::from_utf8_lossy(&output.stderr)
        );

        assert!(output.status.success());
    });
}

#[test]
fn write_literally() {
    common::compare_git_and_rsgit(|cmd, path| {
        common::init_empty_repo(path);

        let mut proc = Command::new(cmd)
            .current_dir(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(&[
                "hash-object",
                "-w",
                "-t",
                "1234567890",
                "--literally",
                "--stdin",
            ])
            .spawn()
            .unwrap();

        {
            let stdin = proc.stdin.as_mut().unwrap();
            stdin.write_all(b"example").unwrap();
        }

        let output = proc.wait_with_output().unwrap();
        println!(
            "--- stderr\n\n{}\n",
            String::from_utf8_lossy(&output.stderr)
        );

        assert!(output.status.success());
    });
}

#[test]
fn literally_long_type() {
    common::compare_git_and_rsgit(|cmd, path| {
        common::init_empty_repo(path);

        let long_type = "1234567890".repeat(150);

        let mut proc = Command::new(cmd)
            .current_dir(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .args(&["hash-object", "-t", &long_type, "--literally", "--stdin"])
            .spawn()
            .unwrap();

        {
            let stdin = proc.stdin.as_mut().unwrap();
            stdin.write_all(b"example").unwrap();
        }

        let output = proc.wait_with_output().unwrap();
        println!(
            "--- stderr\n\n{}\n",
            String::from_utf8_lossy(&output.stderr)
        );

        assert!(output.status.success());
    });
}
