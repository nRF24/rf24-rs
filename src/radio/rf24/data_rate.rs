use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};

use super::registers;
use crate::radio::{prelude::EsbDataRate, Nrf24Error, RF24};
use crate::DataRate;

impl<SPI, DO, DELAY> EsbDataRate for RF24<SPI, DO, DELAY>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    type DataRateErrorType = Nrf24Error<SPI::Error, DO::Error>;

    fn get_data_rate(&mut self) -> Result<DataRate, Self::DataRateErrorType> {
        let result = self.spi_read(1, registers::RF_SETUP);
        result?;
        let da_bin = self._buf[1] >> 3 & 5;
        match da_bin {
            0 => Ok(DataRate::Mbps1),
            1 => Ok(DataRate::Mbps2),
            4 => Ok(DataRate::Kbps250),
            _ => Err(Nrf24Error::BinaryCorruption),
        }
    }

    fn set_data_rate(&mut self, data_rate: DataRate) -> Result<(), Self::DataRateErrorType> {
        let da_bin = {
            match data_rate {
                DataRate::Mbps1 => {
                    self._tx_delay = 280;
                    0 as u8
                }
                DataRate::Mbps2 => {
                    self._tx_delay = 240;
                    1 as u8
                }
                DataRate::Kbps250 => {
                    self._tx_delay = 505;
                    4 as u8
                }
            }
        } << 3;
        self.spi_read(1, registers::RF_SETUP)?;
        let out = self._buf[1] & !0x28 | da_bin;
        self.spi_write_byte(registers::RF_SETUP, out)
    }
}

/////////////////////////////////////////////////////////////////////////////////
/// unit tests
#[cfg(test)]
mod test {
    extern crate std;
    use crate::radio::prelude::EsbDataRate;
    use crate::radio::rf24::commands;
    use crate::DataRate;

    use super::{registers, RF24};
    use embedded_hal_mock::eh1::delay::NoopDelay;
    use embedded_hal_mock::eh1::digital::Mock as PinMock;
    use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
    use std::vec;

    #[test]
    pub fn get_data_rate() {
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
            SpiTransaction::transfer_in_place(vec![registers::RF_SETUP, 0u8], vec![0xEu8, 8u8]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::RF_SETUP, 8u8], vec![0xEu8, 0x20u8]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::RF_SETUP, 0x20u8],
                vec![0xEu8, 0x28u8],
            ),
            SpiTransaction::transaction_end(),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        assert_eq!(radio.get_data_rate(), Ok(DataRate::Mbps1));
        assert_eq!(radio.get_data_rate(), Ok(DataRate::Mbps2));
        assert_eq!(radio.get_data_rate(), Ok(DataRate::Kbps250));
        assert_eq!(
            radio.get_data_rate(),
            Err(crate::radio::Nrf24Error::BinaryCorruption)
        );
        spi_mock.done();
        pin_mock.done();
    }

    #[test]
    pub fn set_data_rate() {
        // Create pin
        let pin_expectations = [];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

        let spi_expectations = [
            // set the RF_SETUP register value for each possible enumeration of CrcLength
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::RF_SETUP, 0u8], vec![0xEu8, 0x28u8]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::RF_SETUP | commands::W_REGISTER, 0u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::RF_SETUP, 0u8], vec![0xEu8, 0x28u8]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::RF_SETUP | commands::W_REGISTER, 0x8u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::RF_SETUP, 0u8], vec![0xEu8, 0x28u8]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::RF_SETUP | commands::W_REGISTER, 0x20u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        radio.set_data_rate(DataRate::Mbps1).unwrap();
        radio.set_data_rate(DataRate::Mbps2).unwrap();
        radio.set_data_rate(DataRate::Kbps250).unwrap();
        spi_mock.done();
        pin_mock.done();
    }
}
