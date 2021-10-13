<a name="unreleased"></a>

## NEXT (UNRELEASED)

<a name="0.8.1"></a>
## 0.8.1 (2021-10-12)

#### Changed

* Updated many dependencies. Deadlinks no longer has any dependencies that fail `cargo audit`. [PR#153]

#### Fixed

* Tests now pass even if the project directory is not named "cargo-deadlinks". [PR#149]

[PR#153]: https://github.com/deadlinks/cargo-deadlinks/pull/153
[PR#149]: https://github.com/deadlinks/cargo-deadlinks/pull/149

<a name="0.8.0"></a>
## 0.8.0 (2020-01-17)

#### Added

* `cargo deadlinks` and `deadlinks` now take a `--forbid-http` argument which gives an error if any HTTP links are present.
  This can be useful for ensuring all documentation is viewable offline. [PR#138]

#### Changed

* `CheckError` now has an `HttpForbidden` variant. [PR#138]
* The `check_http` field of `CheckContext` is now an enum instead of a boolean [PR#138]
* ureq has been upgraded to 2.0. This affects the public `CheckError` API, but should otherwise have no user-facing impact. [PR#134]

[PR#134]: https://github.com/deadlinks/cargo-deadlinks/pull/134
[PR#138]: https://github.com/deadlinks/cargo-deadlinks/pull/138

<a name="0.7.2"></a>
## 0.7.2 (2020-01-09)

#### Fixed

* When a website gives 405 Method Not Supported for HEAD requests, fall back to GET. In particular,
  this no longer marks all links to play.rust-lang.org as broken. [PR#136]
* URL-encoded fragments, like `#%E2%80%A0`, are now decoded. [PR#141]

[PR#136]: https://github.com/deadlinks/cargo-deadlinks/pull/136
[PR#141]: https://github.com/deadlinks/cargo-deadlinks/pull/141

#### Changed

* Give a warning when HTTP links are present but `--check-http` wasn't passed. Previously this was only a DEBUG message.
  Note that this still requires opting-in to warnings with `RUST_LOG=warn`. [PR#137]

[PR#137]: https://github.com/deadlinks/cargo-deadlinks/pull/137

<a name="0.7.1"></a>
## 0.7.1 (2020-12-18)

#### Fixed

* HTML `<meta>` redirects are now followed.

<a name="0.7.0"></a>
## 0.7.0 (2020-12-06)

#### Added

* `cargo deadlinks` now takes a `--cargo-dir` argument, allowing you to check projects other than the current directory.
  This is most useful for developing deadlinks itself, but might be helpful for other use cases. [PR#119]
* `cargo deadlinks` can now check for broken [intra-doc links] based on heuristics.
  This feature is still experimental and may have bugs; in particular, only
  links with backticks (i.e. generated as `<code>`) are currently found.
  You can opt in with `--check-intra-doc-links`.
  `deadlinks` has not been changed. [PR#126] [PR#128]

[intra-doc links]: https://doc.rust-lang.org/rustdoc/linking-to-items-by-name.html
[PR#128]: https://github.com/deadlinks/cargo-deadlinks/pull/128
[PR#126]: https://github.com/deadlinks/cargo-deadlinks/pull/126
[PR#119]: https://github.com/deadlinks/cargo-deadlinks/pull/119

#### Changed

* `walk_dir` now takes `&CheckContext`, not `CheckContext`. [PR#118]
* `CheckError` now has a new `IntraDocLink` variant. [PR#126]
* `parse_html_file` has been removed. Instead, use `parse_a_hrefs` or `broken_intra_doc_links` (or both). [PR#126]
* `Link::File` now stores a `PathBuf`, not a `String`. [PR#127]
* `print_shortened` has been removed; using `Display` directly is recommended instead. [PR#127]
  In particular, it's no longer possible to shorten files without going
  through `unavailable_urls`. If you were using this API, please let me know
  so I can help design an API that fits your use case; the previous one was a
  maintenance burden.

#### Fixed

* Fragment errors are now shortened to use the directory being checked as the base, the same as normal 'file not found errors'. [PR#127]
* 307 and 308 redirects are now followed. Previously, they would erroneously be reported as an error. [PR#129]

[PR#118]: https://github.com/deadlinks/cargo-deadlinks/pull/118
[PR#127]: https://github.com/deadlinks/cargo-deadlinks/pull/127
[PR#129]: https://github.com/deadlinks/cargo-deadlinks/pull/129

<a name="0.6.2"></a>
## 0.6.2 (2020-11-27)

#### Added

* `cargo-deadlinks` now allows passing arguments to `cargo doc`, using `cargo deadlinks -- <CARGO_ARGS>`. [PR#116]
* `deadlinks` now allows specifying multiple directories to check. [PR#116]

#### Fixed

* Warnings from cargo are no longer silenced when documenting. [PR#114]
* `cargo deadlinks` no longer ignores all directories on Windows. [PR#121]

#### Changes

* Argument parsing now uses `pico-args`, not `docopt`. [PR#116]
* Running `cargo-deadlinks` (not `cargo deadlinks`) now gives a better error message. [PR#116]
* Both binaries now print the name of the binary when passed `--version`. [PR#116]

[PR#114]: https://github.com/deadlinks/cargo-deadlinks/pull/114
[PR#116]: https://github.com/deadlinks/cargo-deadlinks/pull/116
[PR#121]: https://github.com/deadlinks/cargo-deadlinks/pull/121

<a name="0.6.1"></a>
## 0.6.1 (2020-11-23)

#### Added

* `--ignore-fragments` CLI parameter to disable URL fagment checking. [PR#108]

#### Fixed

* Empty fragments are no longer treated as broken links. This allows using `deadlinks` with unsafe functions, which have a generated fragment URL from rustdoc. [PR#109]

[PR#108]: https://github.com/deadlinks/cargo-deadlinks/pull/108
[PR#109]: https://github.com/deadlinks/cargo-deadlinks/pull/109

<a name="0.6.0"></a>
## 0.6.0 (2020-11-19)

#### Added

* `RUST_LOG` is now read, and controls logging. [PR#100]
* There is now a separate `deadlinks` binary which doesn't depend on cargo in any way. [PR#87]
* `CheckContext` now implements `Default`. [PR#101]
* `cargo deadlinks` will now run `cargo doc` automatically. You can opt-out of this behavior with `--no-build`. [PR#102]

#### Changes

* Errors are now printed to stdout, not stderr. [PR#100]
* Logging now follows the standard `env_logger` format. [PR#100]
* `--debug` and `--verbose` are deprecated in favor of `RUST_LOG`. [PR#100]
* Published Linux binaries are now built against musl libc, not glibc. This allows running deadlinks in an alpine docker container. [PR#103]

#### Fixes

* `doc = false` is now taken into account when running `cargo deadlinks`. It will still be ignored when running with `--no-build`. [PR#102]
* `CARGO_BUILD_TARGET` and other cargo configuration is now taken into account when running `cargo deadlinks`. It will still be ignored when running with `--no-build`. [PR#102]

[PR#87]: https://github.com/deadlinks/cargo-deadlinks/pull/87
[PR#100]: https://github.com/deadlinks/cargo-deadlinks/pull/100
[PR#101]: https://github.com/deadlinks/cargo-deadlinks/pull/101
[PR#102]: https://github.com/deadlinks/cargo-deadlinks/pull/102
[PR#103]: https://github.com/deadlinks/cargo-deadlinks/pull/103

<a name="0.5.0"></a>
## 0.5.0 (2020-11-13)

#### Added

* If a URL points to a directory, check if index.html exists in that directory. [PR#90]
* Treat absolute paths as absolute with respect to the `base_url`, not with respect to the file system. [PR#91]
* Check link fragments, with special handling for Rustdoc ranged fragments to highlight source code lines [PR#94]

[PR#90]: https://github.com/deadlinks/cargo-deadlinks/pull/90
[PR#91]: https://github.com/deadlinks/cargo-deadlinks/pull/91
[PR#94]: https://github.com/deadlinks/cargo-deadlinks/pull/94

#### Fixes

* No longer try to document examples that are dynamic libraries

  This was a regression introduced by [PR#68]. That looked at all targets to
  see which should be documented, but the logic for determining whether a target
  had docs was incorrect - it counted tests and examples if they were marked as a
  library. deadlinks will now ignore tests and examples even if they are not
  binaries.

* No longer download dependencies from crates.io when calculating targets

  Previously, `cargo metadata` would download all dependencies even though they weren't used.

#### Changes

* Switch from `reqwest` to `ureq` for HTTP-checking, cutting down the number of dependencies by almost a third. [PR#95]
* Switch from `html5ever` to `lol_html`, making the code much easier to modify. [PR#86]

[PR#86]: https://github.com/deadlinks/cargo-deadlinks/pull/86
[PR#95]: https://github.com/deadlinks/cargo-deadlinks/pull/95

<a name="0.4.2"></a>
## 0.4.2 (2020-10-12)

#### Added

* Add support for cargo workspaces. Check all crates and targets in the workspaces, excluding tests, benches, and examples. [PR#68], [PR#73]
* Add automatic binary releases. [PR#64] You can find the releases at [/releases] on the GitHub page.

[PR#64]: https://github.com/deadlinks/cargo-deadlinks/pull/64
[/releases]: https://github.com/deadlinks/cargo-deadlinks/releases

#### Fixes

* Take `CARGO_TARGET_DIR` into account when looking for the target directory. [PR#66]
* Give a better error message if Cargo.toml is not present. [PR#67]
* Follow target renames. [PR#68]
* Always output all errors instead of stopping after the first error. [PR#74]

Previously, deadlinks would stop after the first error, but leave other threads running in parallel. This would lead to non-deterministic and incomplete output if there were broken links in many different files.
Deadlinks will now output all errors before exiting.

[PR#66]: https://github.com/deadlinks/cargo-deadlinks/pull/66
[PR#67]: https://github.com/deadlinks/cargo-deadlinks/pull/67
[PR#73]: https://github.com/deadlinks/cargo-deadlinks/pull/73
[PR#74]: https://github.com/deadlinks/cargo-deadlinks/pull/74

#### Changes

* Update dependencies. [PR#51], [PR#76], [22fa61df] Thanks to [@Marwes][user_marwes]!
* Use HEAD instead of GET for HTTP requests. This should decrease the time for HTTP checks slightly. [PR#63] Thanks to [@zummenix]!
* Check all targets, not just targets with the same name as the package. In particular, this now checks both binaries and libraries. [PR#68]
* Shorten path names when `--debug` is not passed. [PR#20]

[@zummenix]: https://github.com/zummenix
[PR#20]: https://github.com/deadlinks/cargo-deadlinks/pull/20
[PR#51]: https://github.com/deadlinks/cargo-deadlinks/pull/51
[PR#63]: https://github.com/deadlinks/cargo-deadlinks/pull/63
[PR#68]: https://github.com/deadlinks/cargo-deadlinks/pull/68
[PR#76]: https://github.com/deadlinks/cargo-deadlinks/pull/76
[22fa61df]: https://github.com/deadlinks/cargo-deadlinks/commit/22fa61df44820d7f05415e026fa8396ee0e82954

<a name="0.4.1"></a>
## 0.4.1 (2019-03-26)

#### Features

* Provide a crate in addition to the binary. [PR#48][pr_48] Thanks to [@Marwes][user_marwes]!

<a name="0.4.0"></a>
## 0.4.0 (2019-03-17)

#### Features

* Add checking of HTTP links via `reqwest` (Thanks to [@gsquire][user_gsquire]!)
  * Can be used with `cargo deadlinks --check-http`
* Improved error message on missing docs directory. [PR#33][pr_33]


<a name="0.3.0"></a>
## 0.3.0 (2017-11-16)

???

<a name="0.2.1"></a>
## 0.2.1 (2017-10-12)

???

<a name="0.2.0"></a>
## 0.2.0 (2017-10-06)

???

<a name="0.1.0"></a>
## 0.1.0 (2016-03-25)

???

<!-- Contributor links -->
[user_gsquire]: https://github.com/gsquire
[user_marwes]: https://github.com/Marwes

[pr_33]: https://github.com/deadlinks/cargo-deadlinks/pull/33
[pr_48]: https://github.com/deadlinks/cargo-deadlinks/pull/48
