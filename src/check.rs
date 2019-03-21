//! Provides functionality for checking the availablility of URLs.
use std::path::PathBuf;

use reqwest::{self, StatusCode};
use url::Url;

use super::CheckContext;

const PREFIX_BLACKLIST: [&'static str; 1] = ["https://doc.rust-lang.org"];

pub enum HttpError {
    UnexpectedStatus(Url, StatusCode),
    Fetch(Url, reqwest::Error),
}

pub enum CheckError {
    File(PathBuf),
    Http(HttpError),
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

    let resp = reqwest::get(url.as_str());
    match resp {
        Ok(r) => {
            if r.status() == StatusCode::Ok {
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
