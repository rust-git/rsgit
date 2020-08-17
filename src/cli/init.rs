use std::io::Write;
use std::path::Path;

use super::{Cli, Result};

use rsgit::repo::OnDisk;

use clap::{App, Arg, ArgMatches, SubCommand};

pub(crate) fn subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("init")
        .about("Create an empty git repository")
        .arg(
            Arg::with_name("directory")
                .required(true)
                .help("The directory to create"),
        )
}

pub(crate) fn run(cli: &mut Cli, init_matches: &ArgMatches) -> Result {
    let dir = init_matches.value_of("directory").unwrap();
    let path = Path::new(dir);

    OnDisk::init(path)?;

    writeln!(
        cli,
        "Initialized empty Git repository in {}",
        path.display()
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::cli::Cli;
    use crate::test_support::TempGitRepo;

    #[test]
    fn matches_command_line_git() {
        let tgr = TempGitRepo::new();
        let c_path = tgr.path();

        let r_path = tempfile::tempdir().unwrap();
        let r_pathstr = r_path.path().to_str().unwrap();

        let stdout = Cli::run_with_args(vec!["rsgit", "init", &r_pathstr]).unwrap();

        let expected_std = format!("Initialized empty Git repository in {}\n", r_pathstr);

        assert_eq!(stdout, expected_std.as_bytes());

        assert_eq!(
            dir_diff::is_different(c_path, r_path.path()).unwrap(),
            false
        );
    }

    #[test]
    fn error_no_dir() {
        let err = Cli::run_with_args(vec!["rsgit", "init"]).unwrap_err();

        let errmsg = err.to_string();
        assert!(
            errmsg.contains("required arguments were not provided"),
            "\nincorrect error message:\n\n{}",
            errmsg
        );
    }

    #[test]
    fn error_too_many_args() {
        let err = Cli::run_with_args(vec!["rsgit", "init", "here", "and there"]).unwrap_err();

        let errmsg = err.to_string();
        assert!(
            errmsg.contains("wasn't expected"),
            "\nincorrect error message:\n\n{}",
            errmsg
        );
    }
}
