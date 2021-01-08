/// Foo function
///
/// Has something to do with [some website](http://example.com).
///
/// Should also follow 308 redirects: <https://tinyurl.com/rnxcavf>.
/// If HEAD gives a 405 error, fall back to GET for <https://play.rust-lang.org/>.
/// If --forbid-http is passed, it should still be ok to link to <https://doc.rust-lang.org/>.
pub fn foo() {}

/// Bar function
pub fn bar() {}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
