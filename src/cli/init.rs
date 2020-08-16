use std::io::Write;

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

pub(crate) fn run<'a, W>(_matches: &ArgMatches<'a>, _init_matches: &ArgMatches<'a>, stdout: &mut W)
where
    W: Write,
{
    write!(stdout, "INIT hello").unwrap();
}

#[cfg(test)]
mod tests {
    use crate::cli;

    #[test]
    fn happy_path() {
        // panic!("huh");
        let mut stdout: Vec<u8> = Vec::new();
        cli::dispatch_args(vec!["rsgit", "init", "dir"], &mut stdout);

        assert_eq!(stdout.as_slice(), b"INIT hello");
    }
}
