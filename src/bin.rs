use std::io::{self, Write};

mod cli;

#[cfg(test)]
pub(crate) mod test_support;

#[allow(unused_must_use)]
#[cfg(not(tarpaulin_include))]
fn main() {
    // The actual rsgit executable (main fn) doesn't seem to be reachable via Tarpaulin.
    // We put as little as possible into this function so we can reach the rest via
    // other test coverage.

    let stdin = io::stdin();
    let mut stdin = stdin.lock();

    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    let mut cli = cli::Cli {
        arg_matches: cli::app().get_matches(),
        stdin: &mut stdin,
        stdout: &mut stdout,
    };

    let r = cli.run();

    cli.flush();
    // Intentionally ignoring the result of this flush.

    std::process::exit(match r {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("ERROR: {}", err);
            1
        }
    });
}
