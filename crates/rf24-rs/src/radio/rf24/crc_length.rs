use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};

use super::{registers, Config};
use crate::radio::{prelude::EsbCrcLength, Nrf24Error, RF24};
use crate::CrcLength;

impl<SPI, DO, DELAY> EsbCrcLength for RF24<SPI, DO, DELAY>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    type CrcLengthErrorType = Nrf24Error<SPI::Error, DO::Error>;

    fn get_crc_length(&mut self) -> Result<CrcLength, Self::CrcLengthErrorType> {
        self.spi_read(1, registers::CONFIG)?;
        if self._buf[1] & Config::CRC_MASK == 4 {
            return Err(Nrf24Error::BinaryCorruption);
        }
        self._config_reg = Config::from_bits(self._buf[1]);
        Ok(self._config_reg.crc_length())
    }

    fn set_crc_length(&mut self, crc_length: CrcLength) -> Result<(), Self::CrcLengthErrorType> {
        self.spi_read(1, registers::CONFIG)?;
        self._config_reg = self._config_reg.with_crc_length(crc_length);
        self.spi_write_byte(registers::CONFIG, self._config_reg.into_bits())
    }
}

/////////////////////////////////////////////////////////////////////////////////
/// unit tests
#[cfg(test)]
mod test {
    extern crate std;
    use super::{registers, EsbCrcLength};
    use crate::{radio::rf24::commands, spi_test_expects, test::mk_radio, CrcLength};
    use embedded_hal_mock::eh1::spi::Transaction as SpiTransaction;
    use std::vec;

    #[test]
    pub fn get_crc_length() {
        let spi_expectations = spi_test_expects![
            // get the CONFIG register value for each possible result
            (vec![registers::CONFIG, 0u8], vec![0xEu8, 0u8]),
            (vec![registers::CONFIG, 0u8], vec![0xEu8, 0x8u8]),
            (vec![registers::CONFIG, 0x8u8], vec![0xEu8, 0xCu8]),
            (vec![registers::CONFIG, 0xCu8], vec![0xEu8, 4u8]),
        ];
        let mocks = mk_radio(&[], &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        assert_eq!(radio.get_crc_length(), Ok(CrcLength::Disabled));
        assert_eq!(radio.get_crc_length(), Ok(CrcLength::Bit8));
        assert_eq!(radio.get_crc_length(), Ok(CrcLength::Bit16));
        assert_eq!(
            radio.get_crc_length(),
            Err(crate::radio::Nrf24Error::BinaryCorruption)
        );
        spi.done();
        ce_pin.done();
    }

    #[test]
    pub fn set_crc_length() {
        let spi_expectations = spi_test_expects![
            // set the CONFIG register value for each possible enumeration of CrcLength
            (vec![registers::CONFIG, 0u8], vec![0xEu8, 4u8]),
            (
                vec![registers::CONFIG | commands::W_REGISTER, 0u8],
                vec![0xEu8, 0u8],
            ),
            (vec![registers::CONFIG, 0u8], vec![0xEu8, 4u8]),
            (
                vec![registers::CONFIG | commands::W_REGISTER, 0x8u8],
                vec![0xEu8, 0u8],
            ),
            (vec![registers::CONFIG, 0u8], vec![0xEu8, 4u8]),
            (
                vec![registers::CONFIG | commands::W_REGISTER, 0xCu8],
                vec![0xEu8, 0u8],
            ),
        ];
        let mocks = mk_radio(&[], &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        radio.set_crc_length(CrcLength::Disabled).unwrap();
        radio.set_crc_length(CrcLength::Bit8).unwrap();
        radio.set_crc_length(CrcLength::Bit16).unwrap();
        spi.done();
        ce_pin.done();
    }
}
