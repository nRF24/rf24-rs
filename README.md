<!-- markdownlint-disable MD041 MD033 -->
[![Python][python-ci-badge]][python-ci] [![Node.js][node-js-ci-badge]][node-js-ci] [![Rust][rust-ci-badge]][rust-ci] [![codecov][codecov-badge]][codecov-project] [![Docs][docs-badge]][docs]

# rf24-rs

This is a pure-rust driver for the nRF24L01 wireless transceivers.

## Supported platforms

This project aims to support the [embedded rust][embedded-rs] ecosystem.
This includes but is not limited to Linux on RPi. Other points of interest:

- [crates.io for embedded-hal crates][crates-hal]
- the [awesome embedded rust][awesome-hal] list
- the [embedded-hal][eh] framework

## Deployed Packages

This project deploys several packages: Rust crates and bindings to Node.js and Python.

All packages are developed under the MIT license.

### Rust crates

[rf24-rs-badge]: https://img.shields.io/crates/v/rf24-rs
[rf24-rs-link]: https://crates.io/crates/rf24-rs
[rf24-rs-msrv]: https://img.shields.io/crates/msrv/rf24-rs
[rf24-rs-docs-badge]: https://img.shields.io/docsrs/rf24-rs
[rf24-rs-docs-link]: https://docs.rs/rf24-rs
[rf24ble-rs-badge]: https://img.shields.io/crates/v/rf24ble-rs
[rf24ble-rs-link]: https://crates.io/crates/rf24ble-rs
[rf24ble-rs-msrv]: https://img.shields.io/crates/msrv/rf24ble-rs
[rf24ble-rs-docs-badge]: https://img.shields.io/docsrs/rf24ble-rs
[rf24ble-rs-docs-link]: https://docs.rs/rf24ble-rs

| name | version | API docs | Minimum Supported<br>Rust Version |
|:-----|:-------:|:--------:|:---------------------------------:|
| `rf24-rs` | [![Crates.io Version][rf24-rs-badge]][rf24-rs-link] | [![docs.rs][rf24-rs-docs-badge]][rf24-rs-docs-link] | ![Crates.io MSRV][rf24-rs-msrv] |
| `rf24ble-rs` | [![Crates.io Version][rf24ble-rs-badge]][rf24ble-rs-link] | [![docs.rs][rf24ble-rs-docs-badge]][rf24ble-rs-docs-link] | ![Crates.io MSRV][rf24ble-rs-msrv] |

### Bindings

The bindings provided expose all the above Rust crates in one package (per language).
We do this to avoid interdependency problems in the language's FFI (Foreign Function Interface).

The binding packages provided will only function on Linux machines.
However, installing the packages in a non-Linux environment will still provide the typing information used on Linux.

[rtd-badge]: https://img.shields.io/readthedocs/rf24-rs

#### Node.js

[npm-badge]: https://img.shields.io/npm/v/%40rf24%2Frf24
[npm-link]: https://www.npmjs.com/package/@rf24/rf24
[node-version]: https://img.shields.io/node/v/%40rf24%2Frf24?color=blue

| name | version | API docs | Minimum Supported<br>Node Version |
|:-----|:-------:|:--------:|:---------------------------------:|
| `@rf24/rf24` | [![NPM Version][npm-badge]][npm-link] | [![Node.js API][rtd-badge]][node-api] | ![Node Current][node-version] |

[napi-rs-deep-dive]: https://napi.rs/docs/deep-dive/release#3-the-native-addon-for-different-platforms-is-distributed-through-different-npm-packages

> [!NOTE]
> The Node.js binding is actually provided in several packages.
> The package listed above will list the pre-compiled binary packages as optional dependencies.
> See the [napi-rs docs][napi-rs-deep-dive] for more detail and rationale.

#### Python

Distributions support CPython and PyPy.

[pypi-link]: https://pypi.org/project/rf24-py/
[pypi-badge]: https://img.shields.io/pypi/v/rf24-py
[piwheels-badge]: https://img.shields.io/piwheels/v/rf24-py
[piwheels-link]: https://www.piwheels.org/project/rf24-py/
[py-min-ver]: https://img.shields.io/badge/python->=3.8-blue

| name | version | API docs | Minimum Supported<br>Python Version |
|:-----|:-------:|:--------:|:-----------------------------------:|
| `rf24-py` | [![PyPI - Version][pypi-badge]][pypi-link]<br>[![PiWheels Version][piwheels-badge]][piwheels-link] | [![Python API][rtd-badge]][python-api] | ![Minimum Python Version: >=3.8][py-min-ver] |

## Goals

Here is the intended roadmap:

- [x] implement driver for the nRF24L01 (OTA compatible with RF24 library)

    This should be HAL-agnostic in terms of MCU. It would also be nice to
    reimplement the same API (using [rust's `trait` feature][rust-traits])
    for use on nRF5x radios.

- [x] implement a fake BLE API for the nRF24L01
- [ ] implement network layers (OTA compatible with RF24Network and RF24Mesh libraries)
- [ ] implement ESB support for nRF5x MCUs. This might be guarded under [cargo features][cargo-feat].

Code coverage is only measured against the [Rust crates](#rust-crates).
The bindings' code is not run through unit tests because they are just a thin wrapper around the [Rust crates](#rust-crates).

## Why?

Mostly because I :heart: rust. There are [other driver libraries for the nRF24L01 in pure rust][crates-rf24],
but they all seem unmaintained or designed to be application-specific. There's even
a [crate to use the nRF5x chips' ESB support][crate-esb], but this too seems lacking
maintainers' attention.

[python-ci-badge]: https://github.com/nRF24/rf24-rs/actions/workflows/python.yml/badge.svg
[python-ci]: https://github.com/nRF24/rf24-rs/actions/workflows/python.yml
[node-js-ci-badge]: https://github.com/nRF24/rf24-rs/actions/workflows/node.yml/badge.svg
[node-js-ci]: https://github.com/nRF24/rf24-rs/actions/workflows/node.yml
[docs-badge]: https://github.com/nRF24/rf24-rs/actions/workflows/docs.yml/badge.svg
[docs]: https://rf24-rs.readthedocs.io/en/latest
[rust-ci-badge]: https://github.com/nRF24/rf24-rs/actions/workflows/rust.yml/badge.svg
[rust-ci]: https://github.com/nRF24/rf24-rs/actions/workflows/rust.yml
[codecov-badge]: https://codecov.io/gh/nRF24/rf24-rs/graph/badge.svg?token=BMQ97Y5RVP
[codecov-project]: https://codecov.io/gh/nRF24/rf24-rs

[embedded-rs]: https://docs.rust-embedded.org/book/
[crates-hal]: https://crates.io/search?q=embedded-hal
[awesome-hal]: https://github.com/rust-embedded/awesome-embedded-rust
[eh]: https://github.com/rust-embedded/embedded-hal
[cargo-feat]: https://doc.rust-lang.org/cargo/reference/features.html
[rust-traits]: https://doc.rust-lang.org/book/ch19-03-advanced-traits.html#advanced-traits
[crates-rf24]: https://crates.io/search?q=rf24
[crate-esb]: https://crates.io/crates/esb

<!--absolute-links-->
[node-api]: https://rf24-rs.readthedocs.io/en/latest/node-api/
[python-api]: https://rf24-rs.readthedocs.io/en/latest/python-api/
