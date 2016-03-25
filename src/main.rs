#[macro_use]
extern crate log;
extern crate env_logger;
extern crate rustc_serialize;
extern crate docopt;

extern crate html5ever;
extern crate url;

#[macro_use]
extern crate string_cache;
extern crate tendril;

extern crate cargo_edit;

mod check;
mod parse;

use std::path::Path;

use log::LogLevelFilter;
use env_logger::LogBuilder;
use docopt::Docopt;

use cargo_edit::Manifest;

use check::check_urls;
use parse::parse_html_file;

const MAIN_USAGE: &'static str = "
Check your package's documentation for dead links.

Usage:
    cargo deadlinks [--dir <directory>] [options]

Options:
    -h --help               Print this message
    --dir                   Specify a directory to check (default is target/doc/<package>)
    --debug                 Use debug output
    -v, --verbose           Use verbose output
";

#[derive(Debug, RustcDecodable)]
struct MainArgs {
    arg_directory: Option<String>,
    flag_verbose: bool,
    flag_debug: bool,
}

fn main() {
    let args: MainArgs = Docopt::new(MAIN_USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    init_logger(&args);

    let dir = determine_dir(args.arg_directory);
    let dir_path = Path::new(&dir);
    walk_dir(dir_path);
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
fn determine_dir(arg_dir: Option<String>) -> String {
    let pwd_out = String::from_utf8(::std::process::Command::new("pwd").output().unwrap().stdout).unwrap();
    let pwd = &pwd_out[0..pwd_out.len() - 1];
    if arg_dir.is_some() {
        return Path::new(pwd).join(arg_dir.unwrap()).to_str().unwrap().to_owned();
    }

    match Manifest::open(&Some(pwd)) {
        Ok(manifest) => {
            let mut package_name = manifest.data.get("package").unwrap().as_table().unwrap()
                                                .get("name").unwrap().as_str().unwrap()
                                                .to_owned();
            package_name = package_name.replace("-", "_");

            let default_path = Path::new(pwd).join("target").join("doc").join(package_name);
            default_path.to_str().unwrap().to_string()
        },
        Err(err) => {
            debug!("Error: {:?}", err);
            error!("Could not find a Cargo.toml.");
            ::std::process::exit(1);
        }
    }

}

/// Traverses a given path recursively, checking all *.html files found.
fn walk_dir(dir_path: &Path) {
    match dir_path.read_dir() {
        Ok(read_dir) => {
            for dir_entry in read_dir {
                match dir_entry {
                    Ok(entry) => {
                        if entry.file_type().unwrap().is_file() {
                            let entry_path = entry.path();
                            let extension = entry_path.extension();
                            if extension.is_some() && extension.unwrap() == "html" {
                                let urls = parse_html_file(&entry.path());
                                check_urls(&urls);
                            }
                        } else {
                            walk_dir(&entry.path());
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
            error!("Could not read directory {}. Did you run `cargo doc`?", dir_path.to_str().unwrap());
        }
    }
}
