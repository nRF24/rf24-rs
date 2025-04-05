pub mod radio;
pub mod services;

use crate::radio::config::RadioConfig;
use pyo3::prelude::*;

/// Returns a [`RadioConfig`][rf24_py.RadioConfig] object tailored for
/// OTA compatibility with BLE specifications.
///
/// See also:
///     This configuration complies with inherent
///     [Limitations](https://docs.rs/rf24ble-rs/latest/rf24ble/index.html#limitations).
#[pyfunction]
pub fn ble_config() -> RadioConfig {
    RadioConfig::from_inner(rf24ble::ble_config())
}
