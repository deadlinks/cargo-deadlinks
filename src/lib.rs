use std::{
    fmt,
    path::{Path, PathBuf},
};

use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use walkdir::{DirEntry, WalkDir};

use check::is_available;
use parse::parse_html_file;

pub use check::{CheckError, IoError};

mod check;
mod parse;

// NOTE: this could be Copy, but we intentionally choose not to guarantee that.
#[derive(Clone, Debug)]
pub struct CheckContext {
    pub check_http: bool,
    pub verbose: bool,
    pub check_fragments: bool,
}

impl Default for CheckContext {
    fn default() -> Self {
        CheckContext {
            check_http: false,
            verbose: false,
            check_fragments: true,
        }
    }
}

#[derive(Debug)]
pub struct FileError {
    pub path: PathBuf,
    pub errors: Vec<CheckError>,
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Found invalid urls in {}:", self.path.display())?;
        for e in &self.errors {
            write!(f, "\n\t{}", e)?;
        }
        Ok(())
    }
}

/// Traverses a given path recursively, checking all *.html files found.
///
/// For each error that occurred, print an error message.
/// Returns whether an error occurred.
pub fn walk_dir(dir_path: &Path, ctx: &CheckContext) -> bool {
    let pool = ThreadPoolBuilder::new()
        .num_threads(num_cpus::get())
        .build()
        .unwrap();

    pool.install(|| {
        unavailable_urls(dir_path, ctx)
            .map(|mut err| {
                if !ctx.verbose {
                    err.shorten_all(dir_path);
                }
                println!("{}", err);
                true
            })
            // ||||||
            .reduce(|| false, |initial, new| initial || new)
    })
}

impl FileError {
    fn shorten_all(&mut self, prefix: &Path) {
        use check::Link;

        if let Ok(shortened) = self.path.strip_prefix(&prefix) {
            self.path = shortened.to_path_buf();
        };
        for mut e in &mut self.errors {
            if let CheckError::File(epath) | CheckError::Fragment(Link::File(epath), _, _) = &mut e
            {
                if let Ok(shortened) = epath.strip_prefix(prefix) {
                    *epath = shortened.to_path_buf();
                }
            }
        }
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
            let urls = parse_html_file(dir_path, entry.path());
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
                let path = entry.path().to_owned();
                Some(FileError { path, errors })
            }
        })
}
