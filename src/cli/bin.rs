use clap::{crate_version, App};

#[cfg(not(tarpaulin_include))]
pub fn main() {
    let _matches = App::new("rsgit").version(crate_version!()).get_matches();

    println!("Hello from rsgit!");
}

#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use predicates::prelude::*;

    #[test]
    fn happy_path() {
        let mut cmd = Command::cargo_bin("rsgit").unwrap();
        cmd.assert().success().stdout("Hello from rsgit!\n");
    }

    #[test]
    fn version() {
        let mut cmd = Command::cargo_bin("rsgit").unwrap();
        cmd.arg("--version")
            .assert()
            .success()
            .stdout(predicate::str::starts_with("rsgit 0."));
    }
}
