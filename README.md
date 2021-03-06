# cargo-deadlinks • [![Crates.io](http://meritbadge.herokuapp.com/cargo-deadlinks)](https://crates.io/crates/cargo-deadlinks) ![License](https://img.shields.io/crates/l/cargo-deadlinks.svg)

Check your `cargo doc` documentation for broken links!

Useful if you just refactored the structure of your crate or want to ensure that
your documentation is readable offline.

This crate pairs well with [intra-doc links] and [`cargo-intraconv`], which
make it easier to write links without having to know the exact HTML page
rustdoc will generate.

`deadlinks` can also be used on projects that aren't Rust crates.

## Installation

Install cargo-deadlinks via:
```bash
cargo install cargo-deadlinks
```

Alternatively, install pre-built binaries from the [releases page].

[intra-doc links]: https://doc.rust-lang.org/rustdoc/linking-to-items-by-name.html
[`cargo-intraconv`]: https://github.com/poliorcetics/cargo-intraconv
[releases page]: https://github.com/deadlinks/cargo-deadlinks/releases

## Usage

From your packages directory run:
```bash
# this will automatically run `cargo doc` for you
# any broken links will show up in the output
cargo deadlinks
# if you also want to check http and https links
cargo deadlinks --check-http
```
By default `cargo deadlinks` will check only the offline (`file://`) links of your package.

If you want to check the documentation in another directory e.g. to check all
your dependencies, you can provide the `--dir` argument:
```bash
cargo deadlinks --dir target/doc
```

To check a project that isn't a crate, use `deadlinks` instead:
```bash
# this requires that you already have a static site in build/html
deadlinks build/html
```

For information about other arguments run `cargo deadlinks --help`.

## Minimum Supported Rust Version (MSRV)

The current MSRV is **1.45.0**. This may be increased in minor versions, but
will never increase in a patch version.

## Contributing

We are happy about any contributions!

To get started you can take a look at our [Github issues](https://github.com/deadlinks/cargo-deadlinks/issues).

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as below, without any additional terms or
conditions.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
