#![cfg(target_os = "linux")]

use crate::radio::{interface::RF24, types::coerce_to_bool};
use napi::{
    bindgen_prelude::{Buffer, Reference, Result},
    JsNumber,
};
use rf24ble::BleChannels;

use super::services::BlePayload;

/// A class to use the nRF24L01 as a Fake BLE beacon.
///
/// !!! note "See also"
///     This implementation is subject to
///     [Limitations](https://docs.rs/rf24ble-rs/latest/rf24ble/index.html#limitations).
///
///     Use {@link bleConfig} to properly configure the radio for
///     BLE compatibility.
///
///     ```ts
///     import { bleConfig, FakeBle, RF24 } from "@rf24/rf24";
///
///     const radio = new RF24(22, 0);
///     radio.begin();
///     radio.withConfig(bleConfig());
///     const ble = new FakeBle(radio);
///
///     radio.printDetails();
///     ```
#[napi]
pub struct FakeBle {
    radio: Reference<RF24>,
    inner: rf24ble::FakeBle,
}

#[napi]
impl FakeBle {
    /// Create an Fake BLE device using the given RF24 instance.
    #[napi(constructor)]
    pub fn new(radio: Reference<RF24>) -> Self {
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
    #[napi(setter, js_name = "name")]
    pub fn set_name(&mut self, name: String) {
        self.inner.set_name(&name);
    }

    #[napi(getter, js_name = "name")]
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
    #[napi(setter, js_name = "macAddress")]
    pub fn set_mac_address(&mut self, address: &[u8]) {
        self.inner.mac_address.copy_from_slice(&address[0..6]);
    }

    #[napi(getter, js_name = "macAddress")]
    pub fn get_mac_address(&self) -> [u8; 6] {
        let mut result = [0u8; 6];
        result.copy_from_slice(&self.inner.mac_address);
        result
    }

    /// Enable or disable the inclusion of the radio's PA level in advertisements.
    ///
    /// Enabling this feature occupies 3 bytes of the 18 available bytes in
    /// advertised payloads.
    #[napi(setter)]
    pub fn show_pa_level(
        &mut self,
        #[napi(ts_arg_type = "boolean | number")] enable: JsNumber,
    ) -> Result<()> {
        let val = coerce_to_bool(Some(enable), false)?;
        self.inner.show_pa_level = val;
        Ok(())
    }

    #[napi(getter, js_name = "showPaLevel")]
    pub fn has_pa_level(&self) -> bool {
        self.inner.show_pa_level
    }

    /// How many bytes are available in an advertisement payload?
    ///
    /// The `hypothetical` parameter shall be the same value passed to {@link FakeBle.send}.
    ///
    /// In addition to the given `hypothetical` payload length, this function also
    /// accounts for the current state of {@link FakeBle.name} and
    /// {@link FakeBle.showPaLevel}.
    ///
    /// If the returned value is less than `0`, then the `hypothetical` payload will not
    /// be broadcasted.
    #[napi]
    pub fn len_available(&self, hypothetical: &[u8]) -> i8 {
        self.inner.len_available(hypothetical)
    }

    /// Hop the radio's current channel to the next BLE compliant frequency.
    ///
    /// Use this function after {@link FakeBle.send} to comply with BLE specifications.
    /// This is not required, but it is recommended to avoid bandwidth pollution.
    ///
    /// This function should not be called in RX mode. To ensure proper radio behavior,
    /// the caller must ensure that the radio is in TX mode.
    #[napi]
    pub fn hop_channel(&mut self) -> Result<()> {
        let channel = self.radio.get_channel()?;
        if let Some(channel) = BleChannels::increment(channel) {
            self.radio.set_channel(channel)?;
        }
        // if the current channel is not a BLE_CHANNEL, then do nothing
        Ok(())
    }

    /// Send a BLE advertisement
    ///
    /// The `buf` parameter takes a buffer that has been already formatted for
    /// BLE specifications.
    ///
    /// See our convenient API to
    /// - advertise a Battery's remaining change level: {@link BatteryService}
    /// - advertise a Temperature measurement: {@link TemperatureService}
    /// - advertise a URL: {@link UrlService}
    ///
    /// For a custom/proprietary BLE service, the given `buf` must adopt compliance with BLE specifications.
    /// For example, a buffer of `n` bytes shall be formed as follows:
    ///
    /// | index | value |
    /// |:------|:------|
    /// | `0` | `n - 1` |
    /// | `1` | `0xFF`  |
    /// | `2 ... n - 1` | custom data |
    #[napi]
    pub fn send(&mut self, buf: &[u8]) -> Result<bool> {
        if let Some(tx_queue) = self.inner.make_payload(
            buf,
            if self.inner.show_pa_level {
                Some(self.radio.get_pa_level()?.into_inner())
            } else {
                None
            },
            self.radio.get_channel()?,
        ) {
            // Disregarding any hardware error, `RF24::send()` should
            // always return `Ok(true)` because auto-ack is off.
            self.radio.send(Buffer::from(tx_queue.to_vec()), None)
        } else {
            Ok(false)
        }
    }

    /// Read the first available payload from the radio's RX FIFO
    /// and decode it into a {@link BlePayload}.
    ///
    /// > [!WARNING]
    /// > The payload must be decoded while the radio is on
    /// > the same channel that it received the data.
    /// > Otherwise, the decoding process will fail.
    ///
    /// Use {@link RF24.available} to check if there is data in the radio's RX FIFO.
    ///
    /// If the payload was somehow malformed or incomplete,
    /// then this function returns an undefined value.
    #[napi]
    pub fn read(&mut self) -> Result<Option<BlePayload>> {
        let mut buf = self.radio.read(Some(32))?;
        let channel = self.radio.get_channel()?;
        Ok(BlePayload::from_bytes(&mut buf, channel))
    }
}
