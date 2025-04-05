use crate::{
    data_manipulation::{crc24_ble, reverse_bits, whiten},
    services::BlePayload,
};
use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};
use rf24::{
    radio::{
        prelude::{EsbChannel, EsbPaLevel, EsbRadio},
        Nrf24Error, RadioConfig, RF24,
    },
    CrcLength, PaLevel,
};

/// The supported channels used amongst BLE devices.
pub const BLE_CHANNEL: [u8; 3] = [2, 26, 80];

/// A namespace of methods to manage the supported range of BLE channels.
pub struct BleChannels;

impl BleChannels {
    /// Get the index of [`BLE_CHANNEL`] for the given `channel`.
    ///
    /// Returns [`None`] if the given current `channel` is not in [`BLE_CHANNEL`].
    pub fn index_of(channel: u8) -> Option<usize> {
        for (index, ch) in BLE_CHANNEL.iter().enumerate() {
            if *ch == channel {
                return Some(index);
            }
        }
        None
    }

    /// Get the next BLE channel given the `current` radio channel.
    ///
    /// Returns [`None`] if the given `current` channel is not in [`BLE_CHANNEL`].
    pub fn increment(current: u8) -> Option<u8> {
        if let Some(index) = Self::index_of(current) {
            if index < (BLE_CHANNEL.len() - 1) {
                return Some(BLE_CHANNEL[index + 1]);
            } else {
                return Some(BLE_CHANNEL[0]);
            }
        }
        None
    }
}

/// The only address usable in BLE context.
const BLE_ADDRESS: [u8; 4] = [0x71, 0x91, 0x7d, 0x6b];

/// Returns a [`RadioConfig`] object tailored for OTA compatibility with
/// BLE specifications.
///
/// This configuration complies with inherent [Limitations](index.html#limitations).
pub fn ble_config() -> RadioConfig {
    RadioConfig::default()
        .with_channel(BLE_CHANNEL[0])
        .with_crc_length(CrcLength::Disabled)
        .with_auto_ack(0)
        .with_auto_retries(0, 0)
        .with_address_length(4)
        .with_rx_address(1, &BLE_ADDRESS)
        .with_tx_address(&BLE_ADDRESS)
}

/// A struct that implements BLE functionality.
///
/// This implementation is subject to [Limitations](index.html#limitations).
///
/// Use [`ble_config()`] to properly configure the radio for BLE compatibility.
///
/// ```ignore
/// use rf24::radio::{prelude::*, RF24};
/// use rf24ble::{ble_config, radio::FakeBle};
///
/// let mut radio = RF24::new(ce_pin, spi_device, delay_impl);
/// radio.init()?;
/// radio.withConfig(&ble_config())?;
/// let mut ble = FakeBle::new();
///
/// radio.print_details()?;
/// ```
pub struct FakeBle {
    pub(crate) name: [u8; 12],
    /// Enable or disable the inclusion of the radio's PA level in advertisements.
    ///
    /// Enabling this feature occupies 3 bytes of the 18 available bytes in
    /// advertised payloads.
    pub show_pa_level: bool,
    /// Set or get the BLE device's MAC address.
    ///
    /// A MAC address is required by BLE specifications.
    /// Use this attribute to uniquely identify the BLE device.
    pub mac_address: [u8; 6],
}

impl Default for FakeBle {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeBle {
    const DEVICE_FLAGS: u8 = 0x42;
    const PROFILE_FLAGS: [u8; 3] = [2, 1, 5];

    /// Instantiate a BLE device using a given instance of [`RF24`].
    ///
    /// The `radio` object is consumed because altering the radio's setting will
    /// instigate unexpected behavior.
    pub fn new() -> Self {
        let mut mac_address = [0u8; 6];

        // TODO: randomize this default MAC address.
        mac_address.copy_from_slice(b"nRF24L");

        Self {
            name: [0u8; 12],
            show_pa_level: false,
            mac_address,
        }
    }

