use std::collections::HashSet;

use log::debug;
use lol_html::{element, RewriteStrSettings};
use once_cell::sync::Lazy;
use regex::Regex;
use url::Url;

use crate::CheckError;

/// Return all broken intra-doc links in the source (of the form ``[`x`]``),
/// which presumably should have been resolved by rustdoc.
pub fn broken_intra_doc_links(html: &str) -> Vec<CheckError> {
    static BROKEN_INTRA_DOC_LINK: Lazy<Regex> =
        Lazy::new(|| Regex::new(r#"\[<code>(.*)</code>\]"#).unwrap());
    BROKEN_INTRA_DOC_LINK
        .captures_iter(&html)
        .map(|captures| CheckError::IntraDocLink(captures.get(0).unwrap().as_str().to_owned()))
        .collect()
}

/// Return all links in the HTML file, whether or not they are broken.
///
/// `root_url` is a fixed path relative to the documentation directory. For `target/doc/crate_x/y`, it's `crate_x`.
/// `file_url` is the file path relative to the documentation directory; it's different for each file.
/// For `target/doc/crate_x/y`, it's `crate_x/y`.
/// In general, `file_url.starts_with(root_url)` should always be true.
pub fn parse_a_hrefs(html: &str, root_url: &Url, file_url: &Url) -> HashSet<Url> {
    let mut urls = HashSet::new();
    lol_html::rewrite_str(
        html,
        RewriteStrSettings {
            element_content_handlers: vec![element!("a[href]", |el| {
                let href = el.get_attribute("href").unwrap();
                // base is the file path, unless path is absolute (starts with /)
                let (base, href) = if let Some(absolute) = href.strip_prefix('/') {
                    // Treat absolute paths as absolute with respect to the `root_url`, not with respect to the file system.
                    (&root_url, absolute)
                } else {
                    (&file_url, href.as_str())
                };

                if let Ok(link) = base.join(&href) {
                    debug!("link is {:?}", link);
                    urls.insert(link);
                } else {
                    debug!("unparsable link {:?}", href);
                }
                Ok(())
            })],
            ..RewriteStrSettings::default()
        },
    )
    .expect("html rewriting failed");

    urls
}

/// Parses the given string as HTML and returns values of all element's id attributes
pub(crate) fn parse_fragments(html: &str) -> HashSet<String> {
    let mut fragments = HashSet::new();
    lol_html::rewrite_str(
        html,
        RewriteStrSettings {
            element_content_handlers: vec![element!("*[id]", |el| {
                let id = el.get_attribute("id").unwrap();
                fragments.insert(id);
                Ok(())
            })],
            ..RewriteStrSettings::default()
        },
    )
    .expect("html rewriting failed");

    fragments
}

#[cfg(test)]
mod test {
    use super::{parse_a_hrefs, parse_fragments};
    use url::Url;

    #[test]
    fn test_parse_a_hrefs() {
        let html = r#"
        <!DOCTYPE html>
        <html>
            <body>
                <a href="a.html">a</a>
                <a href="/b/c.html">a</a>
            </body>
        </html>"#;

        let urls = parse_a_hrefs(
            html,
            &Url::from_directory_path("/base").unwrap(),
            &Url::from_file_path("/base/test.html").unwrap(),
        );

        assert!(urls.contains(&Url::from_file_path("/base/a.html").unwrap()));
        assert!(urls.contains(&Url::from_file_path("/base/b/c.html").unwrap()));
    }

    #[test]
    fn test_parse_a_hrefs_in_subdirectory() {
        let html = r#"
        <!DOCTYPE html>
        <html>
            <body>
                <a href="a.html">a</a>
                <a href="/b/c.html">a</a>
                <a href="../d.html">d</a>
            </body>
        </html>"#;

        let urls = parse_a_hrefs(
            html,
            &Url::from_directory_path("/root").unwrap(),
            &Url::from_file_path("/root/base/test.html").unwrap(),
        );

        assert!(urls.contains(&Url::from_file_path("/root/base/a.html").unwrap()));
        assert!(urls.contains(&Url::from_file_path("/root/b/c.html").unwrap()));
        assert!(urls.contains(&Url::from_file_path("/root/d.html").unwrap()));
    }

    #[test]
    fn test_parse_fragments() {
        let html = r#"
        <!DOCTYPE html>
        <html>
            <body>
                <a id="a">a</a>
                <h1 id="h1">h1</h1>
            </body>
        </html>"#;

        let fragments = parse_fragments(html);

        assert!(fragments.contains("a"));
        assert!(fragments.contains("h1"));
    }
}
