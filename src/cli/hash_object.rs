use std::io::Write;

use super::{find_repo, Cli, Result};

use rsgit::object::{ContentSource, FileContentSource, Kind, Object, ReadContentSource};
use rsgit::repo::Repo;

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
