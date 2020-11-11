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
#[derive(Clone, Debug, Default)]
pub struct CheckContext {
    pub check_http: bool,
    pub verbose: bool,
}

#[derive(Debug)]
pub struct FileError {
    pub path: PathBuf,
    pub errors: Vec<CheckError>,
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.print_shortened(None))?;
        Ok(())
    }
}

/// Traverses a given path recursively, checking all *.html files found.
///
/// For each error that occurred, print an error message.
/// Returns whether an error occurred.
pub fn walk_dir(dir_path: &Path, ctx: CheckContext) -> bool {
    let pool = ThreadPoolBuilder::new()
        .num_threads(num_cpus::get())
        .build()
        .unwrap();

    pool.install(|| {
        unavailable_urls(dir_path, &ctx)
            .map(|err| {
                if ctx.verbose {
                    println!("{}", err);
                } else {
                    println!("{}", err.print_shortened(Some(dir_path)));
                }
                true
            })
            // ||||||
            .reduce(|| false, |initial, new| initial || new)
    })
}

impl FileError {
    pub fn print_shortened(&self, prefix: Option<&Path>) -> String {
        let prefix = prefix.unwrap_or_else(|| Path::new(""));
        let filepath = self.path.strip_prefix(&prefix).unwrap_or(&self.path);
        let mut ret = format!("Found invalid urls in {}:", filepath.display());

        for e in &self.errors {
            use CheckError::*;

            match e {
                File(epath) => {
                    let epath = epath.strip_prefix(&prefix).unwrap_or(&epath);
                    ret.push_str("\n\tLinked file at path ");
                    ret.push_str(&epath.display().to_string());
                    ret.push_str(" does not exist!");
                }
                Http(_) => ret.push_str(&format!("\n\t{}", e)),
                Fragment(_, _, _) => {
                    ret.push_str("\n\t");
                    ret.push_str(e.to_string().as_str());
                }
                Io(_) => ret.push_str(&format!("\n\t{}", e)),
            }
        }
        ret
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
