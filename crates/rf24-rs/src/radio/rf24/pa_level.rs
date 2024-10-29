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
        let pa_bin = self._buf[1] >> 1 & 3;
        match pa_bin {
            0 => Ok(PaLevel::Min),
            1 => Ok(PaLevel::Low),
            2 => Ok(PaLevel::High),
            _ => Ok(PaLevel::Max),
        }
    }

    fn set_pa_level(&mut self, pa_level: PaLevel) -> Result<(), Self::PaLevelErrorType> {
        let pa_bin = 1
            | (match pa_level {
                PaLevel::Min => 0u8,
                PaLevel::Low => 1u8,
                PaLevel::High => 2u8,
                PaLevel::Max => 3u8,
            } << 1);
        self.spi_read(1, registers::RF_SETUP)?;
        let out = self._buf[1] & !(3 << 1) | pa_bin;
        self.spi_write_byte(registers::RF_SETUP, out)
    }
}

/////////////////////////////////////////////////////////////////////////////////
/// unit tests
#[cfg(test)]
mod test {
    extern crate std;
    use crate::radio::prelude::EsbPaLevel;
    use crate::radio::rf24::commands;
    use crate::{spi_test_expects, PaLevel};

    use super::{registers, RF24};
    use embedded_hal_mock::eh1::delay::NoopDelay;
    use embedded_hal_mock::eh1::digital::Mock as PinMock;
    use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
    use std::vec;

    #[test]
    pub fn get_pa_level() {
        // Create pin
        let pin_expectations = [];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

        let spi_expectations = spi_test_expects![
            // get the RF_SETUP register value for each possible result
            (vec![registers::RF_SETUP, 0u8], vec![0xEu8, 0u8]),
            (vec![registers::RF_SETUP, 0u8], vec![0xEu8, 2u8]),
            (vec![registers::RF_SETUP, 2u8], vec![0xEu8, 4u8]),
            (vec![registers::RF_SETUP, 4u8], vec![0xEu8, 6u8]),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        assert_eq!(radio.get_pa_level(), Ok(PaLevel::Min));
        assert_eq!(radio.get_pa_level(), Ok(PaLevel::Low));
        assert_eq!(radio.get_pa_level(), Ok(PaLevel::High));
        assert_eq!(radio.get_pa_level(), Ok(PaLevel::Max));
        spi_mock.done();
        pin_mock.done();
    }

    #[test]
    pub fn set_pa_level() {
        // Create pin
        let pin_expectations = [];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

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
                vec![registers::RF_SETUP | commands::W_REGISTER, 7u8],
                vec![0xEu8, 0u8],
            ),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        radio.set_pa_level(PaLevel::Min).unwrap();
        radio.set_pa_level(PaLevel::Low).unwrap();
        radio.set_pa_level(PaLevel::High).unwrap();
        radio.set_pa_level(PaLevel::Max).unwrap();
        spi_mock.done();
        pin_mock.done();
    }
}
