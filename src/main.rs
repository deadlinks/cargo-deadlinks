#[macro_use]
extern crate log;
extern crate env_logger;
extern crate rustc_serialize;
extern crate docopt;

extern crate html5ever;
extern crate url;

extern crate string_cache;
extern crate tendril;

extern crate cargo_edit;

mod check;
mod parse;

use std::path::{Path, PathBuf};
use std::process;

use log::LogLevelFilter;
use env_logger::LogBuilder;
use docopt::Docopt;

use cargo_edit::Manifest;

use check::check_urls;
use parse::parse_html_file;

const MAIN_USAGE: &'static str = "
Check your package's documentation for dead links.

Usage:
    cargo-deadlinks [--dir <directory>] [options]

Options:
    -h --help               Print this message
    --dir                   Specify a directory to check (default is target/doc/<package>)
    --debug                 Use debug output
    -v --verbose            Use verbose output
    -V --version            Print version info and exit.
";

#[derive(Debug, RustcDecodable)]
struct MainArgs {
    arg_directory: Option<String>,
    flag_verbose: bool,
    flag_debug: bool,
}

fn main() {
    let args: MainArgs = Docopt::new(MAIN_USAGE)
                            .and_then(|d| {
                                d.version(Some(env!("CARGO_PKG_VERSION").to_owned())).decode()
                            }).unwrap_or_else(|e| e.exit());

    init_logger(&args);

    let dir = args.arg_directory.map_or_else(determine_dir, |dir| PathBuf::from(dir));
    let dir = dir.canonicalize().unwrap();
    if !walk_dir(&dir) {
        process::exit(1);
    }
}

/// Initalizes the logger according to the provided config flags.
fn init_logger(args: &MainArgs) {
    let mut builder = LogBuilder::new();
    builder.format(|record| format!("{}", record.args()));
    match (args.flag_debug, args.flag_verbose) {
        (true, _) => { builder.filter(Some("cargo_deadlinks"), LogLevelFilter::Debug); },
        (false, true) => { builder.filter(Some("cargo_deadlinks"), LogLevelFilter::Info); },
        (false, false) => { builder.filter(Some("cargo_deadlinks"), LogLevelFilter::Error); },
    }
    builder.init().unwrap();
}

/// Returns the directory to use as root of the documentation.
///
/// If an directory has been provided as CLI argument that one is used.
/// Otherwise we try to find the `Cargo.toml` and construct the documentation path
/// from the package name found there.
///
/// All *.html files under the root directory will be checked.
fn determine_dir() -> PathBuf {
    match Manifest::open(&None) {
        Ok(manifest) => {
            let package_name = manifest.data.get("package").unwrap().as_table().unwrap()
                                            .get("name").unwrap().as_str().unwrap();
            let package_name = package_name.replace("-", "_");

            Path::new("target").join("doc").join(package_name)
        },
        Err(err) => {
            debug!("Error: {}", err);
            error!("Could not find a Cargo.toml.");
            ::std::process::exit(1);
        }
    }

}

/// Traverses a given path recursively, checking all *.html files found.
fn walk_dir(dir_path: &Path) -> bool {
    let mut result = true;

    match dir_path.read_dir() {
        Ok(read_dir) => {
            for dir_entry in read_dir {
                match dir_entry {
                    Ok(entry) => {
                        if entry.file_type().unwrap().is_file() {
                            let entry_path = entry.path();
                            let extension = entry_path.extension();
                            if extension == Some("html".as_ref()) {
                                let urls = parse_html_file(&entry.path());
                                result &= check_urls(&urls);
                            }
                        } else {
                            result &= walk_dir(&entry.path());
                        }
                    },
                    Err(err) => {
                        error!("Error when traversing directory: {}", err);
                    }
                }
            }
        }
        Err(err) => {
            debug!("{:?}", err);
            error!("Could not read directory {}. Did you run `cargo doc`?", dir_path.display());
        }
    }

    result
}