    /// Set the BLE device's name for inclusion in advertisements.
    ///
    /// Setting a BLE device name will occupy more bytes from the
    /// 18 available bytes in advertisements. The exact number of bytes occupied
    /// is the length of the given `name` buffer plus 2.
    ///
    /// The maximum supported name length is 10 bytes.
    /// So, up to 12 bytes (10 + 2) will be used in the advertising payload.
    pub fn set_name(&mut self, name: &str) {
        let len = name.len().min(10);
        self.name[2..len + 2].copy_from_slice(&name.as_bytes()[0..len]);
        self.name[0] = len as u8 + 1;
        self.name[1] = 0x08;
    }

    /// Get the current BLE device name included in advertisements.
    ///
    /// If no name is set (with [`FakeBle::set_name()`]), then this function
    /// does nothing.
    /// If a BLE device name has been set, then this function
    /// stores the bytes of the name in the given `name` buffer.
    ///
    /// This function returns the number of bytes changed in the given `name` buffer.
    pub fn get_name(&self, name: &mut [u8]) -> u8 {
        let len = self.name[0];
        if len > 1 {
            let len = (len as usize - 1).min(name.len());
            name[0..len].copy_from_slice(&self.name[2..len + 2]);
            return len as u8;
        }
        0
    }

    /// How many bytes are available in an advertisement payload?
    ///
    /// The `hypothetical` parameter shall be the same `buf` value passed to [`FakeBle::send()`].
    ///
    /// In addition to the given `hypothetical` payload length, this function also
    /// accounts for the current state of [`FakeBle::get_name()`] and
    /// [`FakeBle::show_pa_level`].
    ///
    /// If the returned value is less than `0`, then the `hypothetical` payload will not
    /// be broadcasted.
    pub fn len_available(&self, hypothetical: &[u8]) -> i8 {
        let mut result = 18 - hypothetical.len() as i8;
        let name_len = self.name[0];
        if name_len > 1 {
            result -= name_len as i8 + 1;
        }
        if self.show_pa_level {
            result -= 3;
        }
        result
    }

    /// Hop the radio's current channel to the next BLE compliant frequency.
    ///
    /// Use this function after [`FakeBle::send()`] to comply with BLE specifications.
    /// This is not required, but it is recommended to avoid bandwidth pollution.
    pub fn hop_channel<SPI, DO, DELAY>(
        &self,
        radio: &mut RF24<SPI, DO, DELAY>,
    ) -> Result<(), Nrf24Error<SPI::Error, DO::Error>>
    where
        SPI: SpiDevice,
        DO: OutputPin,
        DELAY: DelayNs,
    {
        let channel = radio.get_channel()?;
        if let Some(channel) = BleChannels::increment(channel) {
            radio.set_channel(channel)?;
        }
        // if the current channel is not a BLE_CHANNEL, then do nothing
        Ok(())
    }

