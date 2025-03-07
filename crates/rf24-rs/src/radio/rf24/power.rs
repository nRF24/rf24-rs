use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};

use crate::radio::{prelude::EsbPower, Nrf24Error, RF24};

use super::registers;

impl<SPI, DO, DELAY> EsbPower for RF24<SPI, DO, DELAY>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    type PowerErrorType = Nrf24Error<SPI::Error, DO::Error>;

    /// After calling [`EsbRadio::as_rx()`](fn@crate::radio::prelude::EsbRadio::as_rx),
    /// a non-PA/LNA radio will consume about
    /// 13.5mA at [`PaLevel::MAX`](type@crate::types::PaLevel::Max).
    /// During active transmission (including RX role when transmitting an auto-ACK
    /// packet), a non-PA/LNA radio will consume about 11.5mA.
    /// In power standby mode (when not receiving nor transmitting), a non-PA/LNA radio
    /// will consume about 26uA (.026mA).
    /// In full power down mode (a sleep state), the radio will consume approximately
    /// 900nA (.0009mA).
    fn power_down(&mut self) -> Result<(), Self::PowerErrorType> {
        self.ce_pin.set_low().map_err(Nrf24Error::Gpo)?; // Guarantee CE is low on powerDown
        self._config_reg = self._config_reg.with_power(false);
        self.spi_write_byte(registers::CONFIG, self._config_reg.into_bits())?;
        Ok(())
    }

    fn power_up(&mut self, delay: Option<u32>) -> Result<(), Self::PowerErrorType> {
        // if not powered up then power up and wait for the radio to initialize
        if self._config_reg.power() {
            return Ok(());
        }
        self._config_reg = self._config_reg.with_power(true);
        self.spi_write_byte(registers::CONFIG, self._config_reg.into_bits())?;

        // For nRF24L01+ to go from power down mode to TX or RX mode it must first pass through stand-by mode.
        // There must be a delay of Tpd2standby (see Table 16.) after the nRF24L01+ leaves power down mode before
        // the CE is set high. Tpd2standby can be up to 5ms per the 1.0 datasheet
        if delay.is_some_and(|val| val > 0) || delay.is_none() {
            self._delay_impl.delay_ns(delay.unwrap_or(5000000));
        }
        Ok(())
    }

    /// Is the radio powered up?
    fn is_powered(&self) -> bool {
        self._config_reg.power()
    }
}

/////////////////////////////////////////////////////////////////////////////////
/// unit tests
#[cfg(test)]
mod test {
    extern crate std;
    use crate::radio::prelude::EsbPower;
    use crate::radio::rf24::commands;
    use crate::spi_test_expects;

    use super::{registers, RF24};
    use embedded_hal_mock::eh1::delay::NoopDelay;
    use embedded_hal_mock::eh1::digital::Mock as PinMock;
    use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
    use std::vec;

    #[test]
    pub fn power_up() {
        // Create pin
        let pin_expectations = [];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

        let spi_expectations = spi_test_expects![
            // get the RF_SETUP register value for each possible result
            (
                vec![registers::CONFIG | commands::W_REGISTER, 0xEu8],
                vec![0xEu8, 0u8],
            ),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        radio.power_up(None).unwrap();
        radio.power_up(None).unwrap();
        // radio.power_up(Some(0)).unwrap();
        spi_mock.done();
        pin_mock.done();
    }

    #[test]
    pub fn power_up_no_blocking() {
        // Create pin
        let pin_expectations = [];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

        let spi_expectations = spi_test_expects![
            // get the RF_SETUP register value for each possible result
            (
                vec![registers::CONFIG | commands::W_REGISTER, 0xEu8],
                vec![0xEu8, 0u8],
            ),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        radio.power_up(Some(0)).unwrap();
        assert!(radio.is_powered());
        spi_mock.done();
        pin_mock.done();
    }

    #[test]
    pub fn power_getter() {
        // Create pin
        let pin_expectations = [];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

        let spi_expectations = vec![];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        // without calling `RF24::init()`, the lib _assumes_ the radio is powered down.
        assert!(!radio.is_powered());
        spi_mock.done();
        pin_mock.done();
    }
}
