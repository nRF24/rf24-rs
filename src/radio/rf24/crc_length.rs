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
        let result = self.spi_read(1, registers::CONFIG);
        result?;
        let crc_bin = (self._buf[1] & (3 << 2)) >> 2;
        match crc_bin {
            0 => Ok(CrcLength::DISABLED),
            2 => Ok(CrcLength::BIT8),
            3 => Ok(CrcLength::BIT16),
            _ => Err(Nrf24Error::BinaryCorruption),
        }
    }

    fn set_crc_length(&mut self, data_rate: CrcLength) -> Result<(), Self::CrcLengthErrorType> {
        let crc_bin = {
            match data_rate {
                CrcLength::DISABLED => 0 as u8,
                CrcLength::BIT8 => 2 as u8,
                CrcLength::BIT16 => 3 as u8,
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
    use crate::CrcLength;

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

        let spi_expectations = [
            // get the CONFIG register value for each possible result
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::CONFIG, 0u8], vec![0xEu8, 0u8]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::CONFIG, 0u8], vec![0xEu8, 0x8u8]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::CONFIG, 0x8u8], vec![0xEu8, 0xCu8]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::CONFIG, 0xCu8], vec![0xEu8, 4u8]),
            SpiTransaction::transaction_end(),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        assert_eq!(radio.get_crc_length(), Ok(CrcLength::DISABLED));
        assert_eq!(radio.get_crc_length(), Ok(CrcLength::BIT8));
        assert_eq!(radio.get_crc_length(), Ok(CrcLength::BIT16));
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

        let spi_expectations = [
            // set the CONFIG register value for each possible enumeration of CrcLength
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::CONFIG, 0u8], vec![0xEu8, 4u8]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::CONFIG | commands::W_REGISTER, 0u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::CONFIG, 0u8], vec![0xEu8, 4u8]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::CONFIG | commands::W_REGISTER, 0x8u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::CONFIG, 0u8], vec![0xEu8, 4u8]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::CONFIG | commands::W_REGISTER, 0xCu8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        radio.set_crc_length(CrcLength::DISABLED).unwrap();
        radio.set_crc_length(CrcLength::BIT8).unwrap();
        radio.set_crc_length(CrcLength::BIT16).unwrap();
        spi_mock.done();
        pin_mock.done();
    }
}
