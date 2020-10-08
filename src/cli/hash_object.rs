use std::io::Write;

use super::{find_repo, Cli, Result};

use clap::{App, Arg, ArgMatches, Error, ErrorKind, SubCommand};

use rsgit::{
    object::{ContentSource, FileContentSource, Kind, Object, ReadContentSource},
    repo::Repo,
};

pub(crate) fn subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("hash-object")
        .about("Compute object ID and optionally creates a blob from a file")
        .arg(
            Arg::with_name("t")
                .short("t")
                .value_name("type")
                .help("Specify the type (default 'blob')"),
        )
        .arg(
            Arg::with_name("w")
                .short("w")
                .help("Actually write the object into the object database"),
        )
        .arg(
            Arg::with_name("stdin")
                .long("stdin")
                .help("Read the object from standard input instead of from a file"),
        )
        .arg(
            Arg::with_name("literally")
                .long("literally")
                .help("Bypass validity checks"),
        )
        .arg(Arg::with_name("file"))
}

pub(crate) fn run(cli: &mut Cli, args: &ArgMatches) -> Result<()> {
    let object = object_from_args(cli, &args)?;

    if !args.is_present("literally") && !object.is_valid()? {
        return Err(Box::new(Error {
            message: format!("corrupt {}", args.value_of("t").unwrap()),
            kind: ErrorKind::InvalidValue,
            info: None,
        }));
    }

    if args.is_present("w") {
        let mut repo = find_repo::from_current_dir()?;
        repo.put_loose_object(&object)?;
    }

    writeln!(cli, "{}", object.id())?;

    Ok(())
}

fn object_from_args(cli: &mut Cli, args: &ArgMatches) -> Result<Object> {
    let kind = type_from_args(&args)?;
    let content_source = content_source_from_args(cli, &args)?;
    let object = Object::new(kind, content_source)?;
    Ok(object)
}

fn type_from_args(args: &ArgMatches) -> Result<Kind> {
    match args.value_of("t") {
        Some(type_str) => match type_str {
            "blob" => Ok(Kind::Blob),
            "commit" => Ok(Kind::Commit),
            "tag" => Ok(Kind::Tag),
            "tree" => Ok(Kind::Tree),
            _ => Err(Box::new(Error {
                message: "-t must be one of blob, commit, tag, or tree".to_string(),
                kind: ErrorKind::InvalidValue,
                info: None,
            })),
        },
        None => Ok(Kind::Blob),
    }
}

