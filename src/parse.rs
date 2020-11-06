use std::collections::HashSet;
use std::path::Path;

use html5ever::parse_document;
use html5ever::rcdom::{Handle, NodeData, RcDom};
use html5ever::tendril::TendrilSink;
use log::{debug, info};
use url::Url;

/// Parse the html file at the provided path and check the availablility of all links in it.
pub fn parse_html_file(path: &Path) -> HashSet<Url> {
    info!("Checking doc page at {}", path.display());
    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .from_file(path)
        .unwrap();

    let base_url = Url::from_file_path(path).unwrap();
    let mut urls = HashSet::new();
    parse_a_hrefs(&dom.document, &base_url, &mut urls);
    urls
}

/// Traverse the DOM of a parsed HTML element, extracting all URLs from <a href="xxx"> links.
fn parse_a_hrefs(handle: &Handle, base_url: &Url, urls: &mut HashSet<Url>) {
    let node = handle;
    if let NodeData::Element {
        ref name,
        ref attrs,
        ..
    } = node.data
    {
        if &name.local == "a" {
            if let Some(attr) = attrs
                .borrow()
                .iter()
                .find(|attr| &attr.name.local == "href")
            {
                let mut val = attr.value.clone();
                // Treat absolute paths as absolute with respect to the `base_url`, not with respect to the file system.
                if attr.value.starts_with('/') {
                    val.pop_front_char();
                }

                if let Ok(link) = base_url.join(&val) {
                    debug!("link is {:?}", link);
                    urls.insert(link);
                } else {
                    debug!("unparsable link {:?}", val);
                }
            }
        }
    }

    for child in node.children.borrow().iter() {
        parse_a_hrefs(&child, base_url, urls);
    }
}

#[cfg(test)]
mod test {
    use html5ever::parse_document;
    use html5ever::{rcdom::RcDom, tendril::TendrilSink};
    use std::collections::HashSet;
    use url::Url;

    use super::parse_a_hrefs;

    #[test]
    fn test_parse_a_hrefs() {
        let html = r#"
        <!DOCTYPE html>
        <html>
            <body>
                <a href="a.html">a</a>
                <a href="/b/c.html">a</a>
            </body>
        </html> 
        "#;

        let dom = parse_document(RcDom::default(), Default::default())
            .from_utf8()
            .read_from(&mut html.as_bytes())
            .unwrap();

        let base_url = Url::from_directory_path("/base").unwrap();

        let mut urls = HashSet::new();
        parse_a_hrefs(&dom.document, &base_url, &mut urls);

        assert!(urls.contains(&Url::from_file_path("/base/a.html").unwrap()));
        assert!(urls.contains(&Url::from_file_path("/base/b/c.html").unwrap()));
    }
}
