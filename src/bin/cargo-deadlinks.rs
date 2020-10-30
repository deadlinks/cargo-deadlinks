use std::path::PathBuf;
use std::process;

use cargo_metadata::MetadataCommand;
use docopt::Docopt;
use serde_derive::Deserialize;

use cargo_deadlinks::{walk_dir, CheckContext};

mod shared;

const MAIN_USAGE: &str = "
Check your package's documentation for dead links.

Usage:
    cargo deadlinks [--dir <directory>] [options]

Options:
    -h --help               Print this message.
    --dir                   Specify a directory to check (default is target/doc/<package>).
    --check-http            Check 'http' and 'https' scheme links.
    --debug                 Use debug output. This option is deprecated; use `RUST_LOG=debug` instead.
    -v --verbose            Use verbose output. This option is deprecated; use `RUST_LOG==info` instead.
    -V --version            Print version info and exit.
";

#[derive(Debug, Deserialize)]
struct MainArgs {
    arg_directory: Option<String>,
    flag_verbose: bool,
    flag_debug: bool,
    flag_check_http: bool,
}

impl From<MainArgs> for CheckContext {
    fn from(args: MainArgs) -> CheckContext {
        CheckContext {
            check_http: args.flag_check_http,
            verbose: args.flag_debug,
        }
    }
}

fn main() {
    let args: MainArgs = Docopt::new(MAIN_USAGE)
        .and_then(|d| {
            d.version(Some(env!("CARGO_PKG_VERSION").to_owned()))
                .deserialize()
        })
        .unwrap_or_else(|e| e.exit());

    shared::init_logger(args.flag_debug, args.flag_verbose, "cargo_deadlinks");

    let dirs = args
        .arg_directory
        .as_ref()
        .map_or_else(determine_dir, |dir| vec![dir.into()]);

    let ctx = CheckContext::from(args);
    let mut errors = false;
    for dir in dirs {
        let dir = match dir.canonicalize() {
            Ok(dir) => dir,
            Err(_) => {
                println!("Could not find directory {:?}.", dir);
                println!();
                println!("Please run `cargo doc` before running `cargo deadlinks`.");
                process::exit(1);
            }
        };
        log::info!("checking directory {:?}", dir);
        if walk_dir(&dir, ctx.clone()) {
            errors = true;
        }
    }
    if errors {
        process::exit(1);
    }
}

/// Returns the directory to use as root of the documentation.
///
/// If an directory has been provided as CLI argument that one is used.
/// Otherwise we try to find the `Cargo.toml` and construct the documentation path
/// from the package name found there.
///
/// All *.html files under the root directory will be checked.
fn determine_dir() -> Vec<PathBuf> {
    let manifest = MetadataCommand::new()
        .no_deps()
        .exec()
        .unwrap_or_else(|err| {
            println!("error: {}", err);
            println!("help: if this is not a cargo directory, use `--dir`");
            ::std::process::exit(1);
        });
    let doc = manifest.target_directory.join("doc");

    // originally written with this impressively bad jq query:
    // `.packages[] |select(.source == null) | .targets[] | select(.kind[] | contains("test") | not) | .name`
    manifest
        .packages
        .into_iter()
        .filter(|package| package.source.is_none())
        .map(|package| package.targets)
        .flatten()
        .filter(has_docs)
        .map(|target| doc.join(target.name.replace('-', "_")))
        .collect()
}

fn has_docs(target: &cargo_metadata::Target) -> bool {
    // Ignore tests, examples, and benchmarks, but still document binaries

    // See https://doc.rust-lang.org/cargo/reference/external-tools.html#compiler-messages
    // and https://github.com/rust-lang/docs.rs/issues/503#issuecomment-562797599
    // for the difference between `kind` and `crate_type`

    let mut kinds = target.kind.iter();
    // By default, ignore binaries
    if target.crate_types.contains(&"bin".into()) {
        // But allow them if this is a literal bin, and not a test or example
        kinds.all(|kind| kind == "bin")
    } else {
        // We also have to consider examples and tests that are libraries
        // (e.g. because of `cdylib`).
        kinds.all(|kind| !["example", "test", "bench"].contains(&kind.as_str()))
    }
}

#[cfg(test)]
mod test {
    use super::has_docs;
    use cargo_metadata::Target;

    fn target(crate_types: &str, kind: &str) -> Target {
        serde_json::from_str(&format!(
            r#"{{
            "crate_types": ["{}"],
            "kind": ["{}"],
            "name": "simple",
            "src_path": "",
            "edition": "2018",
            "doctest": false,
            "test": false
        }}"#,
            crate_types, kind
        ))
        .unwrap()
    }

    #[test]
    fn finds_right_docs() {
        assert!(!has_docs(&target("cdylib", "example")));
        assert!(!has_docs(&target("bin", "example")));
        assert!(!has_docs(&target("bin", "test")));
        assert!(!has_docs(&target("bin", "bench")));
        assert!(!has_docs(&target("bin", "custom-build")));

        assert!(has_docs(&target("bin", "bin")));
        assert!(has_docs(&target("dylib", "dylib")));
        assert!(has_docs(&target("rlib", "rlib")));
        assert!(has_docs(&target("lib", "lib")));
        assert!(has_docs(&target("proc-macro", "proc-macro")));
    }
}
