use std::{io::Write, path::Path};

use super::{Cli, Result};

use clap::{App, Arg, ArgMatches, SubCommand};
use rsgit_on_disk::OnDisk;

pub(crate) fn subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("init")
        .about("Create an empty git repository")
        .arg(
            Arg::with_name("directory")
                .required(true)
                .help("The directory to create"),
        )
}

pub(crate) fn run(cli: &mut Cli, init_matches: &ArgMatches) -> Result<()> {
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
    use crate::cmds::Cli;

    use rsgit_on_disk::TempGitRepo;

    #[test]
    fn matches_command_line_git() {
        let tgr = TempGitRepo::new();
        let c_path = tgr.path();

        let r_path = tempfile::tempdir().unwrap();
        let r_pathstr = r_path.path().to_str().unwrap();

        let stdout = Cli::run_with_args(vec!["init", &r_pathstr]).unwrap();

        let expected_std = format!("Initialized empty Git repository in {}\n", r_pathstr);

        assert_eq!(stdout, expected_std.as_bytes());
        assert!(!dir_diff::is_different(c_path, r_path.path()).unwrap());
    }

    #[test]
    fn error_no_dir() {
        let err = Cli::run_with_args(vec!["init"]).unwrap_err();

        let errmsg = err.to_string();
        assert!(
            errmsg.contains("required arguments were not provided"),
            "\nincorrect error message:\n\n{}",
            errmsg
        );
    }

    #[test]
    fn error_too_many_args() {
        let err = Cli::run_with_args(vec!["init", "here", "and there"]).unwrap_err();

        let errmsg = err.to_string();
        assert!(
            errmsg.contains("wasn't expected"),
            "\nincorrect error message:\n\n{}",
            errmsg
        );
    }
}
