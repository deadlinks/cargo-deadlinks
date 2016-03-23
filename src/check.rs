//! Provides functionality for checking the availablility of URLs.
use url::Url;

/// Checks multiple URLs for availablility.
pub fn check_urls(urls: &[Url]) {
    for url in urls {
        check_url(url);
    }
}

/// Check a single URL for availablility.
fn check_url(url: &Url) {
    match &*url.scheme {
        "file" => {
            check_file_url(url);
        },
        "http" | "https" => {
            check_http_url(url);
        },
        scheme @ "javascript" => {
            debug!("Not checking URL scheme {:?}", scheme);
        }
        other => {
            debug!("Unrecognized URL scheme {:?}", other);
        }
    }
}

/// Check a URL with the "file" scheme for availablility.
fn check_file_url(url: &Url) {
    let path = url.to_file_path().unwrap();
    match path.exists() {
        false => {
            error!("Linked file at path {} does not exist!", path.display());
        }
        true => {
            debug!("Linked file at path {} does exist.", path.display());
        }
    }
}

/// Check a URL with "http" or "https" scheme for availablility.
fn check_http_url(url: &Url) {
    debug!("Can't check http/https URLs yet. ({})", url);
}
