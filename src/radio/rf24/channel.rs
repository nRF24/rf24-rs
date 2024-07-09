use crate::radio::{rf24::registers, EsbChannel};
use crate::{Nrf24Error, RF24};
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
