use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};

use crate::{radio::EsbPower, Nrf24Error, RF24};

use super::registers;

impl<SPI, DO, DELAY> EsbPower for RF24<SPI, DO, DELAY>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    type PowerErrorType = Nrf24Error<SPI::Error, DO::Error>;

    /// After calling [`ESBRadio::start_listening()`](fn@crate::radio::EsbRadio::start_listening),
    /// a non-PA/LNA radio will consume about
    /// 13.5mA at [`PaLevel::MAX`](type@crate::enums::PaLevel::MAX).
    /// During active transmission (including RX role when transmitting an auto-ACK
    /// packet), a non-PA/LNA radio will consume about 11.5mA.
    /// In power standby mode (when not receiving nor transmitting), a non-PA/LNA radio
    /// will consume about 26uA (.026mA).
    /// In full power down mode (a sleep state), the radio will consume approximately
    /// 900nA (.0009mA).
    fn power_down(&mut self) -> Result<(), Self::PowerErrorType> {
        self._ce_pin.set_low().map_err(Nrf24Error::Gpo)?; // Guarantee CE is low on powerDown
        self._config_reg &= 0xFD;
        self.spi_write_byte(registers::CONFIG, self._config_reg)?;
        Ok(())
    }

    fn power_up(&mut self, delay: Option<u32>) -> Result<(), Self::PowerErrorType> {
        // if not powered up then power up and wait for the radio to initialize
        if self._config_reg & 0xFD > 0 {
            return Ok(());
        }
        self._config_reg |= 2;
        self.spi_write_byte(registers::CONFIG, self._config_reg)?;

        // For nRF24L01+ to go from power down mode to TX or RX mode it must first pass through stand-by mode.
        // There must be a delay of Tpd2stby (see Table 16.) after the nRF24L01+ leaves power down mode before
        // the CEis set high. - Tpd2stby can be up to 5ms per the 1.0 datasheet
        if delay.is_some_and(|val| val > 0) || delay.is_none() {
            self._wait.delay_us(delay.unwrap_or_else(|| 5000));
        }
        Ok(())
    }
}