fn content_source_from_args(cli: &mut Cli, args: &ArgMatches) -> Result<Box<dyn ContentSource>> {
    let stdin = args.is_present("stdin");
    let file = args.value_of("file");

    if file.is_some() && !stdin {
        Ok(Box::new(FileContentSource::new(file.unwrap())?))
    } else if stdin && file.is_none() {
        Ok(Box::new(ReadContentSource::new(&mut cli.stdin)?))
    } else {
        Err(Box::new(Error {
            message: "content source must be either --stdin or a file path".to_string(),
            kind: ErrorKind::MissingRequiredArgument,
            info: None,
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::Write,
        process::{Command, Stdio},
    };

    use crate::{
        cli::Cli,
        test_support::{TempCwd, TempGitRepo},
    };

    use serial_test::serial;
    use tempfile::TempDir;

    #[test]
    fn hash_with_no_repo() {
        // $ echo 'test content' | git hash-object --stdin
        // d670460b4b4aece5915caf5c68d12f560a9fe3e4

        let stdin: Vec<u8> = b"test content\n".to_vec();
        let stdout = Cli::run_with_stdin_and_args(stdin, vec!["hash-object", "--stdin"]).unwrap();

        let expected_stdout = "d670460b4b4aece5915caf5c68d12f560a9fe3e4\n";
        assert_eq!(stdout, expected_stdout.as_bytes());
    }

    #[test]
    fn large_file_on_disk_no_repo() {
        let dir = TempDir::new().unwrap();
        let path = dir.as_ref().join("example");

        {
            let mut f = File::create(&path).unwrap();
            for _ in 0..1000 {
                f.write_all(b"foobar").unwrap();
            }
        }

        let path_str = path.to_str().unwrap();

        let rsgit_stdout = Cli::run_with_args(vec!["hash-object", path_str]).unwrap();

        let cgit_stdout = Command::new("git")
            .args(&["hash-object", path_str])
            .output()
            .unwrap()
            .stdout;

        assert_eq!(rsgit_stdout, cgit_stdout);
    }

    #[test]
    #[serial]
    fn matches_command_line_git() {
        let stdin: Vec<u8> = b"test content\n".to_vec();

        let c_tgr = TempGitRepo::new();
        let c_path = c_tgr.path();

        let mut cgit = Command::new("git")
            .current_dir(c_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .args(&["hash-object", "-w", "--stdin"])
            .spawn()
            .unwrap();

        {
            let cgit_stdin = cgit.stdin.as_mut().unwrap();
            cgit_stdin.write_all(&stdin).unwrap();
        }

        let c_stdout = cgit.wait_with_output().unwrap().stdout;
        let r_tgr = TempGitRepo::new();
        let r_path = r_tgr.path();

        let _r_cwd = TempCwd::new(r_path);
        let r_stdout =
            Cli::run_with_stdin_and_args(stdin, vec!["hash-object", "-w", "--stdin"]).unwrap();

        assert_eq!(c_stdout, r_stdout);

        assert!(!dir_diff::is_different(c_path, r_path).unwrap());
    }

    #[test]
    #[serial]
    fn err_corrupt_commit() {
        let stdin: Vec<u8> = b"test content\n".to_vec();

        let c_tgr = TempGitRepo::new();
        let c_path = c_tgr.path();

        let r_tgr = TempGitRepo::new();
        let r_path = r_tgr.path();

        let _r_cwd = TempCwd::new(r_path);
        let r_err = Cli::run_with_stdin_and_args(
            stdin,
            vec!["hash-object", "-t", "commit", "-w", "--stdin"],
        )
        .unwrap_err();

        assert_eq!(r_err.to_string(), "corrupt commit\n");

        assert!(!dir_diff::is_different(c_path, r_path).unwrap());
    }

    #[test]
    #[serial]
    fn err_corrupt_tree() {
        let stdin: Vec<u8> = b"test content\n".to_vec();

        let c_tgr = TempGitRepo::new();
        let c_path = c_tgr.path();

        let r_tgr = TempGitRepo::new();
        let r_path = r_tgr.path();

        let _r_cwd = TempCwd::new(r_path);
        let r_err =
            Cli::run_with_stdin_and_args(stdin, vec!["hash-object", "-t", "tree", "-w", "--stdin"])
                .unwrap_err();

        assert_eq!(r_err.to_string(), "corrupt tree\n");

        assert!(!dir_diff::is_different(c_path, r_path).unwrap());
    }

    #[test]
    #[serial]
    fn err_corrupt_tag() {
        let stdin: Vec<u8> = b"test content\n".to_vec();

        let c_tgr = TempGitRepo::new();
        let c_path = c_tgr.path();

        let r_tgr = TempGitRepo::new();
        let r_path = r_tgr.path();

        let _r_cwd = TempCwd::new(r_path);
        let r_err =
            Cli::run_with_stdin_and_args(stdin, vec!["hash-object", "-t", "tag", "-w", "--stdin"])
                .unwrap_err();

        assert_eq!(r_err.to_string(), "corrupt tag\n");

        assert!(!dir_diff::is_different(c_path, r_path).unwrap());
    }

    //     #[test]
    //     fn error_no_dir() {
    //         let err = Cli::run_with_args(vec!["init"]).unwrap_err();

    //         let errmsg = err.to_string();
    //         assert!(
    //             errmsg.contains("required arguments were not provided"),
    //             "\nincorrect error message:\n\n{}",
    //             errmsg
    //         );
    //     }

    //     #[test]
    //     fn error_too_many_args() {
    //         let err = Cli::run_with_args(vec!["init", "here", "and there"]).unwrap_err();

    //         let errmsg = err.to_string();
    //         assert!(
    //             errmsg.contains("wasn't expected"),
    //             "\nincorrect error message:\n\n{}",
    //             errmsg
    //         );
    //     }
}
