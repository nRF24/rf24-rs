<!-- markdownlint-disable MD041 -->
[![Python builds][python-ci-badge]][python-ci] [![Node.js Builds][node-js-ci-badge]][node-js-ci] [![Tests][test-ci-badge]][test-ci] [![codecov][codecov-badge]][codecov-project] [![Docs][docs-ci-badge]][docs-ci] [![Examples][examples-ci-badge]][examples-ci]

# rf24-rs

This is a pure-rust driver for the nRF24L01 wireless transceivers.

> [!WARNING]
> This project is a Work-In-Progress.
> This warning will be removed when this project is ready for deployment.

## Supported platforms

This project aims to support the [embedded rust][embedded-rs] ecosystem.
This includes but is not limited to Linux on RPi. Other points of interest:

- [crates.io for embedded-hal crates][crates-hal]
- the [awesome embedded rust][awesome-hal] list
- the [embedded-hal][eh] framework

## Goals

Here is the intended roadmap:

- [x] implement driver for the nRF24L01 (OTA compatible with other RF24 library)

    This should be HAL-agnostic in terms of MCU. It would also be nice to
    reimplement the same API (using [rust's `trait` feature][rust-traits])
    for use on nRF5x radios.

- [ ] implement network layers (OTA compatible with RF24Network and RF24Mesh libraries)
- [ ] implement ESB support for nRF5x MCUs. This might be guarded under [cargo features][cargo-feat].

## Why?

Mostly because I :heart: rust. There are [other driver libraries for the nRF24L01 in pure rust][crates-rf24],
but they all seem unmaintained or designed to be application-specific. There's even
a [crate to use the nRF5x chips' ESB support][crate-esb], but this too seems lacking
maintainers' attention.

[python-ci-badge]: https://github.com/nRF24/rf24-rs/actions/workflows/python-packaging.yml/badge.svg
[python-ci]: https://github.com/nRF24/rf24-rs/actions/workflows/python-packaging.yml
[node-js-ci-badge]: https://github.com/nRF24/rf24-rs/actions/workflows/node-packaging.yml/badge.svg
[node-js-ci]: https://github.com/nRF24/rf24-rs/actions/workflows/node-packaging.yml
[docs-ci-badge]: https://github.com/nRF24/rf24-rs/actions/workflows/docs.yml/badge.svg
[docs-ci]: https://github.com/nRF24/rf24-rs/actions/workflows/docs.yml
[examples-ci-badge]: https://github.com/nRF24/rf24-rs/actions/workflows/examples.yml/badge.svg
[examples-ci]: https://github.com/nRF24/rf24-rs/actions/workflows/examples.yml
[test-ci-badge]: https://github.com/nRF24/rf24-rs/actions/workflows/tests.yml/badge.svg
[test-ci]: https://github.com/nRF24/rf24-rs/actions/workflows/tests.yml
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
