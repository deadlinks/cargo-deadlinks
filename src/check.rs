//! Provides functionality for checking the availablility of URLs.
use std::{fmt, path::PathBuf};

use reqwest::{self, StatusCode};
use url::Url;

use super::CheckContext;

const PREFIX_BLACKLIST: [&str; 1] = ["https://doc.rust-lang.org"];

#[derive(Debug)]
pub enum HttpError {
    UnexpectedStatus(Url, StatusCode),
    Fetch(Url, reqwest::Error),
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HttpError::UnexpectedStatus(url, status) => {
                write!(f, "Unexpected HTTP status fetching {}: {}", url, status)
            }
            HttpError::Fetch(url, e) => write!(f, "Error fetching {}: {}", url, e),
        }
    }
}

#[derive(Debug)]
pub enum CheckError {
    File(PathBuf),
    Http(HttpError),
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CheckError::File(path) => {
                write!(f, "Linked file at path {} does not exist!", path.display())
            }
            CheckError::Http(err) => err.fmt(f),
        }
    }
}

/// Check a single URL for availability. Returns `false` if it is unavailable.
pub fn is_available(url: &Url, ctx: &CheckContext) -> Result<(), CheckError> {
    match url.scheme() {
        "file" => check_file_url(url, ctx),
        "http" | "https" => check_http_url(url, ctx),
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

    if path.exists() {
        debug!("Linked file at path {} does exist.", path.display());
        Ok(())
    } else {
        debug!("Linked file at path {} does not exist!", path.display());
        Err(CheckError::File(path))
    }
}

/// Check a URL with "http" or "https" scheme for availability. Returns `false` if it is unavailable.
fn check_http_url(url: &Url, ctx: &CheckContext) -> Result<(), CheckError> {
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

    let resp = reqwest::blocking::get(url.as_str());
    match resp {
        Ok(r) => {
            if r.status() == StatusCode::OK {
                Ok(())
            } else {
                Err(CheckError::Http(HttpError::UnexpectedStatus(
                    url.clone(),
                    r.status(),
                )))
            }
        }
        Err(e) => Err(CheckError::Http(HttpError::Fetch(url.clone(), e))),
    }
}
