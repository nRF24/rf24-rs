use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};

use super::registers;
use crate::radio::{prelude::EsbDataRate, Nrf24Error, RF24};
use crate::DataRate;

/// A function to set the [`RF24::tx_delay`] in accordance with the desired [`DataRate`].
///
/// This function is only public to the crate::radio::rf24 module.
pub(super) fn set_tx_delay(data_rate: DataRate) -> u32 {
    match data_rate {
        DataRate::Mbps1 => 280,
        DataRate::Mbps2 => 240,
        DataRate::Kbps250 => 505,
    }
}

impl<SPI, DO, DELAY> EsbDataRate for RF24<SPI, DO, DELAY>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    fn get_data_rate(&mut self) -> Result<DataRate, Self::Error> {
        self.spi_read(1, registers::RF_SETUP)?;
        let da_bin = self.buf[1] & DataRate::MASK;
        if da_bin == DataRate::MASK {
            return Err(Nrf24Error::BinaryCorruption);
        }
        Ok(DataRate::from_bits(da_bin))
    }

    fn set_data_rate(&mut self, data_rate: DataRate) -> Result<(), Self::Error> {
        self.tx_delay = set_tx_delay(data_rate);
        self.spi_read(1, registers::RF_SETUP)?;
        let da_bin = data_rate.into_bits();
        let out = self.buf[1] & !DataRate::MASK | da_bin;
        self.spi_write_byte(registers::RF_SETUP, out)
    }
}

/////////////////////////////////////////////////////////////////////////////////
/// unit tests
#[cfg(test)]
mod test {
    extern crate std;
    use super::{registers, DataRate, EsbDataRate, Nrf24Error};
    use crate::{radio::rf24::commands, spi_test_expects, test::mk_radio};
    use embedded_hal_mock::eh1::spi::Transaction as SpiTransaction;
    use std::vec;

    #[test]
    pub fn get_data_rate() {
        let spi_expectations = spi_test_expects![
            // get the RF_SETUP register value for each possible result
            (vec![registers::RF_SETUP, 0], vec![0xEu8, 0]),
            (vec![registers::RF_SETUP, 0], vec![0xEu8, 8]),
            (vec![registers::RF_SETUP, 8], vec![0xEu8, 0x20]),
            (vec![registers::RF_SETUP, 0x20], vec![0xEu8, DataRate::MASK]),
        ];
        let mocks = mk_radio(&[], &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        assert_eq!(radio.get_data_rate(), Ok(DataRate::Mbps1));
        assert_eq!(radio.get_data_rate(), Ok(DataRate::Mbps2));
        assert_eq!(radio.get_data_rate(), Ok(DataRate::Kbps250));
        assert_eq!(radio.get_data_rate(), Err(Nrf24Error::BinaryCorruption));
        spi.done();
        ce_pin.done();
    }

    #[test]
    pub fn set_data_rate() {
        let spi_expectations = spi_test_expects![
            // set the RF_SETUP register value for each possible enumeration of CrcLength
            (vec![registers::RF_SETUP, 0], vec![0xEu8, DataRate::MASK]),
            (
                vec![registers::RF_SETUP | commands::W_REGISTER, 0],
                vec![0xEu8, 0],
            ),
            (vec![registers::RF_SETUP, 0], vec![0xEu8, DataRate::MASK]),
            (
                vec![registers::RF_SETUP | commands::W_REGISTER, 0x8],
                vec![0xEu8, 0],
            ),
            (vec![registers::RF_SETUP, 0], vec![0xEu8, DataRate::MASK]),
            (
                vec![registers::RF_SETUP | commands::W_REGISTER, 0x20],
                vec![0xEu8, 0],
            ),
        ];
        let mocks = mk_radio(&[], &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        radio.set_data_rate(DataRate::Mbps1).unwrap();
        radio.set_data_rate(DataRate::Mbps2).unwrap();
        radio.set_data_rate(DataRate::Kbps250).unwrap();
        spi.done();
        ce_pin.done();
    }
}
