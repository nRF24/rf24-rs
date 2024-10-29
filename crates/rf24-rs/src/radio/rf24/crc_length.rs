use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};

use super::registers;
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
        let crc_bin = (self._buf[1] & (3 << 2)) >> 2;
        match crc_bin {
            0 => Ok(CrcLength::Disabled),
            2 => Ok(CrcLength::Bit8),
            3 => Ok(CrcLength::Bit16),
            _ => Err(Nrf24Error::BinaryCorruption),
        }
    }

    fn set_crc_length(&mut self, crc_length: CrcLength) -> Result<(), Self::CrcLengthErrorType> {
        let crc_bin = {
            match crc_length {
                CrcLength::Disabled => 0u8,
                CrcLength::Bit8 => 2u8,
                CrcLength::Bit16 => 3u8,
            }
        } << 2;
        self.spi_read(1, registers::CONFIG)?;
        let out = self._buf[1] & !(3 << 2) | crc_bin;
        self.spi_write_byte(registers::CONFIG, out)
    }
}

/////////////////////////////////////////////////////////////////////////////////
/// unit tests
#[cfg(test)]
mod test {
    extern crate std;
    use crate::radio::prelude::EsbCrcLength;
    use crate::radio::rf24::commands;
    use crate::{spi_test_expects, CrcLength};

    use super::{registers, RF24};
    use embedded_hal_mock::eh1::delay::NoopDelay;
    use embedded_hal_mock::eh1::digital::Mock as PinMock;
    use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
    use std::vec;

    #[test]
    pub fn get_crc_length() {
        // Create pin
        let pin_expectations = [];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

        let spi_expectations = spi_test_expects![
            // get the CONFIG register value for each possible result
            (vec![registers::CONFIG, 0u8], vec![0xEu8, 0u8]),
            (vec![registers::CONFIG, 0u8], vec![0xEu8, 0x8u8]),
            (vec![registers::CONFIG, 0x8u8], vec![0xEu8, 0xCu8]),
            (vec![registers::CONFIG, 0xCu8], vec![0xEu8, 4u8]),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        assert_eq!(radio.get_crc_length(), Ok(CrcLength::Disabled));
        assert_eq!(radio.get_crc_length(), Ok(CrcLength::Bit8));
        assert_eq!(radio.get_crc_length(), Ok(CrcLength::Bit16));
        assert_eq!(
            radio.get_crc_length(),
            Err(crate::radio::Nrf24Error::BinaryCorruption)
        );
        spi_mock.done();
        pin_mock.done();
    }

    #[test]
    pub fn set_crc_length() {
        // Create pin
        let pin_expectations = [];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

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
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        radio.set_crc_length(CrcLength::Disabled).unwrap();
        radio.set_crc_length(CrcLength::Bit8).unwrap();
        radio.set_crc_length(CrcLength::Bit16).unwrap();
        spi_mock.done();
        pin_mock.done();
    }
}
