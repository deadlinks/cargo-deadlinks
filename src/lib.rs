use std::{
    fmt,
    path::{Path, PathBuf},
};

use log::info;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use url::Url;
use walkdir::{DirEntry, WalkDir};

use check::is_available;

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
    let root_url = Url::from_directory_path(dir_path).unwrap();

    WalkDir::new(dir_path)
        .into_iter()
        .par_bridge()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file() && is_html_file(&entry))
        .flat_map(move |entry| {
            let path = entry.path();
            info!("Checking doc page at {}", path.display());
            let html = std::fs::read_to_string(path)
                .unwrap_or_else(|e| panic!("{} did not contain valid UTF8: {}", path.display(), e));

            let file_url = Url::from_file_path(path).unwrap();
            let urls = parse::parse_a_hrefs(&html, &root_url, &file_url);
            let broken_intra_doc_links = parse::broken_intra_doc_links(&html);
            let errors = urls
                .into_iter()
                .filter_map(|url| is_available(&url, &ctx).err())
                .chain(broken_intra_doc_links)
                .collect::<Vec<_>>();

            if errors.is_empty() {
                None
            } else {
                let path = entry.path().to_owned();
                Some(FileError { path, errors })
            }
        })
}
