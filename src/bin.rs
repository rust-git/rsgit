use std::io;

mod cli;

#[cfg(not(tarpaulin_include))]
fn main() {
    // The actual rsgit executable (main fn) doesn't seem to be reachable via Tarpaulin.
    // We put as little as possible into this function so we can reach the rest via
    // other test coverage.
    let matches = cli::app().get_matches();

    let stdout = io::stdout();
    let mut stdout_handle = stdout.lock();

    cli::dispatch(&matches, &mut stdout_handle);
}
