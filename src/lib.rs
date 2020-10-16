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

use std::{
    fmt,
    path::{Path, PathBuf},
};

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

#[derive(Debug)]
pub struct FileError {
    pub path: PathBuf,
    pub errors: Vec<CheckError>,
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Found invalid urls in {}:", self.path.display(),)?;
        for err in &self.errors {
            writeln!(f)?;
            write!(f, "\t{}", err)?;
        }
        Ok(())
    }
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
) -> impl ParallelIterator<Item = FileError> + 'a {
    WalkDir::new(dir_path)
        .into_iter()
        .par_bridge()
        .filter_map(|e| e.ok())
        .filter(|entry| entry.file_type().is_file() && is_html_file(&entry))
        .flat_map(move |entry| {
            let urls = parse_html_file(entry.path());
            let errors = urls
                .into_iter()
                .filter_map(|url| match is_available(&url, &ctx) {
                    Ok(()) => None,
                    Err(err) => Some(err),
                })
                .collect::<Vec<_>>();

            if errors.is_empty() {
                None
            } else {
                Some(FileError {
                    path: entry.path().to_owned(),
                    errors,
                })
            }
        })
}
