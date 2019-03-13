extern crate assert_cmd;
extern crate predicates;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

mod simple_project {
    use super::*;

    #[test]
    #[allow(unused_must_use)]
    fn it_checks_okay_project_correctly() {
        // cargo-deadlinks fails when docs have not been generated before
        match std::fs::remove_dir_all("./tests/simple_project/target") {
            Ok(_) => {}
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => {}
                _ => panic!(
                    "Unexpected error when trying do remove target directory: {:?}",
                    err
                ),
            },
        };
        Command::cargo_bin("cargo-deadlinks")
            .unwrap()
            .arg("deadlinks")
            .current_dir("./tests/simple_project")
            .assert()
            .failure()
            .stdout(predicate::str::contains(
                "Could not find directory \"target/doc/simple_project\".",
            ))
            .stdout(predicate::str::contains(
                "Please run `cargo doc` before running `cargo deadlinks`.",
            ));

        // generate docs
        Command::new("cargo")
            .arg("doc")
            .current_dir("./tests/simple_project")
            .assert()
            .success();

        // succeeds with generated docs
        Command::cargo_bin("cargo-deadlinks")
            .unwrap()
            .arg("deadlinks")
            .current_dir("./tests/simple_project")
            .assert()
            .success();
    }
}
