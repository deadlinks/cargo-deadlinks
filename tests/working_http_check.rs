extern crate assert_cmd;

use assert_cmd::prelude::*;
use predicates::str::contains;
use std::process::Command;

mod working_http_check {
    use super::*;

    #[test]
    fn works() {
        match std::fs::remove_dir_all("./tests/working_http_check/target") {
            Ok(_) => {}
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => {}
                _ => panic!(
                    "Unexpected error when trying do remove target directory: {:?}",
                    err
                ),
            },
        };

        // generate docs
        Command::new("cargo")
            .arg("doc")
            .current_dir("./tests/working_http_check")
            .assert()
            .success();

        // succeeds with --check-http flag
        Command::cargo_bin("cargo-deadlinks")
            .unwrap()
            .args(&["deadlinks", "--check-http"])
            .current_dir("./tests/working_http_check")
            .assert()
            .success();
    }

    #[test]
    fn forbid_checking() {
        Command::cargo_bin("cargo-deadlinks")
            .unwrap()
            .args(&["deadlinks", "--forbid-http"])
            .current_dir("./tests/working_http_check")
            .assert()
            .failure()
            .stdout(contains("HTTP checking is forbidden"));
    }
}
