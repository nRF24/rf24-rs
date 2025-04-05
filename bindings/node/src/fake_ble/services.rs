#![allow(clippy::new_without_default)]
use napi::bindgen_prelude::Buffer;
use rf24ble::services::prelude::*;

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
}

#[napi]
impl BatteryService {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: rf24ble::services::BatteryService::default(),
        }
    }

    #[napi(getter)]
    pub fn data(&self) -> u8 {
        self.inner.data()
    }

    /// The battery charge level (as a percentage integer) data.
    #[napi(setter, js_name = "data")]
    pub fn set_data(&mut self, value: u8) {
        self.inner.set_data(value);
    }

    /// Transform the service data into a BLE compliant buffer that is ready for broadcasting.
    #[napi(getter)]
    pub fn buffer(&mut self) -> Buffer {
        Buffer::from(self.inner.buffer())
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
}

#[napi]
impl TemperatureService {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: rf24ble::services::TemperatureService::default(),
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
        Buffer::from(self.inner.buffer())
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
    pub fn pa_level(&self) -> i8 {
        self.inner.pa_level()
    }

    /// The predicted PA (Power Amplitude) level at 1 meter radius.
    #[napi(setter, js_name = "paLevel")]
    pub fn set_pa_level(&mut self, value: i8) {
        self.inner.set_pa_level(value);
    }

    #[napi(getter)]
    pub fn data(&self) -> String {
        self.inner.data()
    }

    /// The URL to be broadcasted.
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

/// A structure to represent received BLE data.
#[napi]
pub struct BlePayload {
    mac_address: Buffer,
    short_name: Option<String>,
    tx_power: Option<i8>,
    battery_charge: Option<BatteryService>,
    url: Option<UrlService>,
    temperature: Option<TemperatureService>,
}

impl BlePayload {
    #[cfg_attr(
        not(target_os = "linux"),
        allow(dead_code, reason = "fn is only used on Linux targets")
    )]
    pub(crate) fn from_bytes(buf: &mut [u8], channel: u8) -> Option<Self> {
        if let Some(payload) = rf24ble::services::BlePayload::from_bytes(buf, channel) {
            return Some(Self {
                mac_address: Buffer::from(payload.mac_address.to_vec()),
                short_name: payload
                    .short_name
                    .map(|n| String::from_utf8_lossy(&n).to_string()),
                tx_power: payload.tx_power,
                battery_charge: payload
                    .battery_charge
                    .map(|batt| BatteryService { inner: batt }),
                url: payload.url.map(|u| UrlService { inner: u }),
                temperature: payload.temperature.map(|t| TemperatureService { inner: t }),
            });
        }
        None
    }
}

#[napi]
impl BlePayload {
    /// The transmitting device's MAC address.
    #[napi(getter)]
    pub fn mac_address(&self) -> Buffer {
        self.mac_address.clone()
    }

    /// The transmitting device's PA (Power Amplitude) level.
    #[napi(getter)]
    pub fn tx_power(&self) -> Option<i8> {
        self.tx_power
    }

    /// The transmitting device's short name.
    #[napi(getter)]
    pub fn short_name(&self) -> Option<String> {
        self.short_name.clone()
    }

    /// The transmitting device's remaining battery charge level.
    #[napi(getter)]
    pub fn battery_charge(&self) -> Option<BatteryService> {
        self.battery_charge
    }

    /// The transmitting device's temperature measurement.
    #[napi(getter)]
    pub fn temperature(&self) -> Option<TemperatureService> {
        self.temperature
    }

    /// The transmitting device's advertised URL.
    #[napi(getter)]
    pub fn url(&self) -> Option<UrlService> {
        self.url
    }
}
