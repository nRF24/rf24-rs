use embedded_hal::{
    delay::DelayNs,
    digital::{Error, OutputPin},
    spi::SpiDevice,
};

use crate::radio::{prelude::EsbPower, RF24};

use super::registers;

impl<SPI, DO, DELAY> EsbPower for RF24<SPI, DO, DELAY>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    /// After calling [`EsbRadio::as_rx()`](fn@crate::radio::prelude::EsbRadio::as_rx),
    /// a non-PA/LNA radio will consume about
    /// 13.5mA at [`PaLevel::MAX`](type@crate::types::PaLevel::Max).
    /// During active transmission (including RX role when transmitting an auto-ACK
    /// packet), a non-PA/LNA radio will consume about 11.5mA.
    /// In power standby mode (when not receiving nor transmitting), a non-PA/LNA radio
    /// will consume about 26uA (.026mA).
    /// In full power down mode (a sleep state), the radio will consume approximately
    /// 900nA (.0009mA).
    fn power_down(&mut self) -> Result<(), Self::Error> {
        self.ce_pin.set_low().map_err(|e| e.kind())?; // Guarantee CE is low on powerDown
        self.config_reg = self.config_reg.with_power(false);
        self.spi_write_byte(registers::CONFIG, self.config_reg.into_bits())?;
        Ok(())
    }

    fn power_up(&mut self, delay: Option<u32>) -> Result<(), Self::Error> {
        // if not powered up then power up and wait for the radio to initialize
        if self.config_reg.power() {
            return Ok(());
        }
        self.config_reg = self.config_reg.with_power(true);
        self.spi_write_byte(registers::CONFIG, self.config_reg.into_bits())?;

        // For nRF24L01+ to go from power down mode to TX or RX mode it must first pass through stand-by mode.
        // There must be a delay of Tpd2standby (see Table 16.) after the nRF24L01+ leaves power down mode before
        // the CE is set high. Tpd2standby can be up to 5ms per the 1.0 datasheet
        match delay {
            Some(d) => {
                if d > 0 {
                    self.delay_impl.delay_us(d);
                }
            }
            None => self.delay_impl.delay_us(5000),
        }
        Ok(())
    }

    /// Is the radio powered up?
    fn is_powered(&self) -> bool {
        self.config_reg.power()
    }
}

/////////////////////////////////////////////////////////////////////////////////
/// unit tests
#[cfg(test)]
mod test {
    extern crate std;
    use super::{registers, EsbPower};
    use crate::{radio::rf24::commands, spi_test_expects, test::mk_radio};
    use embedded_hal_mock::eh1::spi::Transaction as SpiTransaction;
    use std::vec;

    #[test]
    pub fn power_up() {
        let spi_expectations = spi_test_expects![
            // get the RF_SETUP register value for each possible result
            (
                vec![registers::CONFIG | commands::W_REGISTER, 0xEu8],
                vec![0xEu8, 0u8],
            ),
        ];
        let mocks = mk_radio(&[], &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        radio.power_up(None).unwrap();
        radio.power_up(None).unwrap();
        spi.done();
        ce_pin.done();
    }

    #[test]
    pub fn power_up_no_blocking() {
        let spi_expectations = spi_test_expects![
            // get the RF_SETUP register value for each possible result
            (
                vec![registers::CONFIG | commands::W_REGISTER, 0xEu8],
                vec![0xEu8, 0u8],
            ),
        ];
        let mocks = mk_radio(&[], &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        radio.power_up(Some(0)).unwrap();
        assert!(radio.is_powered());
        spi.done();
        ce_pin.done();
    }

    #[test]
    pub fn power_up_custom_delay() {
        let spi_expectations = spi_test_expects![
            // get the RF_SETUP register value for each possible result
            (
                vec![registers::CONFIG | commands::W_REGISTER, 0xEu8],
                vec![0xEu8, 0u8],
            ),
        ];
        let mocks = mk_radio(&[], &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        radio.power_up(Some(5000)).unwrap();
        spi.done();
        ce_pin.done();
    }

    #[test]
    pub fn power_getter() {
        let mocks = mk_radio(&[], &[]);
        let (radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        // without calling `RF24::init()`, the lib _assumes_ the radio is powered down.
        assert!(!radio.is_powered());
        spi.done();
        ce_pin.done();
    }
}
