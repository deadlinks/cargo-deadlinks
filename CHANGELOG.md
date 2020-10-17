<a name="unreleased"></a>
## NEXT (UNRELEASED)

#### Fixes

* Take `CARGO_TARGET_DIR` into account when looking for the target directory. [PR#66]
* Give a better error message if Cargo.toml is not present. [PR#67]
* Follow target renames. [PR#68]

[PR#66]: https://github.com/deadlinks/cargo-deadlinks/pull/66
[PR#67]: https://github.com/deadlinks/cargo-deadlinks/pull/67

#### Changes

* Update dependencies. [PR#51] Thanks to [@Marwes][user_marwes]!
* Use HEAD instead of GET for HTTP requests. This should decrease the time for HTTP checks slightly. [PR#63] Thanks to [@zummenix]!
* Check all targets, not just targets with the same name as the package. In particular, this now checks both binaries and libraries. [PR#68]

[@zummenix]: https://github.com/zummenix
[PR#51]: https://github.com/deadlinks/cargo-deadlinks/pull/51
[PR#63]: https://github.com/deadlinks/cargo-deadlinks/pull/63
[PR#68]: https://github.com/deadlinks/cargo-deadlinks/pull/68

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