    /// Create a buffer to be used as a BLE advertisement payload.
    ///
    /// This is a helper method to [`FakeBle::send()`], but it is publicly exposed for
    /// advanced usage only (eg. FFI binding).
    ///
    /// If the resulting payload length is larger than 32 bytes, then [`None`] is returned.
    pub fn make_payload(
        &self,
        buf: &[u8],
        pa_level: Option<PaLevel>,
        channel: u8,
    ) -> Option<[u8; 32]> {
        let mut payload_length = buf.len() + 9;

        let mut tx_queue = [0; 32];
        // tx_queue[11..29] is available for user data.
        tx_queue[0] = Self::DEVICE_FLAGS;
        // tx_queue[1] is for the total payload size excluding the following data:
        // - GATT profile flags (tx_queue[0]) at beginning
        // - payload size at tx_queue[1]
        // - CRC24 at the end
        tx_queue[2..8].copy_from_slice(&self.mac_address);
        // flags for declaring device capabilities
        tx_queue[8..11].copy_from_slice(&Self::PROFILE_FLAGS);
        let mut offset = 11;

        if let Some(pa_level) = pa_level {
            let pa_level: i8 = match pa_level {
                rf24::PaLevel::Min => -18,
                rf24::PaLevel::Low => -12,
                rf24::PaLevel::High => -6,
                rf24::PaLevel::Max => 0,
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
            return None;
        }

        tx_queue[1] = payload_length as u8;
        for byte in buf {
            tx_queue[offset] = *byte;
            offset += 1;
        }
        let crc = crc24_ble(&tx_queue[0..offset]);
        tx_queue[offset..offset + 3].copy_from_slice(&crc);
        offset += 3;

        let coefficient = (BleChannels::index_of(channel).unwrap_or_default() + 37) | 0x40;
        whiten(&mut tx_queue[0..offset], coefficient as u8);

        reverse_bits(&mut tx_queue[0..offset]);
        Some(tx_queue)
    }

    /// Send a BLE advertisement
    ///
    /// The `buf` parameter takes a buffer that has been already formatted for
    /// BLE specifications.
    ///
    /// See our convenient API to
    /// - advertise a Battery's remaining change level: [`BatteryService`](struct@crate::services::BatteryService)
    /// - advertise a Temperature measurement: [`TemperatureService`](struct@crate::services::TemperatureService)
    /// - advertise a URL: [`UrlService`](struct@crate::services::UrlService)
    ///
    /// For a custom/proprietary BLE service, the given `buf` must adopt compliance with BLE specifications.
    /// For example, a buffer of `n` bytes shall be formed as follows:
    ///
    /// | index | value |
    /// |:------|:------|
    /// | `0` | `n - 1` |
    /// | `1` | `0xFF`  |
    /// | `2 ... n - 1` | custom data |
    pub fn send<SPI, DO, DELAY>(
        &self,
        radio: &mut RF24<SPI, DO, DELAY>,
        buf: &[u8],
    ) -> Result<bool, Nrf24Error<SPI::Error, DO::Error>>
    where
        SPI: SpiDevice,
        DO: OutputPin,
        DELAY: DelayNs,
    {
        if let Some(tx_queue) = self.make_payload(
            buf,
            if self.show_pa_level {
                Some(radio.get_pa_level()?)
            } else {
                None
            },
            radio.get_channel()?,
        ) {
            // Disregarding any hardware error, `RF24::send()` should
            // always return `Ok(true)` because auto-ack is off.
            return radio.send(&tx_queue, false);
        }
        Ok(false)
    }

    /// Read the first available payload from the radio's RX FIFO
    /// and decode it into a [`BlePayload`].
    ///
    /// <div class="warning">
    ///
    /// The payload must be decoded while the radio is on
    /// the same channel that it received the data.
    /// Otherwise, the decoding process will fail.
    ///
    /// </div>
    ///
    /// Use [`RF24::available`](fn@rf24::radio::prelude::EsbFifo::available) to
    /// check if there is data in the radio's RX FIFO.
    ///
    /// If the payload was somehow malformed or incomplete,
    /// then this function returns a [`None`] value.
    pub fn read<SPI, DO, DELAY>(
        &self,
        radio: &mut RF24<SPI, DO, DELAY>,
    ) -> Result<Option<BlePayload>, Nrf24Error<SPI::Error, DO::Error>>
    where
        SPI: SpiDevice,
        DO: OutputPin,
        DELAY: DelayNs,
    {
        let mut buf = [0u8; 32];
        radio.read(&mut buf, Some(32))?;
        let channel = radio.get_channel()?;
        Ok(BlePayload::from_bytes(&mut buf, channel))
    }
}

/////////////////////////////////////////////////////////////////////////////////
/// unit tests
#[cfg(test)]
mod test {
    extern crate std;
    use super::{ble_config, FakeBle, BLE_ADDRESS, BLE_CHANNEL};
    use crate::{spi_test_expects, test::mk_radio};
    use embedded_hal_mock::eh1::{
        digital::{State, Transaction as PinTransaction},
        spi::Transaction as SpiTransaction,
    };
    use rf24::{CrcLength, PaLevel};
    use std::vec;

