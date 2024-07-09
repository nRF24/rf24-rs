use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};

use crate::{radio::EsbCrcLength, CrcLength, Nrf24Error, RF24};

use super::registers;

impl<SPI, DO, DELAY> EsbCrcLength for RF24<SPI, DO, DELAY>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    type CrcLengthErrorType = Nrf24Error<SPI::Error, DO::Error>;

    fn get_crc_length(&mut self) -> Result<CrcLength, Self::CrcLengthErrorType> {
        let result = self.spi_read(1, registers::CONFIG);
        result?;
        let crc_bin = self._buf[1] >> 2 & 3;
        match crc_bin {
            0 => Ok(CrcLength::DISABLED),
            5 => Ok(CrcLength::BIT8),
            6 => Ok(CrcLength::BIT16),
            _ => Err(Nrf24Error::BinaryCorruption),
        }
    }

    fn set_crc_length(&mut self, data_rate: CrcLength) -> Result<(), Self::CrcLengthErrorType> {
        let crc_bin = {
            match data_rate {
                CrcLength::DISABLED => 0 as u8,
                CrcLength::BIT8 => 5 as u8,
                CrcLength::BIT16 => 6 as u8,
            }
        } << 2;
        self.spi_read(1, registers::CONFIG)?;
        let out = self._buf[1] & (3 << 2) | crc_bin;
        self.spi_write_byte(registers::CONFIG, out)
    }
}
