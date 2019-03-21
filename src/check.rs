//! Provides functionality for checking the availablility of URLs.

use reqwest::{self, StatusCode};
use url::Url;

use super::CheckContext;

const PREFIX_BLACKLIST: [&'static str; 1] = ["https://doc.rust-lang.org"];

/// Check a single URL for availability. Returns `false` if it is unavailable.
pub fn is_available(url: &Url, ctx: &CheckContext) -> bool {
    match url.scheme() {
        "file" => check_file_url(url, ctx),
        "http" | "https" => check_http_url(url, ctx),
        scheme @ "javascript" => {
            debug!("Not checking URL scheme {:?}", scheme);
            true
        }
        other => {
            debug!("Unrecognized URL scheme {:?}", other);
            true
        }
    }
}

/// Check a URL with the "file" scheme for availability. Returns `false` if it is unavailable.
fn check_file_url(url: &Url, _ctx: &CheckContext) -> bool {
    let path = url.to_file_path().unwrap();

    if path.exists() {
        debug!("Linked file at path {} does exist.", path.display());
        true
    } else {
        error!("Linked file at path {} does not exist!", path.display());
        false
    }
}

/// Check a URL with "http" or "https" scheme for availability. Returns `false` if it is unavailable.
fn check_http_url(url: &Url, ctx: &CheckContext) -> bool {
    if !ctx.check_http {
        debug!(
            "Skip checking {} as checking of http URLs is turned off",
            url
        );
        return true;
    }

    for blacklisted_prefix in PREFIX_BLACKLIST.iter() {
        if url.as_str().starts_with(blacklisted_prefix) {
            debug!(
                "Skip checking {} as URL prefix is on the builtin blacklist",
                url
            );
            return true;
        }
    }

    let resp = reqwest::get(url.as_str());
    match resp {
        Ok(r) => r.status() == StatusCode::Ok,
        Err(e) => {
            error!("Error fetching {}: {}", url, e);
            false
        }
    }
}