    #[test]
    fn name() {
        let mut ble = FakeBle::default();
        let mut expected = [0u8; 10];
        assert_eq!(0, ble.get_name(&mut expected));
        ble.set_name("nRF24L");
        assert_eq!(6, ble.get_name(&mut expected));
        assert!(expected.starts_with(b"nRF24L"));
        assert_eq!(ble.len_available(b""), 10);
    }

    #[test]
    fn mac() {
        let mut mac = [0u8; 6];
        mac.copy_from_slice(b"nRF24L");
        let mut ble = FakeBle::default();
        ble.mac_address.copy_from_slice(&mac);
        assert_eq!(ble.len_available(b""), 18);
    }

    #[test]
    fn pa_level() {
        let mut ble = FakeBle::default();
        assert_eq!(ble.len_available(b""), 18);
        ble.show_pa_level = true;
        assert_eq!(ble.len_available(b""), 15);
    }

    #[test]
    fn config() {
        let config = ble_config();
        assert_eq!(config.channel(), BLE_CHANNEL[0]);
        assert_eq!(config.crc_length(), CrcLength::Disabled);
        assert_eq!(config.auto_ack(), 0);
        assert_eq!(config.auto_retry_count(), 0);
        assert_eq!(config.auto_retry_delay(), 0);
        assert_eq!(config.address_length(), 4);
        let mut address = [0u8; 4];
        config.tx_address(&mut address);
        assert_eq!(address, BLE_ADDRESS);
        config.rx_address(1, &mut address);
        assert_eq!(address, BLE_ADDRESS);
        for pipe in 0..5 {
            let enabled = config.is_rx_pipe_enabled(pipe);
            assert_eq!(enabled, pipe == 1);
        }
    }

    /// radio's register to control the channel
    const RF_CH: u8 = 5;
    /// mnemonic to write to a register
    const W_REGISTER: u8 = 0x20;

