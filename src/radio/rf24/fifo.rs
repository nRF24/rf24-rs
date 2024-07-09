use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};

use crate::{radio::EsbFifo, FifoState, Nrf24Error, RF24};

use super::{commands, registers};

impl<SPI, DO, DELAY> EsbFifo for RF24<SPI, DO, DELAY>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    type FifoErrorType = Nrf24Error<SPI::Error, DO::Error>;

    fn available(&mut self) -> Result<bool, Self::FifoErrorType> {
        self.available_pipe(&mut None)
    }

    fn available_pipe(&mut self, pipe: &mut Option<u8>) -> Result<bool, Self::FifoErrorType> {
        self.spi_read(1, registers::FIFO_STATUS)?;
        if self._buf[1] & 1 == 0 {
            // RX FIFO is not empty
            // get last used pipe (if pipe != None)
            if let Some(rx_pipe) = pipe {
                self.spi_read(1, registers::STATUS)?;
                *rx_pipe = &self._buf[1].clone() >> 1 & 7;
            }
            return Ok(true);
        }
        Ok(false)
    }

    /// Use this to discard all 3 layers in the radio's RX FIFO.
    fn flush_rx(&mut self) -> Result<(), Self::FifoErrorType> {
        self.spi_read(0, commands::FLUSH_RX)
    }

    /// Use this to discard all 3 layers in the radio's TX FIFO.
    fn flush_tx(&mut self) -> Result<(), Self::FifoErrorType> {
        self.spi_read(0, commands::FLUSH_TX)
    }

    fn get_fifo_state(&mut self, about_tx: bool) -> Result<FifoState, Self::FifoErrorType> {
        self.spi_read(1, registers::FIFO_STATUS)?;
        let offset = about_tx as u8 * 4;
        let status = (self._buf[1] & (3 << offset)) >> offset;
        match status {
            0 => Ok(FifoState::Occupied),
            1 => Ok(FifoState::Empty),
            2 => Ok(FifoState::Full),
            _ => Err(Nrf24Error::BinaryCorruption),
        }
    }
}
