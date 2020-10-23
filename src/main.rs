extern crate docopt;
extern crate env_logger;
#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;

extern crate html5ever;
extern crate url;

extern crate cargo_metadata;
extern crate num_cpus;
extern crate rayon;
extern crate reqwest;
extern crate serde_json;

extern crate cargo_deadlinks;

use std::path::{Path, PathBuf};
use std::process;

use cargo_metadata::Metadata;
use docopt::Docopt;
use log::LevelFilter;

use rayon::{prelude::*, ThreadPoolBuilder};

use cargo_deadlinks::{unavailable_urls, CheckContext};

const MAIN_USAGE: &str = "
Check your package's documentation for dead links.

Usage:
    cargo deadlinks [--dir <directory>] [options]

Options:
    -h --help               Print this message
    --dir                   Specify a directory to check (default is target/doc/<package>)
    --check-http            Check 'http' and 'https' scheme links
    --debug                 Use debug output
    -v --verbose            Use verbose output
    -V --version            Print version info and exit.
";

#[derive(Debug, Deserialize)]
struct MainArgs {
    arg_directory: Option<String>,
    flag_verbose: bool,
    flag_debug: bool,
    flag_check_http: bool,
}

impl Into<CheckContext> for MainArgs {
    fn into(self) -> CheckContext {
        CheckContext {
            check_http: self.flag_check_http,
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

    init_logger(&args);

    let dirs = args
        .arg_directory
        .as_ref()
        .map_or_else(determine_dir, |dir| vec![dir.into()]);

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
        if walk_dir(&dir, &args) {
            errors = true;
        }
    }
    if errors {
        process::exit(1);
    }
}

pub fn metadata_run(additional_args: Option<String>) -> Result<Metadata, ()> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| String::from("cargo"));
    let mut cmd = std::process::Command::new(cargo);
    cmd.arg("metadata");
    cmd.args(&["--format-version", "1"]);
    if let Some(additional_args) = additional_args {
        cmd.arg(&additional_args);
    }

    let fail_msg = "failed to run `cargo metadata` - do you have cargo installed?";
    let output = cmd
        .stdout(process::Stdio::piped())
        .spawn()
        .expect(fail_msg)
        .wait_with_output()
        .expect(fail_msg);

    if !output.status.success() {
        // don't need more info because we didn't capture stderr;
        // hopefully `cargo metadata` gave a useful error, but if not we can't do
        // anything
        return Err(());
    }

    let stdout = std::str::from_utf8(&output.stdout).expect("invalid UTF8");
    Ok(serde_json::from_str(stdout).expect("invalid JSON"))
}

/// Initalizes the logger according to the provided config flags.
fn init_logger(args: &MainArgs) {
    use std::io::Write;

    let mut builder = env_logger::Builder::new();
    builder.format(|f, record| writeln!(f, "{}", record.args()));
    match (args.flag_debug, args.flag_verbose) {
        (true, _) => {
            builder.filter(Some("cargo_deadlinks"), LevelFilter::Debug);
        }
        (false, true) => {
            builder.filter(Some("cargo_deadlinks"), LevelFilter::Info);
        }
        (false, false) => {
            builder.filter(Some("cargo_deadlinks"), LevelFilter::Error);
        }
    }
    builder.init();
}

/// Returns the directory to use as root of the documentation.
///
/// If an directory has been provided as CLI argument that one is used.
/// Otherwise we try to find the `Cargo.toml` and construct the documentation path
/// from the package name found there.
///
/// All *.html files under the root directory will be checked.
fn determine_dir() -> Vec<PathBuf> {
    let manifest = metadata_run(None).unwrap_or_else(|()| {
        error!("help: if this is not a cargo directory, use `--dir`");
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

/// Traverses a given path recursively, checking all *.html files found.
///
/// Returns whether an error occurred.
fn walk_dir(dir_path: &Path, args: &MainArgs) -> bool {
    let pool = ThreadPoolBuilder::new()
        .num_threads(num_cpus::get())
        .build()
        .unwrap();

    let ctx = CheckContext {
        check_http: args.flag_check_http,
    };
    pool.install(|| {
        unavailable_urls(dir_path, &ctx)
            .map(|err| {
                if args.flag_debug {
                    error!("{}", err);
                } else {
                    error!("{}", err.print_shortened(Some(dir_path)));
                }
                true
            })
            // ||||||
            .reduce(|| false, |initial, new| initial || new)
    })
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
