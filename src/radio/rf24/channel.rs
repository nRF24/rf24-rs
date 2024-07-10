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
        let ch = {
            if channel > 125 {
                125
            } else {
                channel
            }
        };
        self.spi_write_byte(registers::RF_CH, ch)
    }

    /// See also [`RF24::set_channel()`].
    fn get_channel(&mut self) -> Result<u8, Self::ChannelErrorType> {
        self.spi_read(1, registers::RF_CH)?;
        Ok(self._buf[1].clone())
    }
}

/////////////////////////////////////////////////////////////////////////////////
/// unit tests
#[cfg(test)]
mod test {
    extern crate std;
    use crate::radio::prelude::EsbChannel;

    use super::{registers, RF24};
    use embedded_hal_mock::eh1::delay::NoopDelay;
    use embedded_hal_mock::eh1::digital::Mock as PinMock;
    use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
    use std::vec;

    // set_channel() is already tested in RF24::init() and RF24::start_carrier_wave()

    #[test]
    pub fn get_channel() {
        // Create pin
        let pin_expectations = [];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

        let spi_expectations = [
            // get the RF_CH register value
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::RF_CH, 0u8], vec![0xEu8, 76u8]),
            SpiTransaction::transaction_end(),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        assert_eq!(radio.get_channel().unwrap(), 76u8);
        spi_mock.done();
        pin_mock.done();
    }
}
