use std::io::Write;

use clap::{crate_version, App, AppSettings, ArgMatches};

mod init;

pub(crate) fn app<'a, 'b>() -> App<'a, 'b> {
    App::new("rsgit")
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .subcommand(init::subcommand())
}

pub(crate) fn dispatch<'a, W>(matches: &ArgMatches<'a>, stdout: &mut W)
where
    W: Write,
{
    match matches.subcommand() {
        ("init", Some(init_matches)) => init::run(matches, &init_matches, stdout),
        _ => unreachable!(),
        // unreachable: Should have exited out with appropriate help or
        // error message if no subcommand was given.
    }
}

#[cfg(test)]
use std::ffi::OsString;

#[cfg(test)]
pub(crate) fn dispatch_args<I, T, W>(itr: I, stdout: &mut W)
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
    W: Write,
{
    let matches = app().get_matches_from(itr);
    dispatch(&matches, stdout);
}

#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use predicates::prelude::*;

    #[test]
    fn no_subcommand_prints_help() {
        let mut cmd = Command::cargo_bin("rsgit").unwrap();
        cmd.assert()
            .failure()
            .stdout("")
            .stderr(predicate::str::starts_with("rsgit 0."))
            .stderr(predicate::str::contains("USAGE:"));
    }

    #[test]
    fn version() {
        let mut cmd = Command::cargo_bin("rsgit").unwrap();
        cmd.arg("--version")
            .assert()
            .success()
            .stdout(predicate::str::starts_with("rsgit 0."))
            .stderr("");
    }
}
