use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};
use rf24::{
    radio::{
        prelude::{EsbChannel, EsbInit, EsbPaLevel, EsbRadio},
        Nrf24Error, RadioConfig, RF24,
    },
    CrcLength,
};

use crate::data_manipulation::{crc24_ble, reverse_bits, whitener};

/// The supported channels used amongst BLE devices.
pub const BLE_CHANNEL: [u8; 3] = [2, 26, 80];

/// The only address usable in BLE context.
const BLE_ADDRESS: [u8; 4] = [0x71, 0x91, 0x7d, 0x6b];

pub struct FakeBle<'ble, SPI, DO, DELAY>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    radio: &'ble mut RF24<SPI, DO, DELAY>,
    name: [u8; 10],
    include_pa_level: bool,
    buf: [u8; 32],
}

/// Returns a [`RadioConfig`] object tailored for OTA compatibility with
/// BLE specifications.
pub fn ble_config() -> RadioConfig {
    RadioConfig::default()
        .with_channel(BLE_CHANNEL[0])
        .with_crc_length(CrcLength::Disabled)
        .with_auto_ack(0)
        .with_address_length(4)
        .with_rx_address(1, &BLE_ADDRESS)
        .with_tx_address(&BLE_ADDRESS)
}

impl<'ble, SPI, DO, DELAY> FakeBle<'ble, SPI, DO, DELAY>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    /// Instantiate a BLE device using a given instance of [`RF24`].
    ///
    /// The `radio` object is consumed because altering the radio's setting will
    /// instigate unexpected behavior.
    pub fn new(radio: &'ble mut RF24<SPI, DO, DELAY>) -> Self {
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

    /// Set the BLE device's name for inclusion in advertisements.
    ///
    /// Setting a BLE device name will occupy more bytes from the
    /// 18 available bytes in advertisements. The exact number of bytes occupied
    /// is the length of the given `name` buffer plus 2.
    ///
    /// The maximum supported name length is 8 bytes.
    /// So, up to 10 bytes (8 + 2) will be used in the advertising payload.
    pub fn set_name(&mut self, name: &[u8]) {
        let len = name.len().min(8);
        self.name[2..len + 2].copy_from_slice(&name[0..len]);
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
            let len = len as usize - 1;
            name[0..len].copy_from_slice(&self.name[2..len + 2]);
            return len as u8;
        }
        0
    }

    /// Set the BLE device's MAC address.
    ///
    /// A MAC address is required by BLE specifications.
    /// Use this attribute to uniquely identify the BLE device.
    pub fn set_mac_address(&mut self, address: &[u8; 6]) {
        self.buf[2..8].copy_from_slice(address);
    }

    /// Get the BLE device's MAC address.
    ///
    /// See also [`FakeBle::set_mac_address()`].
    pub fn get_mac_address(&self, address: &mut [u8; 6]) {
        address.copy_from_slice(&self.buf[2..8]);
    }

    /// Enable or disable the inclusion of the radio's PA level in advertisements.
    ///
    /// Enabling this feature occupies 3 bytes of the 18 available bytes in
    /// advertised payloads.
    pub fn show_pa_level(&mut self, enable: bool) {
        self.include_pa_level = enable;
    }

    /// Will the advertisements include the radio's PA level?
    ///
    /// See also [`FakeBle::show_pa_level()`].
    pub fn has_pa_level(&self) -> bool {
        self.include_pa_level
    }

    /// How many bytes are available in an advertisement payload?
    ///
    /// The `hypothetical` parameter shall be the same value passed to [`FakeBle::send()`].
    ///
    /// In addition to the given `hypothetical` payload length, this function also
    /// accounts for the current state of [`FakeBle::get_name()`] and
    /// [`FakeBle::has_pa_level()`].
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

    /// Configure the radio to be used as a BLE device.
    ///
    /// Be sure to call [`EsbInit::init`] before calling this function.
    pub fn init(&mut self) -> Result<(), Nrf24Error<SPI::Error, DO::Error>> {
        self.radio.with_config(&ble_config())
    }

    /// Hop the radio's current channel to the next BLE compliant frequency.
    ///
    /// Use this function after [`FakeBle::send()`] to comply with BLE specifications.
    /// This is not required, but it is recommended to avoid bandwidth pollution.
    pub fn hop_channel(&mut self) -> Result<(), Nrf24Error<SPI::Error, DO::Error>> {
        let channel = self.radio.get_channel()?;
        for (index, ch) in BLE_CHANNEL.iter().enumerate() {
            if *ch == channel {
                if index < (BLE_CHANNEL.len() - 1) {
                    return self.radio.set_channel(BLE_CHANNEL[index + 1]);
                } else {
                    return self.radio.set_channel(BLE_CHANNEL[0]);
                }
            }
        }
        // if the current channel is not a BLE_CHANNEL, then do nothing
        Ok(())
    }

    /// Whiten the given `buf` using the radio's current channel as a coefficient.
    ///
    /// If using this function to de-whiten a received payload, the radio's current
    /// channel is assumed to be the channel in which the payload was received.
    /// If the payload was received on a channel other than the current channel, then
    /// this will not successfully de-whiten the received payload.
    fn whiten(&mut self, buf: &mut [u8]) -> Result<(), Nrf24Error<SPI::Error, DO::Error>> {
        let coefficient = (self.radio.get_channel()? + 37) | 0x40;
        whitener(buf, coefficient);
        Ok(())
    }

    /// Send a BLE advertisement
    ///
    /// The `buf` parameter takes a buffer that has been already formatted for
    /// BLE specifications.
    pub fn send(&mut self, buf: &[u8]) -> Result<bool, Nrf24Error<SPI::Error, DO::Error>> {
        let mut payload_length = buf.len() + 9;
        let mut tx_queue = [0; 32];
        let mut offset = 11;
        tx_queue[0..offset].copy_from_slice(&self.buf[0..offset]);

        if self.include_pa_level {
            let pa_level: i8 = match self.radio.get_pa_level()? {
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

        self.whiten(&mut tx_queue[0..offset])?;

        reverse_bits(&mut tx_queue[0..offset]);

        // Disregarding any hardware error, `RF24::send()` should
        // always return `Ok(true)` because auto-ack is off.
        self.radio.send(&tx_queue[0..offset], false)
    }
}

/////////////////////////////////////////////////////////////////////////////////
/// unit tests
#[cfg(test)]
mod test {
    extern crate std;
    use super::FakeBle;
    use crate::{radio::BLE_CHANNEL, spi_test_expects, test::mk_radio};
    use embedded_hal_mock::eh1::spi::Transaction as SpiTransaction;
    use std::vec;

    #[test]
    fn name() {
        let mut name = [0u8; 6];
        let mocks = mk_radio(&[], &[]);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        let mut ble = FakeBle::new(&mut radio);
        assert_eq!(0, ble.get_name(&mut name));
        name.copy_from_slice(b"nRF24L");
        ble.set_name(&name);
        let mut expected = [0u8; 6];
        assert_eq!(6, ble.get_name(&mut expected));
        assert_eq!(expected, name);
        assert_eq!(ble.len_available(b""), 10);
        spi.done();
        ce_pin.done();
    }

    #[test]
    fn mac() {
        let mut mac = [0u8; 6];
        mac.copy_from_slice(b"nRF24L");
        let mocks = mk_radio(&[], &[]);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        let mut ble = FakeBle::new(&mut radio);
        ble.set_mac_address(&mac);
        let mut expected = [0u8; 6];
        ble.get_mac_address(&mut expected);
        assert_eq!(expected, mac);
        assert_eq!(ble.len_available(b""), 18);
        spi.done();
        ce_pin.done();
    }

    #[test]
    fn pa_level() {
        let mocks = mk_radio(&[], &[]);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        let mut ble = FakeBle::new(&mut radio);
        ble.show_pa_level(true);
        assert!(ble.has_pa_level());
        assert_eq!(ble.len_available(b""), 15);
        spi.done();
        ce_pin.done();
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
        let mut ble = FakeBle::new(&mut radio);
        for _ in 0..4 {
            ble.hop_channel().unwrap();
        }
        spi.done();
        ce_pin.done();
    }
}
