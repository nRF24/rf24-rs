use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};

use crate::{radio::EsbPaLevel, Nrf24Error, PaLevel, RF24};

use super::registers;

impl<SPI, DO, DELAY> EsbPaLevel for RF24<SPI, DO, DELAY>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    type PaLevelErrorType = Nrf24Error<SPI::Error, DO::Error>;

    fn get_pa_level(&mut self) -> Result<PaLevel, Self::PaLevelErrorType> {
        let result = self.spi_read(1, registers::RF_SETUP);
        result?;
        let pa_bin = self._buf[1] >> 1 & 3;
        match pa_bin {
            0 => Ok(PaLevel::MIN),
            1 => Ok(PaLevel::LOW),
            2 => Ok(PaLevel::HIGH),
            3 => Ok(PaLevel::MAX),
            _ => Err(Nrf24Error::BinaryCorruption),
        }
    }

    fn set_pa_level(&mut self, pa_level: PaLevel) -> Result<(), Self::PaLevelErrorType> {
        let pa_bin = {
            match pa_level {
                PaLevel::MIN => 0 as u8,
                PaLevel::LOW => 1 as u8,
                PaLevel::HIGH => 2 as u8,
                PaLevel::MAX => 3 as u8,
            }
        } << 1;
        self.spi_read(1, registers::RF_SETUP)?;
        let out = self._buf[1] & (3 << 1) | pa_bin;
        self.spi_write_byte(registers::RF_SETUP, out)
    }
}
