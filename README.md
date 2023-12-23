Iron Oxide Unsafe (IROX-UNSAFE) Libraries
=============================
A collection of (hopefully) useful unsafe crates written in Rust.

[![MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/spmadden/irox-unsafe/blob/master/LICENSE)
[![Apache](https://img.shields.io/badge/license-Apache-blue.svg)](https://github.com/spmadden/irox-unsafe/blob/master/LICENSE-APACHE)
![Maintenance](https://img.shields.io/maintenance/yes/2024)
![GitHub Workflow Status (with event)](https://img.shields.io/github/actions/workflow/status/spmadden/irox-unsafe/rust.yml)
[![Crates.io](https://img.shields.io/crates/v/irox-unsafe)](https://crates.io/crates/irox-unsafe/)
[![docs.rs](https://img.shields.io/docsrs/irox-unsafe/latest)](https://docs.rs/irox-unsafe/latest/irox-unsafe/)

[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg)](https://github.com/spmadden/irox-unsafe/blob/master/CODE_OF_CONDUCT.md)
[![Semver2.0](https://img.shields.io/badge/semver-2.0-blue)](https://semver.org/spec/v2.0.0.html)
[![ConvCommits](https://img.shields.io/badge/conventional--commits-1.0-pink)](https://www.conventionalcommits.org/en/v1.0.0/)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-permitted%20%3A%28-red.svg)](https://github.com/rust-secure-code/safety-dance/)

Current Modules & Organization:
-----------------

* [`irox-unsafe`](https://github.com/spmadden/irox-unsafe/blob/master/irox-unsafe) - Aggregator module
* [`libraries`](https://github.com/spmadden/irox-unsafe/blob/master/libraries) - Rust 'library' crates, usually without
  binaries
    * [`safe-windows`] - Wrappers around the windows native unsafe functions to make them ergonomic

Version Status
------------------

| Crate               | Status                                                                                          |
|---------------------|-------------------------------------------------------------------------------------------------|
| `irox-safe-windows` | [![safe-windows-vsn-shield]][safe-windows-crate] [![safe-windows-doc-shield]][safe-windows-doc] |

[`safe-windows`]: https://github.com/spmadden/irox-unsafe/blob/master/libraries/safe-windows
[safe-windows-vsn-shield]: https://img.shields.io/crates/v/irox-safe-windows.svg
[safe-windows-doc-shield]: https://docs.rs/irox-safe-windows/badge.svg
[safe-windows-crate]: https://crates.io/crates/irox-safe-windows
[safe-windows-doc]: https://docs.rs/irox-safe-windows

