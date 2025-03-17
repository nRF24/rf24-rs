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
