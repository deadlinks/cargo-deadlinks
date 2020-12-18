//! Provides functionality for checking the availablility of URLs.
use std::collections::HashSet;
use std::fmt;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use log::{debug, warn};
use once_cell::sync::Lazy;
use regex::Regex;
use url::Url;

use cached::cached_key_result;
use cached::SizedCache;

use super::CheckContext;

use crate::parse::{parse_fragments, parse_redirect};

const PREFIX_BLACKLIST: [&str; 1] = ["https://doc.rust-lang.org"];

#[derive(Debug)]
pub enum IoError {
    HttpUnexpectedStatus(Url, ureq::Response),
    HttpFetch(Url, ureq::Error),
    FileIo(String, std::io::Error),
}

impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IoError::HttpUnexpectedStatus(url, resp) => write!(
                f,
                "Unexpected HTTP status fetching {}: {}",
                url,
                resp.status_text()
            ),
            IoError::HttpFetch(url, e) => write!(f, "Error fetching {}: {}", url, e),
            IoError::FileIo(url, e) => write!(f, "Error fetching {}: {}", url, e),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Link {
    File(PathBuf),
    Http(Url),
}

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Link::File(path) => write!(f, "{}", path.display()),
            Link::Http(url) => f.write_str(url.as_str()),
        }
    }
}

impl Link {
    /// Removes the fragment
    fn without_fragment(&self) -> Link {
        match self {
            Link::Http(url) => {
                let mut url = url.clone();
                url.set_fragment(None);

                Link::Http(url)
            }
            _ => self.clone(),
        }
    }
}

#[derive(Debug)]
pub enum CheckError {
    /// An intra-doc link went unresolved by rustdoc and ended up in the final HTML
    IntraDocLink(String),
    /// A relatively linked file did not exist
    File(PathBuf),
    /// A linked HTTP URL did not exist
    Http(Url),
    /// The linked file existed, but was missing the linked HTML anchor
    Fragment(Link, String, Option<Vec<String>>),
    /// An error occured while trying to find whether the file or URL existed
    Io(Box<IoError>),
}

impl fmt::Display for CheckError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CheckError::IntraDocLink(text) => {
                write!(f, "Broken intra-doc link to {}!", text)
            }
            CheckError::File(path) => {
                write!(f, "Linked file at path {} does not exist!", path.display())
            }
            CheckError::Http(url) => write!(f, "Linked URL {} does not exist!", url),
            CheckError::Fragment(link, fragment, missing_parts) => match missing_parts {
                Some(missing_parts) => write!(
                    f,
                    "Fragments #{} as expected by ranged fragment #{} at {} do not exist!\n\
                    This is likely a bug in rustdoc itself.",
                    missing_parts.join(", #"),
                    fragment,
                    link
                ),
                None => write!(f, "Fragment #{} at {} does not exist!", fragment, link),
            },
            CheckError::Io(err) => err.fmt(f),
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

cached_key_result! {
    CHECK_FILE: SizedCache<String, HashSet<String>> = SizedCache::with_size(100);
    Key = { link.without_fragment().to_string() };
    // `fetch_html` is different depending on whether the link is being
    // loaded from disk or from the network.
    fn fragments_from(
        link: &Link,
        fetch_html: impl Fn() -> Result<String, CheckError>
    ) -> Result<HashSet<String>, CheckError> = {
        fetch_html().map(|html| parse_fragments(&html))
    }
}

fn is_fragment_available(
    link: &Link,
    fragment: &str,
    fetch_html: impl Fn() -> Result<String, CheckError>,
) -> Result<(), CheckError> {
    // Empty fragments (e.g. file.html#) are commonly used to reach the top
    // of the document, see https://html.spec.whatwg.org/multipage/browsing-the-web.html#scroll-to-fragid
    if fragment.is_empty() {
        return Ok(());
    }

    let fragments = fragments_from(link, fetch_html)?;

    if fragments.contains(fragment) {
        return Ok(());
    }

    // Rust documentation uses `#n-m` fragments and JavaScript to highlight
    // a range of lines in HTML of source code, an element with `id`
    // attribute of (literal) "#n-m" will not exist, but elements with
    // `id`s n through m should, this parses the ranged n-m anchor and
    // checks if elements with `id`s n through m do exist
    static RUST_LINE_HIGLIGHT_RX: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"^(?P<start>[0-9]+)-(?P<end>[0-9]+)$"#).unwrap());
    match RUST_LINE_HIGLIGHT_RX.captures(fragment) {
        Some(capture) => match (capture.name("start"), capture.name("end")) {
            (Some(start_str), Some(end_str)) => {
                // NOTE: assumes there are less than 2.pow(32) lines in a source file
                let start = start_str.as_str().parse::<i32>().unwrap();
                let end = end_str.as_str().parse::<i32>().unwrap();
                let missing = (start..=end)
                    .map(|i| i.to_string())
                    .filter(|i| !fragments.contains(i))
                    .collect::<Vec<String>>();
                if !missing.is_empty() {
                    Err(CheckError::Fragment(
                        link.clone(),
                        fragment.to_string(),
                        Some(missing),
                    ))
                } else {
                    Ok(())
                }
            }
            _ => unreachable!("if the regex matches, it should have capture groups"),
        },
        None => Err(CheckError::Fragment(
            link.clone(),
            fragment.to_string(),
            None,
        )),
    }
}

