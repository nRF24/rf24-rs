use super::registers;
use crate::radio::{prelude::EsbChannel, Nrf24Error, RF24};
use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};

impl<SPI, DO, DELAY> EsbChannel for RF24<SPI, DO, DELAY>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    type ChannelErrorType = Nrf24Error<SPI::Error, DO::Error>;

    /// The nRF24L01 support 126 channels. The specified `channel` is
    /// clamped to the range [0, 125].
    fn set_channel(&mut self, channel: u8) -> Result<(), Self::ChannelErrorType> {
        self.spi_write_byte(registers::RF_CH, channel.min(125))
    }

    /// See also [`RF24::set_channel()`].
    fn get_channel(&mut self) -> Result<u8, Self::ChannelErrorType> {
        self.spi_read(1, registers::RF_CH)?;
        Ok(self._buf[1])
    }
}

/////////////////////////////////////////////////////////////////////////////////
/// unit tests
#[cfg(test)]
mod test {
    extern crate std;
    use super::{registers, EsbChannel};
    use crate::{spi_test_expects, test::mk_radio};
    use embedded_hal_mock::eh1::spi::Transaction as SpiTransaction;
    use std::vec;

    // set_channel() is already tested in RF24::init() and RF24::start_carrier_wave()

    #[test]
    pub fn get_channel() {
        let spi_expectations = spi_test_expects![
            // get the RF_CH register value
            (vec![registers::RF_CH, 0u8], vec![0xEu8, 76u8]),
        ];
        let mocks = mk_radio(&[], &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        assert_eq!(radio.get_channel().unwrap(), 76u8);
        spi.done();
        ce_pin.done();
    }
}
