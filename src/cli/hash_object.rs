use std::error::Error;

use super::{find_repo, Cli};

// use rsgit::repo::OnDisk;
use rsgit::object::Object;

use clap::{App, Arg, ArgMatches, SubCommand};

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

pub(crate) fn run(_cli: &mut Cli, args: &ArgMatches) -> super::Result {
    let _repo = find_repo::from_current_dir()?;

    let _object = object_from_args(&args)?;

    // TO DO: Parse cmd-line opts.

    // TO DO: Hash the object, etc.

    Ok(())
}

fn object_from_args(args: &ArgMatches) -> Result<Object, Box<dyn Error>> {
    let type = type_from_args(args: &ArgMatches)
}

fn type_from_args(args: &ArgMatches) ->

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
