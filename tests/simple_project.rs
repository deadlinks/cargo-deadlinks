extern crate assert_cmd;
extern crate predicates;

use assert_cmd::prelude::*;
use predicate::str::{contains, starts_with};
use predicates::prelude::*;
use std::env;
use std::path::Path;
use std::process::Command;

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

fn deadlinks() -> Command {
    let mut cmd = Command::cargo_bin("cargo-deadlinks").unwrap();
    cmd.arg("deadlinks").env_remove("CARGO_TARGET_DIR");
    cmd
}

#[must_use = "Assert does nothing until you specify an assert"]
fn assert_doc(dir: impl AsRef<Path>, env: &[(&str, &str)]) -> assert_cmd::assert::Assert {
    let dir = dir.as_ref();

    // generate docs
    Command::new("cargo")
        .arg("doc")
        .env_remove("CARGO_TARGET_DIR")
        .envs(env.iter().copied())
        .current_dir(dir)
        .assert()
        .success();

    // succeeds with generated docs
    deadlinks()
        .envs(env.iter().copied())
        .current_dir(dir)
        .assert()
}

mod simple_project {
    use super::*;

    #[test]
    fn it_gives_help_if_cargo_toml_missing() {
        deadlinks()
            .current_dir(env::temp_dir())
            .assert()
            .failure()
            .stderr(
                contains("help: if this is not a cargo directory, use `--dir`")
                    .and(contains("error: could not find `Cargo.toml`")),
            );
    }

    #[test]
    fn it_checks_okay_project_correctly() {
        // cargo-deadlinks generates the documentation if it does not yet exist
        remove_all("./tests/simple_project/target");

        deadlinks()
            .current_dir("./tests/simple_project")
            .assert()
            .success();

        assert_doc("./tests/simple_project", &[]).success();

        // TODO: test that the docs aren't rebuilt
        remove_all("./tests/simple_project/target2");
        assert_doc("./tests/simple_project", &[("CARGO_TARGET_DIR", "target2")]).success();

        remove_all("./tests/simple_project/target");
        assert_doc(
            "./tests/simple_project",
            &[("CARGO_BUILD_TARGET", "x86_64-unknown-linux-gnu")],
        )
        .success();

        // fn it_shortens_path_on_error
        remove_all("./tests/simple_project/target");
        assert_doc("./tests/simple_project", &[]).success();
        std::fs::remove_file("./tests/simple_project/target/doc/simple_project/fn.bar.html")
            .unwrap();

        // without --debug, paths are shortened
        // NOTE: uses `deadlinks` to avoid rebuilding the docs
        Command::cargo_bin("deadlinks")
            .unwrap()
            .arg("./tests/simple_project/target/doc/simple_project")
            .assert()
            .failure()
            .stdout(
                contains("Linked file at path fn.bar.html does not exist!")
                    .and(contains("Found invalid urls in fn.foo.html:")),
            );

        // with --debug, paths are not shortened
        Command::cargo_bin("deadlinks")
            .unwrap()
            .arg("--debug")
            .arg("./tests/simple_project/target/doc/simple_project")
            .assert()
            .failure()
            .stdout(
                contains(
                    "cargo-deadlinks/tests/simple_project/ta\
                  rget/doc/simple_project/fn.foo.html:",
                )
                .and(contains(
                    "cargo-deadlinks/tests/simple_proj\
                    ect/target/doc/simple_project/fn.bar.html does not exist!",
                )),
            );
    }
}

mod renamed_project {
    use super::*;

    #[test]
    fn it_follows_package_renames() {
        remove_all("./tests/renamed_package/target");
        assert_doc("./tests/renamed_package", &[]).success();
    }
}

mod workspace {
    use super::*;

    #[test]
    fn it_checks_workspaces() {
        remove_all("./tests/workspace/target");
        assert_doc("./tests/workspace", &[]).success();
    }
}

mod cli_args {
    use super::*;

    #[test]
    fn it_passes_arguments_through_to_cargo() {
        remove_all("./tests/cli_args/target");
        deadlinks()
            .current_dir("./tests/cli_args")
            .arg("--")
            .arg("--document-private-items")
            .assert()
            .success();
        assert!(Path::new("./tests/cli_args/target/doc/cli_args/struct.Private.html").exists());
    }

    #[test]
    fn it_exits_with_success_on_info_queries() {
        for arg in &["-h", "--help", "-V", "--version"] {
            deadlinks().arg(arg).assert().success();
        }
    }

    #[test]
    fn dir_works() {
        deadlinks()
            .arg("--dir")
            .arg("./tests/broken_links/hardcoded-target")
            .assert()
            .failure()
            .stdout(contains("Found invalid urls"));
    }

    #[test]
    fn missing_deadlinks_gives_helpful_error() {
        Command::cargo_bin("cargo-deadlinks")
            .unwrap()
            .assert()
            .failure()
            .stderr(contains("should be run as `cargo deadlinks`"));
    }

    #[test]
    fn too_many_args_is_an_error() {
        deadlinks()
            .arg("x")
            .assert()
            .failure()
            .stderr(contains("error:").and(contains("x")));
    }

    #[test]
    fn version_contains_binary_name() {
        Command::cargo_bin("deadlinks")
            .unwrap()
            .arg("--version")
            .assert()
            .stdout(starts_with("deadlinks "));
        Command::cargo_bin("cargo-deadlinks")
            .unwrap()
            .arg("deadlinks")
            .arg("--version")
            .assert()
            .stdout(starts_with("cargo-deadlinks "));
    }
}
