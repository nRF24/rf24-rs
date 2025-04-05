#![cfg(target_os = "linux")]
use std::borrow::Cow;

use pyo3::prelude::*;

use crate::radio::interface::RF24;
use rf24ble::BleChannels;

use super::services::BlePayload;

/// A class to use the nRF24L01 as a Fake BLE beacon.
///
/// See also:
///     This implementation is subject to
///     [Limitations](https://docs.rs/rf24ble-rs/latest/rf24ble/index.html#limitations).
///
///     Use [`ble_config()`][rf24_py.ble_config] to properly configure the radio for
///     BLE compatibility.
///
///     ```py
///     from rf24_py import RF24, FakeBle, ble_config
///
///     radio = RF24(22, 0)
///     radio.begin()
///     radio.with_config(ble_config())
///     ble = FakeBle(radio)
///
///     radio.print_details()
///     ```
#[pyclass(module = "rf24_py")]
pub struct FakeBle {
    radio: Py<RF24>,
    inner: rf24ble::FakeBle,
}

#[pymethods]
impl FakeBle {
    /// Create an Fake BLE device using the given RF24 instance.
    #[new]
    pub fn new(radio: Py<RF24>) -> Self {
        Self {
            radio,
            inner: rf24ble::FakeBle::default(),
        }
    }

    /// Set or get the BLE device's name for included in advertisements.
    ///
    /// Setting a BLE device name will occupy more bytes from the
    /// 18 available bytes in advertisements. The exact number of bytes occupied
    /// is the length of the given `name` string plus 2.
    ///
    /// The maximum supported name length is 10 bytes.
    /// So, up to 12 bytes (10 + 2) will be used in the advertising payload.
    #[setter]
    pub fn set_name(&mut self, name: Option<String>) {
        match name {
            Some(n) => self.inner.set_name(&n),
            None => self.inner.set_name(""),
        }
    }

    #[getter]
    pub fn get_name(&self) -> Option<String> {
        let mut tmp = [0u8; 12];
        let len = self.inner.get_name(&mut tmp) as usize;
        if len > 0 {
            let result = String::from_utf8_lossy(&tmp[2..len + 2]);
            return Some(result.to_string());
        }
        None
    }

    /// Set or get the BLE device's MAC address.
    ///
    /// A MAC address is required by BLE specifications.
    /// Use this attribute to uniquely identify the BLE device.
    #[setter]
    pub fn set_mac_address(&mut self, address: [u8; 6]) {
        self.inner.mac_address.copy_from_slice(&address);
    }

    #[getter]
    pub fn get_mac_address(&self) -> Cow<[u8]> {
        Cow::from(&self.inner.mac_address)
    }

    /// Enable or disable the inclusion of the radio's PA level in advertisements.
    ///
    /// Enabling this feature occupies 3 bytes of the 18 available bytes in
    /// advertised payloads.
    #[setter]
    pub fn show_pa_level(&mut self, enable: i32) {
        self.inner.show_pa_level = enable != 0;
    }

    #[getter("show_pa_level")]
    pub fn has_pa_level(&self) -> bool {
        self.inner.show_pa_level
    }

    /// How many bytes are available in an advertisement payload?
    ///
    /// The `hypothetical` parameter shall be the same value passed to [`FakeBle.send()`][rf24_py.FakeBle.send].
    ///
    /// In addition to the given `hypothetical` payload length, this function also
    /// accounts for the current state of [`FakeBle.name`][rf24_py.FakeBle.name] and
    /// [`FakeBle.show_pa_level`][rf24_py.FakeBle.show_pa_level].
    ///
    /// If the returned value is less than `0`, then the `hypothetical` payload will not
    /// be broadcasted.
    pub fn len_available(&self, hypothetical: &[u8]) -> i8 {
        self.inner.len_available(hypothetical)
    }

    /// Hop the radio's current channel to the next BLE compliant frequency.
    ///
    /// Use this function after [`FakeBle.send()`][rf24_py.FakeBle.send] to comply with BLE specifications.
    /// This is not required, but it is recommended to avoid bandwidth pollution.
    ///
    /// This function should not be called in RX mode. To ensure proper radio behavior,
    /// the caller must ensure that the radio is in TX mode.
    pub fn hop_channel(&mut self) -> PyResult<()> {
        Python::with_gil(|py| {
            let mut radio = self.radio.bind(py).borrow_mut();
            let channel = radio.get_channel()?;
            if let Some(channel) = BleChannels::increment(channel) {
                radio.set_channel(channel)?;
            }
            // if the current channel is not a BLE_CHANNEL, then do nothing
            Ok(())
        })
    }

    /// Send a BLE advertisement
    ///
    /// The `buf` parameter takes a buffer that has been already formatted for
    /// BLE specifications.
    ///
    /// See convenient API to
    /// - advertise a Battery's remaining change level: [`BatteryService`][rf24_py.BatteryService]
    /// - advertise a Temperature measurement: [`TemperatureService`][rf24_py.TemperatureService]
    /// - advertise a URL: [`UrlService`][rf24_py.UrlService]
    ///
    /// For a custom/proprietary BLE service, the given `buf` must adopt compliance with BLE specifications.
    /// For example, a buffer of `n` bytes shall be formed as follows:
    ///
    /// | index | value |
    /// |:------|:------|
    /// | `0` | `n - 1` |
    /// | `1` | `0xFF`  |
    /// | `2 ... n - 1` | custom data |
    pub fn send(&mut self, buf: &[u8]) -> PyResult<bool> {
        Python::with_gil(|py| {
            let mut radio = self.radio.bind(py).borrow_mut();
            if let Some(tx_queue) = self.inner.make_payload(
                buf,
                if self.inner.show_pa_level {
                    Some(radio.get_pa_level()?.into_inner())
                } else {
                    None
                },
                radio.get_channel()?,
            ) {
                // Disregarding any hardware error, `RF24::send()` should
                // always return `Ok(true)` because auto-ack is off.
                radio.send(&tx_queue, false as i32)
            } else {
                Ok(false)
            }
        })
    }

    /// Read the first available payload from the radio's RX FIFO
    /// and decode it into a [`BlePayload`][rf24_py.BlePayload].
    ///
    /// > [!WARNING]
    /// > The payload must be decoded while the radio is on
    /// > the same channel that it received the data.
    /// > Otherwise, the decoding process will fail.
    ///
    /// Use [`RF24.available`][rf24_py.RF24.available] to
    /// check if there is data in the radio's RX FIFO.
    ///
    /// If the payload was somehow malformed or incomplete,
    /// then this function returns an `None` value.
    pub fn read(&mut self) -> PyResult<Option<BlePayload>> {
        Python::with_gil(|py| {
            let mut radio = self.radio.bind(py).borrow_mut();
            let mut buf = [0u8; 32];
            buf.copy_from_slice(&radio.read(Some(32))?);
            let channel = radio.get_channel()?;
            Ok(BlePayload::from_bytes(&mut buf, channel))
        })
    }
}
