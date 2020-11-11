use std::collections::HashSet;
use std::path::Path;

use log::{debug, info};
use lol_html::{element, RewriteStrSettings};
use url::Url;

/// Parse the html file at the provided path and check the availablility of all links in it.
pub fn parse_html_file(root_dir: &Path, path: &Path) -> HashSet<Url> {
    info!("Checking doc page at {}", path.display());
    let html = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("{} did not contain valid UTF8: {}", path.display(), e));

    // root_url is absolute *relative to* the documentation directory. For `target/dir/crate_x/y`, it's `crate_x`.
    let root_url = Url::from_directory_path(root_dir).unwrap();
    // base_url is the relative file path. For `target/dir/crate_x/y`, it's `crate_x/y`.
    let base_url = Url::from_file_path(path).unwrap();

    parse_a_hrefs(&html, root_url, base_url)
}

/// This is a pure function, unlike `parse_html_file`, allowing it to be easily tested.
fn parse_a_hrefs(html: &str, root_url: Url, base_url: Url) -> HashSet<Url> {
    let mut urls = HashSet::new();
    lol_html::rewrite_str(
        html,
        RewriteStrSettings {
            element_content_handlers: vec![element!("a[href]", |el| {
                let href = el.get_attribute("href").unwrap();
                // base is the file path, unless path is absolute (starts with /)
                let (base, href) = if href.starts_with('/') {
                    // Treat absolute paths as absolute with respect to the `root_url`, not with respect to the file system.
                    (&root_url, &href[1..])
                } else {
                    (&base_url, href.as_str())
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
            Url::from_directory_path("/base").unwrap(),
            Url::from_file_path("/base/test.html").unwrap(),
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
            Url::from_directory_path("/root").unwrap(),
            Url::from_file_path("/root/base/test.html").unwrap(),
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
