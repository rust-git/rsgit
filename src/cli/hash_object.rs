use super::{find_repo, Cli, Result};

use rsgit::object::{FileContentSource, Kind, Object};

use clap::{App, Arg, ArgMatches, Error, ErrorKind, SubCommand};

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
            Arg::with_name("literally")
                .long("literally")
                .help("Bypass validity checks"),
        )
        .arg(Arg::with_name("file").required(true))
}

pub(crate) fn run(_cli: &mut Cli, args: &ArgMatches) -> Result<()> {
    let _repo = find_repo::from_current_dir()?;
    let _object = object_from_args(&args)?;

    // TO DO: Check validity of object (if not --literally).

    // TO DO: Write object to repo (if -w).

    // TO DO: Write object ID.

    Ok(())
}

fn object_from_args(args: &ArgMatches) -> Result<Object> {
    let kind = type_from_args(&args)?;
    let content_source = content_source_from_args(&args)?;
    let object = Object::new(kind, Box::new(content_source))?;
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

fn content_source_from_args(args: &ArgMatches) -> Result<FileContentSource> {
    // Justification for using unwrap() here:
    // CLAP should have errored out before this point
    // if there was no "file" argument.
    let file = args.value_of("file").unwrap();
    let content_source = FileContentSource::new(file)?;
    Ok(content_source)
}

// #[cfg(test)]
// mod tests {
//     use crate::cli::Cli;
//     use crate::test_support::TempGitRepo;

//     #[test]
//     fn matches_command_line_git() {
//         let tgr = TempGitRepo::new();
//         let c_path = tgr.path();

//         let r_path = tempfile::tempdir().unwrap();
//         let r_pathstr = r_path.path().to_str().unwrap();

//         let stdout = Cli::run_with_args(vec!["init", &r_pathstr]).unwrap();

//         let expected_std = format!("Initialized empty Git repository in {}\n", r_pathstr);

//         assert_eq!(stdout, expected_std.as_bytes());
//         assert!(!dir_diff::is_different(c_path, r_path.path()).unwrap());
//     }

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
// }
