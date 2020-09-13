use std::error::Error;

#[cfg(test)]
use std::ffi::OsString;

use std::io::{Read, Write};

use clap::{crate_version, App, AppSettings, ArgMatches};

mod find_repo;
mod init;

pub(crate) fn app<'a, 'b>() -> App<'a, 'b> {
    App::new("rsgit")
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .subcommand(init::subcommand())
}

pub(crate) type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub(crate) struct Cli<'a> {
    pub arg_matches: ArgMatches<'a>,
    pub stdin: &'a mut dyn Read,
    pub stdout: &'a mut dyn Write,
}

impl<'a> Cli<'a> {
    pub fn run(&mut self) -> Result<()> {
        let matches = self.arg_matches.clone();
        // ^^ Ugh. Need an independent copy of matches so we can still pass
        // the Cli struct through to subcommand imps.

        match matches.subcommand() {
            ("init", Some(init_matches)) => init::run(self, &init_matches),
            _ => unreachable!(),
            // unreachable: Should have exited out with appropriate help or
            // error message if no subcommand was given.
        }
    }

    #[cfg(test)]
    pub fn run_with_args<I, T>(args: I) -> std::result::Result<Vec<u8>, Box<dyn Error>>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let mut args: Vec<OsString> = args.into_iter().map(|x| x.into()).collect();
        args.insert(0, OsString::from("rsgit"));

        let mut stdin = std::io::Cursor::new(Vec::new());
        let mut stdout = Vec::new();

        Cli {
            arg_matches: app().get_matches_from_safe(args)?,
            stdin: &mut stdin,
            stdout: &mut stdout,
        }
        .run()?;

        Ok(stdout)
    }
}

impl<'a> Write for Cli<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.stdout.write(buf)
    }

    #[cfg(not(tarpaulin_include))]
    fn flush(&mut self) -> std::io::Result<()> {
        self.stdout.flush()
    }
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
