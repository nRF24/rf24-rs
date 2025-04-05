use crate::radio::config::RadioConfig;

pub mod radio;
pub mod services;

/// Returns a {@link RadioConfig} object tailored for
/// OTA compatibility with BLE specifications.
///
/// !!! note "See also"
///     This configuration complies with inherent
///     [Limitations](https://docs.rs/rf24ble-rs/latest/rf24ble/index.html#limitations).
#[napi]
#[allow(dead_code)]
pub fn ble_config() -> RadioConfig {
    RadioConfig::from_inner(rf24ble::ble_config())
}
