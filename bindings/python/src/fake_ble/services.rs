#![allow(clippy::new_without_default)]
use std::borrow::Cow;

use pyo3::prelude::*;

/// A BLE data service for broadcasting a battery's remaining charge (as a percentage).
///
/// Conforms to Battery Level format as defined in
/// [GATT Specifications Supplement](https://www.bluetooth.org/DocMan/handlers/DownloadDoc.ashx?doc_id=502132&vId=542989).
#[pyclass(module = "rf24_py")]
#[derive(Debug, Clone, Copy)]
pub struct BatteryService {
    inner: rf24ble::services::BatteryService,
    buf: [u8; 5],
}

#[pymethods]
impl BatteryService {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: rf24ble::services::BatteryService::default(),
            buf: [0; 5],
        }
    }

    /// The battery charge level (as integer percentage) data.
    #[getter]
    pub fn get_data(&self) -> u8 {
        self.inner.data()
    }

    #[setter]
    pub fn set_data(&mut self, value: u8) {
        self.inner.set_data(value);
    }

    /// Transform the service data into a BLE compliant buffer that is ready for broadcasting.
    #[getter]
    pub fn get_buffer(&mut self) -> Cow<[u8]> {
        self.buf.copy_from_slice(&self.inner.buffer());
        Cow::from(&self.buf)
    }
}

/// A BLE data service that broadcasts a temperature (in Celsius)
///
/// Conforms to the Health Thermometer Measurement format as defined in
/// [GATT Specifications Supplement](https://www.bluetooth.org/DocMan/handlers/DownloadDoc.ashx?doc_id=502132&vId=542989).
#[pyclass(module = "rf24_py")]
#[derive(Debug, Clone, Copy)]
pub struct TemperatureService {
    inner: rf24ble::services::TemperatureService,
    buf: [u8; 8],
}

#[pymethods]
impl TemperatureService {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: rf24ble::services::TemperatureService::default(),
            buf: [0; 8],
        }
    }

    /// The temperature measurement (in Celsius) data.
    #[getter]
    pub fn get_data(&self) -> f32 {
        self.inner.data()
    }

    #[setter]
    pub fn set_data(&mut self, value: f32) {
        self.inner.set_data(value);
    }

    /// Transform the service data into a BLE compliant buffer that is ready for broadcasting.
    #[getter]
    pub fn get_buffer(&mut self) -> Cow<[u8]> {
        self.buf.copy_from_slice(&self.inner.buffer());
        Cow::from(&self.buf)
    }
}

/// A BLE data service for broadcasting a URL.
///
/// Conforms to specifications defined by [Google's EddyStone][eddystone] data format.
///
/// [eddystone]: https://github.com/google/eddystone
#[pyclass(module = "rf24_py")]
#[derive(Debug, Clone, Copy)]
pub struct UrlService {
    inner: rf24ble::services::UrlService,
}

#[pymethods]
impl UrlService {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: rf24ble::services::UrlService::default(),
        }
    }

    #[getter]
    pub fn get_data(&self) -> String {
        self.inner.data()
    }

    #[setter]
    pub fn set_data(&mut self, value: String) {
        self.inner.set_data(&value);
    }

    /// Transform the service data into a BLE compliant buffer that is ready for broadcasting.
    #[getter]
    pub fn get_buffer(&self) -> Cow<[u8]> {
        Cow::from(self.inner.buffer())
    }
}

/// A structure to represent received BLE data.
#[pyclass(frozen, get_all)]
pub struct BlePayload {
    pub mac_address: [u8; 6],
    pub short_name: Option<String>,
    pub tx_power: Option<i8>,
    pub battery_charge: Option<BatteryService>,
    pub url: Option<UrlService>,
    pub temperature: Option<TemperatureService>,
}

impl BlePayload {
    /// A factory method to create an instance of a
    /// [`BlePayload`][rf24_py.BlePayload] from a buffer of bytes.
    ///
    /// The given `buf` shall be de-whitened and in Big Endian.
    #[cfg_attr(
        not(target_os = "linux"),
        allow(dead_code, reason = "fn is only used on Linux targets")
    )]
    pub(crate) fn from_bytes(buf: &[u8]) -> Option<Self> {
        if let Some(payload) = rf24ble::services::BlePayload::from_bytes(buf) {
            return Some(Self {
                mac_address: payload.mac_address,
                short_name: payload
                    .short_name
                    .map(|n| String::from_utf8_lossy(&n).to_string()),
                tx_power: payload.tx_power,
                battery_charge: payload.battery_charge.map(|bat| BatteryService {
                    inner: bat,
                    buf: [0; 5],
                }),
                url: payload.url.map(|u| UrlService { inner: u }),
                temperature: payload.temperature.map(|t| TemperatureService {
                    inner: t,
                    buf: [0; 8],
                }),
            });
        }
        None
    }
}
