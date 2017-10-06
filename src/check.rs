//! Provides functionality for checking the availablility of URLs.
use url::Url;

/// Checks multiple URLs for availability. Returns `false` if at least one ULR is unavailable.
pub fn check_urls(urls: &[Url]) -> bool {
    let mut result = true;

    for url in urls {
        result &= check_url(url);
    }

    result
}

/// Check a single URL for availability. Returns `false` if it is unavailable.
fn check_url(url: &Url) -> bool {
    match &*url.scheme {
        "file" => {
            check_file_url(url)
        },
        "http" | "https" => {
            check_http_url(url)
        },
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
fn check_file_url(url: &Url) -> bool {
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
fn check_http_url(url: &Url) -> bool{
    debug!("Can't check http/https URLs yet. ({})", url);
    true
}
