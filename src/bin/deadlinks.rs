use std::path::PathBuf;
use std::process;

use cargo_deadlinks::{walk_dir, CheckContext};
use serde_derive::Deserialize;

mod shared;

const MAIN_USAGE: &str = "
Check your package's documentation for dead links.

Usage:
    deadlinks [options] <directory>...

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
    arg_directory: Vec<PathBuf>,
    flag_verbose: bool,
    flag_debug: bool,
    flag_check_http: bool,
    flag_ignore_fragments: bool,
}

impl From<&MainArgs> for CheckContext {
    fn from(args: &MainArgs) -> CheckContext {
        CheckContext {
            check_http: args.flag_check_http,
            verbose: args.flag_debug,
            check_fragments: !args.flag_ignore_fragments,
        }
    }
}

fn parse_args() -> Result<MainArgs, pico_args::Error> {
    let mut args = pico_args::Arguments::from_env();
    if args.contains(["-V", "--version"]) {
        println!(concat!("deadlinks ", env!("CARGO_PKG_VERSION")));
        std::process::exit(0);
    } else if args.contains(["-h", "--help"]) {
        println!("{}", MAIN_USAGE);
        std::process::exit(0);
    }
    Ok(MainArgs {
        flag_verbose: args.contains(["-v", "--verbose"]),
        flag_debug: args.contains("--debug"),
        flag_ignore_fragments: args.contains("--ignore-fragments"),
        flag_check_http: args.contains("--check-http"),
        arg_directory: args.free_os()?.into_iter().map(Into::into).collect(),
    })
}

fn main() {
    let args = match parse_args() {
        Ok(args) => args,
        Err(err) => {
            println!("error: {}", err);
            process::exit(1);
        }
    };
    if args.arg_directory.is_empty() {
        eprintln!("error: missing <directory> argument");
        process::exit(1);
    }
    shared::init_logger(args.flag_debug, args.flag_verbose, "deadlinks");

    let mut errors = false;
    let ctx = CheckContext::from(&args);
    for relative_dir in args.arg_directory {
        let dir = match relative_dir.canonicalize() {
            Ok(dir) => dir,
            Err(_) => {
                println!("Could not find directory {:?}.", relative_dir);
                process::exit(1);
            }
        };
        log::info!("checking directory {:?}", dir);
        errors |= walk_dir(&dir, ctx.clone());
    }
    if errors {
        process::exit(1);
    }
}