/// Check a URL with the "file" scheme for availability. Returns `false` if it is unavailable.
fn check_file_url(url: &Url, ctx: &CheckContext) -> Result<(), CheckError> {
    let path = url.to_file_path().unwrap();

    // determine the full path by looking if the path points to a directory,
    // and if so append `index.html`, this is needed as we'll try to read
    // the file, so `expanded_path` should point to a file not a directory
    let index_html;
    let expanded_path = if path.is_file() {
        &path
    } else if path.is_dir() && path.join("index.html").is_file() {
        index_html = path.join("index.html");
        &index_html
    } else {
        debug!("Linked file at path {} does not exist!", path.display());
        return Err(CheckError::File(path));
    };

    if !ctx.check_fragments {
        return Ok(());
    }

    // The URL might contain a fragment. In that case we need a full GET
    // request to check if the fragment exists.
    match url.fragment() {
        Some(fragment) => check_file_fragment(&path, expanded_path, fragment),
        None => Ok(()),
    }
}

fn check_file_fragment(
    path: &Path,
    expanded_path: &Path,
    fragment: &str,
) -> Result<(), CheckError> {
    debug!(
        "Checking fragment {} of file {}.",
        fragment,
        expanded_path.display()
    );

    fn get_html(expanded_path: &Path) -> Result<String, CheckError> {
        read_to_string(expanded_path).map_err(|err| {
            CheckError::Io(Box::new(IoError::FileIo(
                expanded_path.to_string_lossy().to_string(),
                err,
            )))
        })
    }

    let fetch_html = || {
        let html = get_html(expanded_path)?;
        if let Some(redirect) = parse_redirect(&html) {
            get_html(&expanded_path.parent().unwrap().join(redirect))
        } else {
            Ok(html)
        }
    };
    is_fragment_available(&Link::File(path.to_path_buf()), fragment, fetch_html)
}

