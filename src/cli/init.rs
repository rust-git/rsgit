use std::io::Write;

use super::{Cli, Result};

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

    if dir == "fail" {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "I don't like that name",
        )))
    } else {
        writeln!(cli, "INIT {}", dir)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::cli::Cli;

    #[test]
    fn happy_path() {
        let stdout = Cli::run_with_args(vec!["rsgit", "init", "dir"]).unwrap();
        assert_eq!(stdout.as_slice(), b"INIT dir\n");
    }
}
