# `rf24ble-rs`

[rf24ble-rs-badge]: https://img.shields.io/crates/v/rf24ble-rs
[rf24ble-rs-link]: https://crates.io/crates/rf24ble-rs
[rf24ble-rs-msrv]: https://img.shields.io/crates/msrv/rf24ble-rs
[rf24ble-rs-docs-badge]: https://img.shields.io/docsrs/rf24ble-rs
[rf24ble-rs-docs-link]: https://docs.rs/rf24ble-rs
[changelog-badge]: https://img.shields.io/badge/keep_a_change_log-v1.1.0-ffec3d
[changelog-link]: https://rf24-rs.readthedocs.io/en/latest/rf24ble-rs-changelog/

[![Crates.io Version][rf24ble-rs-badge]][rf24ble-rs-link]
[![docs.rs][rf24ble-rs-docs-badge]][rf24ble-rs-docs-link]
![Crates.io MSRV][rf24ble-rs-msrv]
 [![CHANGELOG][changelog-badge]][changelog-link]

This crate uses the `rf24-rs` crate to make the nRF24L01 imitate a
Bluetooth-Low-Emissions (BLE) beacon. A BLE beacon can send data (referred to as
advertisements) to any BLE compatible device (ie smart devices with Bluetooth
4.0 or later) that is listening.

[fake-ble-research]: http://dmitry.gr/index.php?r=05.Projects&proj=11.%20Bluetooth%20LE%20fakery

Original research was done by [Dmitry Grinberg and his write-up (including C
source code) can be found here][fake-ble-research].
As this technique can prove invaluable in certain project designs, the code
here has been adapted to work with Rust.

## Example

See the [example located in the nRF24/rf24-rs repository][ble-example].

[ble-example]: https://github.com/nRF24/rf24-rs/blob/main/examples/rust/src/bin/fake_ble.rs
