//! Provides functionality for checking the availablility of URLs.
use std::{fmt, path::PathBuf};

use log::debug;
use url::Url;

use super::CheckContext;

#[derive(Debug)]
#[cfg(feature = "http-check")]
pub enum HttpError {
    UnexpectedStatus(Url, ureq::Response),
    Fetch(Url, ureq::Error),
}

#[cfg(feature = "http-check")]
impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HttpError::UnexpectedStatus(url, resp) => write!(
                f,
                "Unexpected HTTP status fetching {}: {}",
                url,
                resp.status_text()
            ),
            HttpError::Fetch(url, e) => write!(f, "Error fetching {}: {}", url, e),
        }
    }
}

#[derive(Debug)]
pub enum CheckError {
    File(PathBuf),
    #[cfg(feature = "http-check")]
    Http(Box<HttpError>),
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CheckError::File(path) => {
                write!(f, "Linked file at path {} does not exist!", path.display())
            }
            #[cfg(feature = "http-check")]
            CheckError::Http(err) => err.fmt(f),
        }
    }
}

/// Check a single URL for availability. Returns `false` if it is unavailable.
pub fn is_available(url: &Url, ctx: &CheckContext) -> Result<(), CheckError> {
    match url.scheme() {
        "file" => check_file_url(url, ctx),
        "http" | "https" => {
            #[cfg(feature = "http-check")]
            let res = check_http_url(url, ctx);
            #[cfg(not(feature = "http-check"))]
            let res = {
                log::warn!("ignoring HTTP URL {}", url);
                log::info!("you can enable http checking with `--features=http-check`");
                Ok(())
            };
            res
        }
        scheme @ "javascript" => {
            debug!("Not checking URL scheme {:?}", scheme);
            Ok(())
        }
        other => {
            debug!("Unrecognized URL scheme {:?}", other);
            Ok(())
        }
    }
}

/// Check a URL with the "file" scheme for availability. Returns `false` if it is unavailable.
fn check_file_url(url: &Url, _ctx: &CheckContext) -> Result<(), CheckError> {
    let path = url.to_file_path().unwrap();

    if path.is_file() || path.join("index.html").is_file() {
        debug!("Linked file at path {} does exist.", path.display());
        Ok(())
    } else {
        debug!("Linked file at path {} does not exist!", path.display());
        Err(CheckError::File(path))
    }
}

#[cfg(feature = "http-check")]
/// Check a URL with "http" or "https" scheme for availability. Returns `false` if it is unavailable.
fn check_http_url(url: &Url, ctx: &CheckContext) -> Result<(), CheckError> {
    const PREFIX_BLACKLIST: [&str; 1] = ["https://doc.rust-lang.org"];

    if !ctx.check_http {
        debug!(
            "Skip checking {} as checking of http URLs is turned off",
            url
        );
        return Ok(());
    }

    for blacklisted_prefix in PREFIX_BLACKLIST.iter() {
        if url.as_str().starts_with(blacklisted_prefix) {
            debug!(
                "Skip checking {} as URL prefix is on the builtin blacklist",
                url
            );
            return Ok(());
        }
    }

    let resp = ureq::head(url.as_str()).call();
    if resp.synthetic() {
        Err(CheckError::Http(Box::new(HttpError::Fetch(
            url.clone(),
            resp.into_synthetic_error().unwrap(),
        ))))
    } else if resp.ok() {
        Ok(())
    } else {
        Err(CheckError::Http(Box::new(HttpError::UnexpectedStatus(
            url.clone(),
            resp,
        ))))
    }
}

#[cfg(test)]
mod test {
    use super::{check_file_url, CheckContext};
    use std::env;
    use url::Url;

    fn test_check_file_url(path: &str) {
        let cwd = env::current_dir().unwrap();
        let url = Url::from_file_path(cwd.join(path)).unwrap();

        check_file_url(&url, &CheckContext { check_http: false }).unwrap();
    }

    #[test]
    fn test_file_path() {
        test_check_file_url("tests/html/index.html");
    }

    #[test]
    fn test_directory_path() {
        test_check_file_url("tests/html/");
    }
}
