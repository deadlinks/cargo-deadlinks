extern crate assert_cmd;

use assert_cmd::prelude::*;
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

        // fails with generated docs
        Command::cargo_bin("cargo-deadlinks")
            .unwrap()
            .arg("deadlinks")
            .current_dir("./tests/non_existent_http_link")
            .assert()
            .failure();
    }
}
