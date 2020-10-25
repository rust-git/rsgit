#![deny(warnings)]

use std::io::{Read, Write};

#[cfg(test)]
use std::ffi::OsString;

use crate::Result;

use clap::{crate_version, App, AppSettings, ArgMatches};

mod hash_object;
mod init;

pub(crate) fn app<'a, 'b>() -> App<'a, 'b> {
    App::new("rsgit")
        .version(crate_version!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .subcommand(hash_object::subcommand())
        .subcommand(init::subcommand())
}

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
            ("hash-object", Some(m)) => hash_object::run(self, &m),
            ("init", Some(m)) => init::run(self, &m),
            _ => unreachable!(),
            // unreachable: Should have exited out with appropriate help or
            // error message if no subcommand was given.
        }
    }

    #[cfg(test)]
    pub fn run_with_stdin_and_args<I, T>(stdin: Vec<u8>, args: I) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let mut args: Vec<OsString> = args.into_iter().map(|x| x.into()).collect();
        args.insert(0, OsString::from("rsgit"));

        let mut stdin = std::io::Cursor::new(stdin);
        let mut stdout = Vec::new();

        Cli {
            arg_matches: app().get_matches_from_safe(args)?,
            stdin: &mut stdin,
            stdout: &mut stdout,
        }
        .run()?;

        Ok(stdout)
    }

    #[cfg(test)]
    pub fn run_with_args<I, T>(args: I) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let stdin: Vec<u8> = Vec::new();
        Cli::run_with_stdin_and_args(stdin, args)
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
