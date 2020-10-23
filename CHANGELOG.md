<a name="unreleased"></a>

## NEXT (UNRELEASED)

#### Fixes

* No longer try to document examples that are dynamic libraries

  This was a regression introduced by [PR#68]. That looked at all targets to
  see which should be documented, but the logic for determining whether a target
  had docs was incorrect - it counted tests and examples if they were marked as a
  library. deadlinks will now ignore tests and examples even if they are not
  binaries.

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
