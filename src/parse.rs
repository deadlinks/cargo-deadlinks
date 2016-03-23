use std::path::Path;

use html5ever::parse_document;
use html5ever::rcdom::{Element, RcDom, Node};
use tendril::TendrilSink;
use url::{UrlParser, Url};

/// Parse the html file at the provided path and check the availablility of all links in it.
pub fn parse_html_file(path: &Path) -> Vec<Url> {
    info!("Checking doc page at {}", path.display());
    let dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .from_file(path)
        .unwrap();

    let base_url = Url::from_file_path(path).unwrap();
    let mut urls = Vec::new();
    parse_a_hrefs(&dom.document.borrow(), &base_url, &mut urls);
    urls
}

/// Traverse the DOM of a parsed HTML element, extracting all URLs from <a href="xxx"> links.
fn parse_a_hrefs(node: &Node, base_url: &Url, urls: &mut Vec<Url>) {
    let mut parser = UrlParser::new();
    parser.base_url(&base_url);

    match node.node {
        Element(ref name, _, ref attrs) => {
            if &*name.local == "a" {
                if let Some(attr) = attrs.iter().find(|attr| &*attr.name.local == "href") {
                    let href_val = &*attr.value;

                    match parser.parse(href_val) {
                        Ok(parsed) => urls.push(parsed),
                        _ => (),
                    }
                }
            }
        }
        _ => {},
    }

    for child in &node.children {
        parse_a_hrefs(&child.borrow(), base_url, urls);
    }
}