fn handle_response(url: &Url, resp: ureq::Response) -> Result<ureq::Response, CheckError> {
    if resp.synthetic() {
        Err(CheckError::Io(Box::new(IoError::HttpFetch(
            url.clone(),
            resp.into_synthetic_error().unwrap(),
        ))))
    } else if resp.ok() {
        Ok(resp)
    } else {
        Err(CheckError::Io(Box::new(IoError::HttpUnexpectedStatus(
            url.clone(),
            resp,
        ))))
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

    // The URL might contain a fragment. In that case we need a full GET
    // request to check if the fragment exists.
    if url.fragment().is_none() || !ctx.check_fragments {
        let resp = ureq::head(url.as_str()).call();

        handle_response(url, resp).map(|_: ureq::Response| ())
    } else {
        // the URL might contain a fragment, in that case we need to check if
        // the fragment exists, this issues a GET request
        check_http_fragment(url, url.fragment().unwrap())
    }
}

fn check_http_fragment(url: &Url, fragment: &str) -> Result<(), CheckError> {
    debug!("Checking fragment {} of URL {}.", fragment, url.as_str());

    fn get_html(url: &Url) -> Result<String, CheckError> {
        let resp = ureq::get(url.as_str()).call();
        handle_response(&url, resp).map(|resp| resp.into_string().unwrap())
    }

    let fetch_html = || {
        let html = get_html(url)?;
        // NOTE: only handles one level of nesting. Maybe we should have multiple levels?
        let redirect = parse_redirect(&html).and_then(|s| {
            Url::parse(&s)
                .map_err(|err| {
                    warn!("failed to parse Rustdoc redirect: {}", err);
                })
                .ok()
        });
        if let Some(redirect) = redirect {
            get_html(&redirect)
        } else {
            Ok(html)
        }
    };

    is_fragment_available(&Link::Http(url.clone()), fragment, fetch_html)
}

#[cfg(test)]
mod test {
    use super::{check_file_url, is_available, CheckContext, CheckError, Link};
    use mockito::{self, mock};
    use std::env;
    use url::Url;

    fn url_for(path: &str) -> Url {
        let cwd = env::current_dir().unwrap();
        let mut parts = path.split('#');
        let file_path = parts.next().unwrap();

        let mut url = if file_path.ends_with('/') {
            Url::from_directory_path(cwd.join(file_path))
        } else {
            Url::from_file_path(cwd.join(file_path))
        }
        .unwrap();

        url.set_fragment(parts.next());
        assert_eq!(parts.count(), 0); // make sure the anchor was valid, not `a.html#x#y`

        url
    }

    fn test_check_file_url(path: &str) -> Result<(), CheckError> {
        check_file_url(&url_for(path), &CheckContext::default())
    }

    #[test]
    fn test_file_path() {
        test_check_file_url("tests/html/index.html").unwrap();
    }

    #[test]
    fn test_directory_path() {
        test_check_file_url("tests/html/").unwrap();
    }

    #[test]
    fn test_anchors() {
        test_check_file_url("tests/html/anchors.html#h1").unwrap();
    }

    #[test]
    fn test_hash_fragment() {
        test_check_file_url("tests/html/anchors.html#").unwrap();
    }

    #[test]
    fn test_missing_anchors() {
        match test_check_file_url("tests/html/anchors.html#nonexistent") {
            Err(CheckError::Fragment(Link::File(path), fragment, None)) => {
                assert!(path.ends_with("tests/html/anchors.html"));
                assert_eq!("nonexistent", fragment);
            }
            x => panic!(
                "Expected to report missing anchor (Err(CheckError::FileAnchor)), got {:?}",
                x
            ),
        }
    }

    #[test]
    fn test_range_anchor() {
        test_check_file_url("tests/html/range.html#2-4").unwrap();
    }

    #[test]
    fn test_missing_range_anchor() {
        match test_check_file_url("tests/html/range.html#4-6") {
            Err(CheckError::Fragment(Link::File(path), fragment, Some(missing_parts))) => {
                assert!(path.ends_with("tests/html/range.html"));
                assert_eq!("4-6", fragment);
                assert_eq!(missing_parts.len(), 1);
                assert!(missing_parts.contains(&"6".to_string()));
            }
            x => panic!(
                "Expected to report missing anchor (Err(CheckError::FileAnchorRange)), got {:?}",
                x
            ),
        }
    }

    #[test]
    fn test_is_available_file_path() {
        is_available(
            &url_for("tests/html/index.html#i1"),
            &CheckContext::default(),
        )
        .unwrap();
    }

    #[test]
    fn test_is_available_directory_path() {
        is_available(&url_for("tests/html/#i1"), &CheckContext::default()).unwrap();
    }

    #[test]
    fn test_missing_dir_index_fragment() {
        match is_available(
            &url_for("tests/html/missing_index/#i1"),
            &CheckContext::default(),
        ) {
            Err(CheckError::File(path)) => assert!(path.ends_with("tests/html/missing_index")),
            x => panic!(
                "Expected to report missing anchor (Err(CheckError::File)), got {:?}",
                x
            ),
        }
    }

    #[test]
    fn test_http_check() {
        let root = mock("HEAD", "/test_http_check").with_status(200).create();

        let mut url = mockito::server_url();
        url.push_str("/test_http_check");

        is_available(
            &Url::parse(&url).unwrap(),
            &CheckContext {
                check_http: true,
                ..CheckContext::default()
            },
        )
        .unwrap();

        root.assert();
    }

    #[test]
    fn test_http_check_fragment() {
        let root = mock("GET", "/test_http_check_fragment")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(
                r#"<!DOCTYPE html>
            <html>
                <body id="r1" />
            </html>"#,
            )
            .create();

        let mut url = mockito::server_url();
        url.push_str("/test_http_check_fragment#r1");

        is_available(
            &Url::parse(&url).unwrap(),
            &CheckContext {
                check_http: true,
                ..CheckContext::default()
            },
        )
        .unwrap();

        root.assert();
    }

    #[test]
    fn test_missing_http_fragment() {
        let root = mock("GET", "/test_missing_http_fragment")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(
                r#"<!DOCTYPE html>
            <html />"#,
            )
            .create();

        let mut url = mockito::server_url();
        url.push_str("/test_missing_http_fragment#missing");

        match is_available(
            &Url::parse(&url).unwrap(),
            &CheckContext {
                check_http: true,
                ..CheckContext::default()
            },
        ) {
            Err(CheckError::Fragment(Link::Http(url), fragment, None)) => {
                assert_eq!(
                    "http://127.0.0.1:1234/test_missing_http_fragment#missing",
                    url.to_string()
                );
                assert_eq!("missing", fragment);
            }
            x => panic!(
                "Expected to report missing anchor (Err(CheckError::File)), got {:?}",
                x
            ),
        }

        root.assert();
    }

    #[test]
    fn test_disabling_fragment_checks_file() {
        check_file_url(
            &url_for("tests/html/anchors.html#nonexistent"),
            &CheckContext {
                check_fragments: false,
                ..CheckContext::default()
            },
        )
        .unwrap();
    }

    #[test]
    fn test_disabling_fragment_checks_http() {
        let root = mock("HEAD", "/test_disabling_fragment_checks_http")
            .with_status(200)
            .create();

        let mut url = mockito::server_url();
        url.push_str("/test_disabling_fragment_checks_http#missing");

        is_available(
            &Url::parse(&url).unwrap(),
            &CheckContext {
                check_http: true,
                check_fragments: false,
                ..CheckContext::default()
            },
        )
        .unwrap();

        root.assert();
    }
}