    #[test]
    fn channel_hop() {
        let expectations = spi_test_expects![
            (vec![RF_CH, 0], vec![0xEu8, BLE_CHANNEL[0]]),
            (vec![RF_CH | W_REGISTER, BLE_CHANNEL[1]], vec![0xEu8, 0]),
            (vec![RF_CH, 0], vec![0xEu8, BLE_CHANNEL[1]]),
            (vec![RF_CH | W_REGISTER, BLE_CHANNEL[2]], vec![0xEu8, 0]),
            (vec![RF_CH, 0], vec![0xEu8, BLE_CHANNEL[2]]),
            (vec![RF_CH | W_REGISTER, BLE_CHANNEL[0]], vec![0xEu8, 0]),
            (vec![RF_CH, 0], vec![0xEu8, 0]),
        ];
        let mocks = mk_radio(&[], &expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        let ble = FakeBle::default();
        for _ in 0..4 {
            ble.hop_channel(&mut radio).unwrap();
        }
        spi.done();
        ce_pin.done();
    }

    const R_RX_PAYLOAD: u8 = 0x61;
    const STATUS: u8 = 7;
    const MASK_RX_DR: u8 = 1 << 6;

    #[test]
    fn read() {
        let ble = FakeBle::default();
        let channel = BLE_CHANNEL[0];
        let payload = ble.make_payload(&[], None, channel).unwrap();
        let mut buf = [0; 33];
        buf[1..].copy_from_slice(&payload);
        buf[0] = 0xE;
        let mut expected = [0; 33];
        expected[0] = R_RX_PAYLOAD;

        let spi_expectations = spi_test_expects![
            (expected.to_vec(), buf.to_vec()),
            (vec![STATUS | W_REGISTER, MASK_RX_DR], vec![0xEu8, 0]),
            (vec![RF_CH, 0], vec![0xEu8, BLE_CHANNEL[0]]),
        ];
        let mocks = mk_radio(&[], &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);

        let ble_payload = ble.read(&mut radio).unwrap().unwrap();
        assert_eq!(&ble.mac_address, &ble_payload.mac_address);
        spi.done();
        ce_pin.done();
    }

    const MASK_TX_DS: u8 = 1 << 5;
    const MASK_MAX_RT: u8 = 1 << 4;
    const W_TX_PAYLOAD: u8 = 0xA0;
    const FLUSH_TX: u8 = 0xE1;
    const NOP: u8 = 0xFF;
    const RF_SETUP: u8 = 0x06;

    fn send_ce_expects() -> vec::Vec<PinTransaction> {
        vec![
            PinTransaction::set(State::Low),
            PinTransaction::set(State::High),
        ]
    }

    fn send_spi_expects(
        ble: &FakeBle,
        pa_level: Option<PaLevel>,
        big_buf: bool,
    ) -> vec::Vec<SpiTransaction<u8>> {
        let channel = BLE_CHANNEL[0];
        let payload = ble.make_payload(&[], pa_level, channel).unwrap();
        let mut buf = [0; 33];
        buf[0] = 0xE;
        let mut expected = [0; 33];
        expected[0] = W_TX_PAYLOAD;
        expected[1..].copy_from_slice(&payload);

        let mut expectations = vec![];
        let bin_pa_level = pa_level.map(|lvl| match lvl {
            // PaLevel::Min => 0,
            // PaLevel::Low => 2,
            // these tests only use Max and High variants
            PaLevel::High => 4,
            /* PaLevel::Max */ _ => 6,
        });
        if let Some(lvl) = bin_pa_level {
            expectations
                .append(&mut spi_test_expects![(vec![RF_SETUP, 0], vec![0xEu8, lvl]),].to_vec());
        }
        expectations.append(
            &mut spi_test_expects![(
                vec![RF_CH, bin_pa_level.unwrap_or_default()],
                vec![0xEu8, BLE_CHANNEL[0]]
            ),]
            .to_vec(),
        );
        if !big_buf {
            expectations.append(
                &mut spi_test_expects![
                    (vec![FLUSH_TX], vec![0xEu8]),
                    (
                        vec![STATUS | W_REGISTER, MASK_TX_DS | MASK_MAX_RT],
                        vec![0xEu8, 0]
                    ),
                    (expected.to_vec(), buf.to_vec()),
                    (vec![NOP], vec![0xE | MASK_TX_DS]),
                ]
                .to_vec(),
            );
        }
        expectations
    }

    #[test]
    fn send() {
        let ble = FakeBle::default();

        let spi_expectations = send_spi_expects(&ble, None, false);
        let ce_expectations = send_ce_expects();
        let mocks = mk_radio(&ce_expectations, &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);

        assert!(ble.send(&mut radio, &[]).unwrap());
        spi.done();
        ce_pin.done();
    }

    #[test]
    fn send_pa_level() {
        let mut ble = FakeBle::new();
        ble.show_pa_level = true;

        let spi_expectations = send_spi_expects(&ble, Some(PaLevel::Max), false);
        let ce_expectations = send_ce_expects();
        let mocks = mk_radio(&ce_expectations, &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);

        assert!(ble.send(&mut radio, &[]).unwrap());
        spi.done();
        ce_pin.done();
    }

    #[test]
    fn send_big_buf() {
        let mut ble = FakeBle::new();
        ble.show_pa_level = true;

        let spi_expectations = send_spi_expects(&ble, Some(PaLevel::High), true);
        let ce_expectations = [];
        let mocks = mk_radio(&ce_expectations, &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);

        assert!(!ble.send(&mut radio, &[0u8; 20]).unwrap());
        spi.done();
        ce_pin.done();
    }
}
