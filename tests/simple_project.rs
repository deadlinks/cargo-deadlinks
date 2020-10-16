extern crate assert_cmd;
extern crate predicates;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

mod simple_project {
    use super::*;

    fn remove_all(path: &str) {
        match std::fs::remove_dir_all(path) {
            Ok(_) => {}
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => {}
                _ => panic!(
                    "Unexpected error when trying do remove target directory: {:?}",
                    err
                ),
            },
        };
    }

    fn assert_doc() -> assert_cmd::assert::Assert {
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
    }

    #[test]
    fn it_checks_okay_project_correctly() {
        use predicate::str::contains;

        std::env::remove_var("CARGO_TARGET_DIR");

        // cargo-deadlinks fails when docs have not been generated before
        remove_all("./tests/simple_project/target");

        Command::cargo_bin("cargo-deadlinks")
            .unwrap()
            .arg("deadlinks")
            .current_dir("./tests/simple_project")
            .assert()
            .failure()
            .stdout(
                contains("Could not find directory")
                    .and(contains("target/doc/simple_project\"."))
                    .and(contains(
                        "Please run `cargo doc` before running `cargo deadlinks`.",
                    )),
            );

        assert_doc().success();

        // NOTE: can't be parallel because of use of `set_var`
        std::env::set_var("CARGO_TARGET_DIR", "target2");
        remove_all("./tests/simple_project/target2");
        assert_doc().success();

        std::env::remove_var("CARGO_TARGET_DIR");
        std::env::set_var("CARGO_BUILD_TARGET", "x86_64-unknown-linux-gnu");
        remove_all("./tests/simple_project/target");
        // This currently breaks due to a cargo bug: https://github.com/rust-lang/cargo/issues/8791
        assert_doc().failure();
    }
}
