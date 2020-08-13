use clap::{crate_version, App};

pub fn main() {
    let _matches = App::new("rsgit").version(crate_version!()).get_matches();

    println!("Hello from rsgit!");
}
