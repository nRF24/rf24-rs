use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};

use super::registers;
use crate::radio::{prelude::EsbPaLevel, Nrf24Error, RF24};
use crate::PaLevel;

impl<SPI, DO, DELAY> EsbPaLevel for RF24<SPI, DO, DELAY>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    type PaLevelErrorType = Nrf24Error<SPI::Error, DO::Error>;

    fn get_pa_level(&mut self) -> Result<PaLevel, Self::PaLevelErrorType> {
        self.spi_read(1, registers::RF_SETUP)?;
        Ok(PaLevel::from_bits(self._buf[1] & PaLevel::MASK))
    }

    fn set_pa_level(&mut self, pa_level: PaLevel) -> Result<(), Self::PaLevelErrorType> {
        self.spi_read(1, registers::RF_SETUP)?;
        let out = self._buf[1] & !PaLevel::MASK | pa_level.into_bits();
        self.spi_write_byte(registers::RF_SETUP, out)
    }
}

/////////////////////////////////////////////////////////////////////////////////
/// unit tests
#[cfg(test)]
mod test {
    extern crate std;
    use super::{registers, EsbPaLevel, PaLevel};
    use crate::{radio::rf24::commands, spi_test_expects, test::mk_radio};
    use embedded_hal_mock::eh1::spi::Transaction as SpiTransaction;
    use std::vec;

    #[test]
    pub fn get_pa_level() {
        let spi_expectations = spi_test_expects![
            // get the RF_SETUP register value for each possible result
            (vec![registers::RF_SETUP, 0u8], vec![0xEu8, 0u8]),
            (vec![registers::RF_SETUP, 0u8], vec![0xEu8, 2u8]),
            (vec![registers::RF_SETUP, 2u8], vec![0xEu8, 4u8]),
            (vec![registers::RF_SETUP, 4u8], vec![0xEu8, 6u8]),
        ];
        let mocks = mk_radio(&[], &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        assert_eq!(radio.get_pa_level(), Ok(PaLevel::Min));
        assert_eq!(radio.get_pa_level(), Ok(PaLevel::Low));
        assert_eq!(radio.get_pa_level(), Ok(PaLevel::High));
        assert_eq!(radio.get_pa_level(), Ok(PaLevel::Max));
        spi.done();
        ce_pin.done();
    }

    #[test]
    pub fn set_pa_level() {
        let spi_expectations = spi_test_expects![
            // set the RF_SETUP register value for each possible enumeration of CrcLength
            (vec![registers::RF_SETUP, 0u8], vec![0xEu8, 7u8]),
            (
                vec![registers::RF_SETUP | commands::W_REGISTER, 1u8],
                vec![0xEu8, 0u8],
            ),
            (vec![registers::RF_SETUP, 0u8], vec![0xEu8, 7u8]),
            (
                vec![registers::RF_SETUP | commands::W_REGISTER, 3u8],
                vec![0xEu8, 0u8],
            ),
            (vec![registers::RF_SETUP, 0u8], vec![0xEu8, 7u8]),
            (
                vec![registers::RF_SETUP | commands::W_REGISTER, 5u8],
                vec![0xEu8, 0u8],
            ),
            (vec![registers::RF_SETUP, 0u8], vec![0xEu8, 0u8]),
            (
                vec![registers::RF_SETUP | commands::W_REGISTER, 6u8],
                vec![0xEu8, 0u8],
            ),
        ];
        let mocks = mk_radio(&[], &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        radio.set_pa_level(PaLevel::Min).unwrap();
        radio.set_pa_level(PaLevel::Low).unwrap();
        radio.set_pa_level(PaLevel::High).unwrap();
        radio.set_pa_level(PaLevel::Max).unwrap();
        spi.done();
        ce_pin.done();
    }
}
