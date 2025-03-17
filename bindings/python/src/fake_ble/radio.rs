#![cfg(target_os = "linux")]
use std::borrow::Cow;

use pyo3::prelude::*;

use crate::radio::{config::RadioConfig, interface::RF24, types::PaLevel};
use rf24ble::{
    data_manipulation::{crc24_ble, reverse_bits, whitener},
    BLE_CHANNEL,
};

use super::services::BlePayload;

/// A class to use the nRF24L01 as a Fake BLE beacon.
#[pyclass(module = "rf24_py")]
pub struct FakeBle {
    radio: Py<RF24>,
    buf: [u8; 32],
    name: [u8; 10],
    include_pa_level: bool,
}

#[pymethods]
impl FakeBle {
    /// Create an Fake BLE device using the given RF24 instance.
    #[new]
    pub fn new(radio: Py<RF24>) -> Self {
        let mut buf = [0u8; 32];
        buf[0] = 0x42; // GATT profile flags

        // buf[1] is for the total payload size (excluding CRC24 at the end)

        // TODO: randomize this default MAC address.
        buf[2..8].copy_from_slice(b"nRF24L");

        // flags for declaring device capabilities
        buf[8..11].copy_from_slice(&[2, 1, 5]);

        // buf[11..29] is available for user data.
        // buf[29..32] is for the CRC24 checksum

        Self {
            radio,
            name: [0u8; 10],
            include_pa_level: false,
            buf,
        }
    }

    /// Configure the radio to be used as a BLE device.
    ///
    /// Be sure to call [`RF24::begin()`][rf24_py.RF24.begin] before calling this function.
    pub fn begin(&mut self) -> PyResult<()> {
        Python::with_gil(|py| {
            let mut radio = self.radio.bind(py).borrow_mut();
            radio.with_config(&RadioConfig::from_inner(rf24ble::ble_config()))
        })
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
        if let Some(name) = name {
            let len = name.len().min(8);
            self.name[2..len + 2].copy_from_slice(&name.as_bytes()[0..len]);
            self.name[0] = len as u8 + 1;
            self.name[1] = 0x08;
        } else {
            self.name[0] = 0;
        }
    }

    #[getter]
    pub fn get_name(&self) -> Option<String> {
        let len = self.name[0];
        if len > 1 {
            let len = len as usize - 1;
            let result = String::from_utf8_lossy(&self.name[2..len + 2]);
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
        self.buf[2..8].copy_from_slice(&address);
    }

    #[getter]
    pub fn get_mac_address(&self) -> Cow<[u8]> {
        Cow::from(&self.buf[2..8])
    }

    /// Enable or disable the inclusion of the radio's PA level in advertisements.
    ///
    /// Enabling this feature occupies 3 bytes of the 18 available bytes in
    /// advertised payloads.
    #[setter]
    pub fn show_pa_level(&mut self, enable: i32) {
        self.include_pa_level = enable != 0;
    }

    #[getter("show_pa_level")]
    pub fn has_pa_level(&self) -> bool {
        self.include_pa_level
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
        let mut result = 18 - hypothetical.len() as i8;
        let name_len = self.name[0];
        if name_len > 1 {
            result -= name_len as i8 + 1;
        }
        if self.include_pa_level {
            result -= 3;
        }
        result
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
            for (index, ch) in BLE_CHANNEL.iter().enumerate() {
                if *ch == channel {
                    if index < (BLE_CHANNEL.len() - 1) {
                        return radio.set_channel(BLE_CHANNEL[index + 1]);
                    } else {
                        return radio.set_channel(BLE_CHANNEL[0]);
                    }
                }
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
        let mut payload_length = buf.len() + 9;
        let mut tx_queue = [0; 32];
        let mut offset = 11;
        tx_queue[0..offset].copy_from_slice(&self.buf[0..offset]);
        Python::with_gil(|py| {
            let mut radio = self.radio.bind(py).borrow_mut();
            if self.include_pa_level {
                let pa_level: i8 = match radio.get_pa_level()? {
                    PaLevel::Min => -18,
                    PaLevel::Low => -12,
                    PaLevel::High => -6,
                    PaLevel::Max => 0,
                };
                payload_length += 3;
                offset += 3;
                tx_queue[11..offset].copy_from_slice(&[2, 0x0A, pa_level as u8]);
            }

            if self.name[0] > 1 {
                let len = self.name[0] as usize + 1;
                payload_length += len;
                tx_queue[offset..offset + len].copy_from_slice(&self.name[0..len]);
                offset += len;
            }

            if payload_length > 28 {
                // TODO should return `Err()` instead
                return Ok(false);
            }

            tx_queue[1] = payload_length as u8;
            for byte in buf {
                tx_queue[offset] = *byte;
                offset += 1;
            }
            let crc = crc24_ble(&tx_queue[0..offset]);
            tx_queue[offset..offset + 3].copy_from_slice(&crc);
            offset += 3;
            // println!("buffer: {tx_queue:02X?}");

            let coefficient = (radio.get_channel()? + 37) | 0x40;
            whitener(&mut tx_queue[0..offset], coefficient);
            // println!("whiten: {tx_queue:02X?}");

            reverse_bits(&mut tx_queue[0..offset]);
            // println!("revers: {tx_queue:02X?}");

            // Disregarding any hardware error, `RF24::send()` should
            // always return `Ok(true)` because auto-ack is off.
            radio.send(&tx_queue, false as i32)
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
    /// then this function returns an undefined value.
    pub fn read(&mut self) -> PyResult<Option<BlePayload>> {
        Python::with_gil(|py| {
            let mut radio = self.radio.bind(py).borrow_mut();
            let mut buf = [0u8; 32];
            buf.copy_from_slice(&radio.read(Some(32))?);
            reverse_bits(&mut buf);
            let coefficient = (radio.get_channel()? + 37) | 0x40;
            whitener(&mut buf, coefficient);
            Ok(BlePayload::from_bytes(&buf))
        })
    }
}
