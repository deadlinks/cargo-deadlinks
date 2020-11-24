extern crate assert_cmd;

use assert_cmd::prelude::*;
use predicates::prelude::PredicateBooleanExt;
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
        // make sure warnings are emitted
        .stderr(contains("unresolved link"))
        .stdout(
            contains("Linked file at path fn.not_here.html does not exist")
                .and(contains("Linked file at path links does not exist!")),
        );
}
