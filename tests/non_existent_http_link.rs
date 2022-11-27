extern crate assert_cmd;

use assert_cmd::prelude::*;
use predicates::str::contains;
use std::process::Command;

mod non_existent_http_link {
    use super::*;

    #[test]
    fn fails_for_broken_http_link() {
        match std::fs::remove_dir_all("./tests/non_existent_http_link/target") {
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
            .current_dir("./tests/non_existent_http_link")
            .assert()
            .success();

        // succeeds without --check-http flag
        Command::cargo_bin("cargo-deadlinks")
            .unwrap()
            .arg("deadlinks")
            .current_dir("./tests/non_existent_http_link")
            .assert()
            .success();

        // fails with --check-http flag
        Command::cargo_bin("cargo-deadlinks")
            .unwrap()
            .args(["deadlinks", "--check-http"])
            .current_dir("./tests/non_existent_http_link")
            .assert()
            .failure()
            .stdout(contains(
                "Unexpected HTTP status fetching http://example.com/this/does/not/exist: Not Found",
            ));
    }
}
