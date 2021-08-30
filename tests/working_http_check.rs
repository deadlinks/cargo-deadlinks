extern crate assert_cmd;

use assert_cmd::prelude::*;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use std::process::Command;

mod working_http_check {
    use super::*;

    fn remove_target(relative_path: &'static str) {
        match std::fs::remove_dir_all(&format!("./tests/working_http_check/{}", relative_path)) {
            Ok(_) => {}
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => {}
                _ => panic!(
                    "Unexpected error when trying do remove target directory: {:?}",
                    err
                ),
            },
        }
    }

    #[test]
    fn works() {
        remove_target("target");
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
        remove_target("target2");
        Command::cargo_bin("cargo-deadlinks")
            .unwrap()
            .args(&[
                "deadlinks",
                "--forbid-http",
                "--",
                "--target-dir",
                "target2",
            ])
            .current_dir("./tests/working_http_check")
            .assert()
            .failure()
            .stdout(
                contains("HTTP checking is forbidden").and(contains("doc.rust-lang.org").not()),
            );
    }
}
