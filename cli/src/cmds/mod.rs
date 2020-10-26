use crate::{App, Result};

mod hash_object;
mod init;

pub(crate) fn add_subcommands<'a, 'b>(app: clap::App<'a, 'b>) -> clap::App<'a, 'b> {
    app.subcommand(hash_object::subcommand())
        .subcommand(init::subcommand())
}

pub(crate) fn dispatch(app: &mut App) -> Result<()> {
    let matches = app.arg_matches.clone();
    // ^^ Ugh. Need an independent copy of matches so we can still pass
    // the App struct through to subcommand imps.

    match matches.subcommand() {
        ("hash-object", Some(m)) => hash_object::run(app, &m),
        ("init", Some(m)) => init::run(app, &m),
        _ => unreachable!(),
        // unreachable: Should have exited out with appropriate help or
        // error message if no subcommand was given.
    }
}
