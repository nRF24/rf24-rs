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
        let result = self.spi_read(1, registers::RF_SETUP);
        result?;
        let pa_bin = self._buf[1] >> 1 & 3;
        match pa_bin {
            0 => Ok(PaLevel::MIN),
            1 => Ok(PaLevel::LOW),
            2 => Ok(PaLevel::HIGH),
            3 => Ok(PaLevel::MAX),
            _ => unreachable!(),
        }
    }

    fn set_pa_level(&mut self, pa_level: PaLevel) -> Result<(), Self::PaLevelErrorType> {
        let pa_bin = 1
            | ({
                match pa_level {
                    PaLevel::MIN => 0 as u8,
                    PaLevel::LOW => 1 as u8,
                    PaLevel::HIGH => 2 as u8,
                    PaLevel::MAX => 3 as u8,
                }
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
    use crate::PaLevel;

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

        let spi_expectations = [
            // get the RF_SETUP register value for each possible result
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::RF_SETUP, 0u8], vec![0xEu8, 0u8]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::RF_SETUP, 0u8], vec![0xEu8, 2u8]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::RF_SETUP, 2u8], vec![0xEu8, 4u8]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::RF_SETUP, 4u8], vec![0xEu8, 6u8]),
            SpiTransaction::transaction_end(),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        assert_eq!(radio.get_pa_level(), Ok(PaLevel::MIN));
        assert_eq!(radio.get_pa_level(), Ok(PaLevel::LOW));
        assert_eq!(radio.get_pa_level(), Ok(PaLevel::HIGH));
        assert_eq!(radio.get_pa_level(), Ok(PaLevel::MAX));
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

        let spi_expectations = [
            // set the RF_SETUP register value for each possible enumeration of CrcLength
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::RF_SETUP, 0u8], vec![0xEu8, 7u8]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::RF_SETUP | commands::W_REGISTER, 1u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::RF_SETUP, 0u8], vec![0xEu8, 7u8]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::RF_SETUP | commands::W_REGISTER, 3u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::RF_SETUP, 0u8], vec![0xEu8, 7u8]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::RF_SETUP | commands::W_REGISTER, 5u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::RF_SETUP, 0u8], vec![0xEu8, 0u8]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::RF_SETUP | commands::W_REGISTER, 7u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        radio.set_pa_level(PaLevel::MIN).unwrap();
        radio.set_pa_level(PaLevel::LOW).unwrap();
        radio.set_pa_level(PaLevel::HIGH).unwrap();
        radio.set_pa_level(PaLevel::MAX).unwrap();
        spi_mock.done();
        pin_mock.done();
    }
}
