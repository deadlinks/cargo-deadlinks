use std::path::Path;

use html5ever::parse_document;
use html5ever::rcdom::{Element, RcDom, Handle};
use tendril::TendrilSink;
use url::{UrlParser, Url};

/// Parse the html file at the provided path and check the availablility of all links in it.
pub fn parse_html_file(path: &Path) -> Vec<Url> {
    info!("Checking doc page at {}", path.to_str().unwrap());
    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .from_file(path)
        .unwrap();

    let base_url = Url::from_file_path(path).unwrap();
    parse_a_hrefs(dom.document, &base_url)
}

/// Traverse the DOM of a parsed HTML element, extracting all URLs from <a href="xxx"> links.
fn parse_a_hrefs(handle: Handle, base_url: &Url) -> Vec<Url> {
    let node = handle.borrow();

    let mut parser = UrlParser::new();
    parser.base_url(&base_url);

    let mut urls = Vec::new();
    match node.node {
        Element(ref name, _, ref attrs) => {
            if name.local.to_string() == "a" {
                for attr in attrs {
                    if attr.name.local.to_string() == "href" {
                        let href_val = attr.value.to_string();

                        match parser.parse(&href_val) {
                            Ok(parsed) => urls.push(parsed),
                            _ => (),
                        }
                    }
                }
            }
        }
        _ => {},
    }

    let child_urls = node.children.iter().flat_map(|child| {
        parse_a_hrefs(child.clone(), base_url)
    }).collect::<Vec<_>>();
    urls.extend_from_slice(&child_urls);
    urls
}
