#![allow(clippy::new_without_default)]
use napi::bindgen_prelude::Buffer;

/// A BLE data service for broadcasting a battery's remaining charge (as a percentage).
///
/// Conforms to Battery Level format as defined in
/// [GATT Specifications Supplement](https://www.bluetooth.org/DocMan/handlers/DownloadDoc.ashx?doc_id=502132&vId=542989).
///
/// @group BLE Service Data Classes
#[napi]
#[derive(Debug, Clone, Copy)]
pub struct BatteryService {
    inner: rf24ble::services::BatteryService,
    buf: [u8; 5],
}

#[napi]
impl BatteryService {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: rf24ble::services::BatteryService::default(),
            buf: [0u8; 5],
        }
    }

    #[napi(getter)]
    pub fn data(&self) -> u8 {
        self.inner.data()
    }

    /// The battery charge level (as integer percentage) data.
    #[napi(setter, js_name = "data")]
    pub fn set_data(&mut self, value: u8) {
        self.inner.set_data(value);
    }

    /// Transform the service data into a BLE compliant buffer that is ready for broadcasting.
    #[napi(getter)]
    pub fn buffer(&mut self) -> Buffer {
        self.buf.copy_from_slice(&self.inner.buffer());
        Buffer::from(&self.buf[..])
    }
}

/// A BLE data service that broadcasts a temperature (in Celsius)
///
/// Conforms to the Health Thermometer Measurement format as defined in
/// [GATT Specifications Supplement](https://www.bluetooth.org/DocMan/handlers/DownloadDoc.ashx?doc_id=502132&vId=542989).
///
/// @group BLE Service Data Classes
#[napi]
#[derive(Debug, Clone, Copy)]
pub struct TemperatureService {
    inner: rf24ble::services::TemperatureService,
    buf: [u8; 8],
}

#[napi]
impl TemperatureService {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: rf24ble::services::TemperatureService::default(),
            buf: [0u8; 8],
        }
    }

    #[napi(getter)]
    pub fn data(&self) -> f32 {
        self.inner.data()
    }

    /// The temperature measurement (in Celsius) data.
    #[napi(setter, js_name = "data")]
    pub fn set_data(&mut self, value: f64) {
        self.inner.set_data(value as f32);
    }

    /// Transform the service data into a BLE compliant buffer that is ready for broadcasting.
    #[napi(getter)]
    pub fn buffer(&mut self) -> Buffer {
        self.buf.copy_from_slice(&self.inner.buffer());
        Buffer::from(&self.buf[..])
    }
}

/// A BLE data service for broadcasting a URL.
///
/// Conforms to specifications defined by [Google's EddyStone][eddystone] data format.
///
/// [eddystone]: https://github.com/google/eddystone
///
/// @group BLE Service Data Classes
#[napi]
#[derive(Debug, Clone, Copy)]
pub struct UrlService {
    inner: rf24ble::services::UrlService,
}

#[napi]
impl UrlService {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: rf24ble::services::UrlService::default(),
        }
    }

    #[napi(getter)]
    pub fn data(&self) -> String {
        self.inner.data()
    }

    #[napi(setter, js_name = "data")]
    pub fn set_data(&mut self, value: String) {
        self.inner.set_data(&value);
    }

    /// Transform the service data into a BLE compliant buffer that is ready for broadcasting.
    #[napi(getter)]
    pub fn buffer(&mut self) -> Buffer {
        Buffer::from(self.inner.buffer())
    }
}
