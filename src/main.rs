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
use env_logger::Builder;
use log::LevelFilter;

use rayon::{prelude::*, ThreadPoolBuilder};

use cargo_deadlinks::{unavailable_urls, CheckContext};

const MAIN_USAGE: &'static str = "
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

    let dir = args
        .arg_directory
        .clone()
        .map_or_else(determine_dir, |dir| PathBuf::from(dir));
    let dir = match dir.canonicalize() {
        Ok(dir) => dir,
        Err(_) => {
            println!("Could not find directory {:?}.", dir);
            println!("");
            println!("Please run `cargo doc` before running `cargo deadlinks`.");
            process::exit(1);
        }
    };
    let ctx: CheckContext = args.into();
    if !walk_dir(&dir, &ctx) {
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

    let output = cmd.output().unwrap();
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let meta = serde_json::from_str(stdout).unwrap();
    Ok(meta)
}

/// Initalizes the logger according to the provided config flags.
fn init_logger(args: &MainArgs) {
    use std::io::Write;

    let mut builder = Builder::new();
    builder.format(|buf, record| writeln!(buf, "{}", record.args()));
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
fn determine_dir() -> PathBuf {
    match metadata_run(None) {
        Ok(manifest) => {
            let package_name = &manifest[&manifest.workspace_members[0]].name;
            let package_name = package_name.replace("-", "_");

            Path::new("target").join("doc").join(package_name)
        }
        Err(_) => {
            error!("Could not find a Cargo.toml.");
            ::std::process::exit(1);
        }
    }
}

/// Traverses a given path recursively, checking all *.html files found.
fn walk_dir(dir_path: &Path, ctx: &CheckContext) -> bool {
    let pool = ThreadPoolBuilder::new()
        .num_threads(num_cpus::get())
        .build()
        .unwrap();

    pool.install(|| {
        !unavailable_urls(dir_path, ctx).any(|err| {
            error!("{}", err);
            true
        })
    })
}
