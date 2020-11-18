extern crate assert_cmd;

use assert_cmd::prelude::*;
use predicates::str::contains;
use std::process::Command;

#[test]
fn reports_broken_links() {
    Command::cargo_bin("cargo-deadlinks")
        .unwrap()
        .arg("deadlinks")
        .current_dir("./tests/broken_links")
        .assert()
        .failure()
        .stdout(contains(
            "Linked file at path fn.not_here.html does not exist",
        ));
}
