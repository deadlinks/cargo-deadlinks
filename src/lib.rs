extern crate docopt;
extern crate env_logger;
#[macro_use]
extern crate log;

extern crate html5ever;
extern crate url;

extern crate cargo_metadata;
extern crate num_cpus;
extern crate rayon;
extern crate reqwest;
extern crate serde_json;
extern crate walkdir;

use std::path::Path;

use rayon::prelude::*;
use walkdir::{DirEntry, WalkDir};

use check::is_available;
use parse::parse_html_file;

pub use check::{CheckError, HttpError};

mod check;
mod parse;

#[derive(Debug)]
pub struct CheckContext {
    pub check_http: bool,
}

fn is_html_file(entry: &DirEntry) -> bool {
    match entry.path().extension() {
        Some(e) => e.to_str().map(|ext| ext == "html").unwrap_or(false),
        None => false,
    }
}

pub fn unavailable_urls<'a>(
    dir_path: &'a Path,
    ctx: &'a CheckContext,
) -> impl ParallelIterator<Item = CheckError> + 'a {
    WalkDir::new(dir_path)
        .into_iter()
        .par_bridge()
        .filter_map(|e| e.ok())
        .filter(|entry| entry.file_type().is_file() && is_html_file(&entry))
        .flat_map(|entry| parse_html_file(entry.path()))
        .filter_map(move |url| match is_available(&url, &ctx) {
            Ok(()) => None,
            Err(err) => Some(err),
        })
}
