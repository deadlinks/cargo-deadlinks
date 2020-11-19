use std::path::PathBuf;
use std::process;

use cargo_deadlinks::{walk_dir, CheckContext};
use docopt::Docopt;
use serde_derive::Deserialize;

mod shared;

const MAIN_USAGE: &str = "
Check your package's documentation for dead links.

Usage:
    deadlinks <directory> [options]

Options:
    -h --help               Print this message
    --check-http            Check 'http' and 'https' scheme links
    --ignore-fragments      Don't check URL fragments.
    --debug                 Use debug output
    -v --verbose            Use verbose output
    -V --version            Print version info and exit.
";

#[derive(Debug, Deserialize)]
struct MainArgs {
    arg_directory: PathBuf,
    flag_verbose: bool,
    flag_debug: bool,
    flag_check_http: bool,
    flag_ignore_fragments: bool,
}

impl From<MainArgs> for CheckContext {
    fn from(args: MainArgs) -> CheckContext {
        CheckContext {
            check_http: args.flag_check_http,
            verbose: args.flag_debug,
            check_fragments: !args.flag_ignore_fragments,
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
    shared::init_logger(args.flag_debug, args.flag_verbose, "deadlinks");

    let dir = match args.arg_directory.canonicalize() {
        Ok(dir) => dir,
        Err(_) => {
            println!("Could not find directory {:?}.", args.arg_directory);
            process::exit(1);
        }
    };
    log::info!("checking directory {:?}", dir);
    if walk_dir(&dir, args.into()) {
        process::exit(1);
    }
}
