//! <https://tinyurl.com/rnxcavf>
use std::{
    fmt,
    path::{Path, PathBuf},
};

use log::{debug, info};
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use url::Url;
use walkdir::{DirEntry, WalkDir};

use check::is_available;

pub use check::{CheckError, IoError};

mod check;
mod parse;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
/// What behavior should deadlinks use for HTTP links?
pub enum HttpCheck {
    /// Make an internet request to ensure the link works.
    Enabled,
    /// Do nothing when encountering a link.
    Ignored,
    /// Give an error when encountering a link.
    ///
    /// Note that even when HTTP links are forbidden, `doc.rust-lang.org` links are still assumed to
    /// be valid.
    Forbidden,
}

// NOTE: this could be Copy, but we intentionally choose not to guarantee that.
/// Link-checking options.
#[derive(Clone, Debug)]
pub struct CheckContext {
    /// Should deadlinks give more detail when checking links?
    ///
    /// Currently, 'more detail' just means not to abbreviate file paths when printing errors.
    pub verbose: bool,
    /// What behavior should deadlinks use for HTTP links?
    pub check_http: HttpCheck,
    /// Should fragments in URLs be checked?
    pub check_fragments: bool,
    pub check_intra_doc_links: bool,
    /// A list of files with ignored link fragments.
    pub ignored_links: Vec<IgnoredFile>,
    /// A list of files with ignored intra-doc links.
    pub ignored_intra_doc_links: Vec<IgnoredFile>,
}

/// A file to ignore.
#[derive(Clone, Debug)]
pub struct IgnoredFile {
    /// What file path should be ignored?
    pub path: PathBuf,
    /// What links in the file should be ignored?
    ///
    /// An empty list means all links should be ignored.
    pub links: Vec<String>,
}

impl Default for CheckContext {
    fn default() -> Self {
        CheckContext {
            check_http: HttpCheck::Ignored,
            verbose: false,
            check_fragments: true,
            check_intra_doc_links: false,
            ignored_links: Vec::new(),
            ignored_intra_doc_links: Vec::new(),
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
    debug!("ignored_links: {:?}", ctx.ignored_links);
    debug!("ignored_intra_doc_links: {:?}", ctx.ignored_intra_doc_links);

    let pool = ThreadPoolBuilder::new()
        .num_threads(num_cpus::get())
        .build()
        .unwrap();

    pool.install(|| {
        unavailable_urls(dir_path, ctx)
            .filter_map(|mut file_err| {
                let shortened_path = file_err.path.strip_prefix(dir_path).unwrap_or(dir_path);
                debug!("file_err={:?}, shortened_path={:?}", file_err, shortened_path);

                // First, filter out ignored errors
                if let Some(ignore) = ctx.ignored_links.iter().find(|ignore| ignore.path == shortened_path) {
                    file_err.errors.retain(|err| {
                        let should_ignore = if ignore.links.is_empty() {
                            // Ignore all links
                            matches!(err, CheckError::Http(_) | CheckError::File(_) | CheckError::Fragment(..))
                        } else {
                            // Ignore links that are present in the list
                            match err {
                                CheckError::Fragment(_, fragment, _) => ignore.links.iter().any(|link| {
                                    #[allow(clippy::or_fun_call)]
                                    let link = link.strip_prefix('#').unwrap_or(link.as_str());
                                    link == fragment
                                }),
                                CheckError::File(path) => ignore.links.iter().any(|link| Path::new(link) == path),
                                CheckError::Http(url) => ignore.links.iter().any(|link| link == url.as_str()),
                                CheckError::IntraDocLink(_) | CheckError::HttpForbidden(_) | CheckError::Io(_) => false,
                            }
                        };
                        !should_ignore
                    });
                }
                if let Some(ignore) = ctx.ignored_intra_doc_links.iter().find(|ignore| ignore.path == shortened_path) {
                    file_err.errors.retain(|err| {
                        let should_ignore = if ignore.links.is_empty() {
                            // Ignore all links
                            matches!(err, CheckError::IntraDocLink(_))
                        } else {
                            // Ignore links that are present in the list
                            match err {
                                CheckError::IntraDocLink(link) => ignore.links.contains(link),
                                _ => false,
                            }
                        };
                        !should_ignore
                    });
                }

                if file_err.errors.is_empty() {
                    return None;
                }

                // Next, print the error for display
                if !ctx.verbose {
                    file_err.shorten_all(dir_path);
                }
                println!("{}", file_err);
                Some(true)
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
            let broken_intra_doc_links = if ctx.check_intra_doc_links {
                parse::broken_intra_doc_links(&html)
            } else {
                Vec::new()
            };
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
