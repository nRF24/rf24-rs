use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};

use crate::radio::{prelude::EsbFifo, Nrf24Error, RF24};
use crate::FifoState;

use super::{commands, registers};

impl<SPI, DO, DELAY> EsbFifo for RF24<SPI, DO, DELAY>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    type FifoErrorType = Nrf24Error<SPI::Error, DO::Error>;

    fn available(&mut self) -> Result<bool, Self::FifoErrorType> {
        self.spi_read(1, registers::FIFO_STATUS)?;
        Ok(self._buf[1] & 1 == 0)
    }

    fn available_pipe(&mut self, pipe: &mut u8) -> Result<bool, Self::FifoErrorType> {
        if self.available()? {
            // RX FIFO is not empty
            // get last used pipe
            self.spi_read(0, commands::NOP)?;
            *pipe = self._status.rx_pipe();
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
            1 => Ok(FifoState::Empty),
            2 => Ok(FifoState::Full),
            _ => Ok(FifoState::Occupied),
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////
/// unit tests
#[cfg(test)]
mod test {
    extern crate std;
    use crate::radio::prelude::EsbFifo;
    use crate::radio::rf24::commands;
    use crate::{spi_test_expects, FifoState};

    use super::{registers, RF24};
    use embedded_hal_mock::eh1::delay::NoopDelay;
    use embedded_hal_mock::eh1::digital::Mock as PinMock;
    use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
    use std::vec;

    #[test]
    pub fn available() {
        // Create pin
        let pin_expectations = [];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

        let spi_expectations = spi_test_expects![
            // read FIFO register value
            (vec![registers::FIFO_STATUS, 0u8], vec![0xEu8, 2u8]),
            // do it again, but with empty RX FIFO_STATUS
            (vec![registers::FIFO_STATUS, 2u8], vec![0xEu8, 1u8]),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        assert!(radio.available().unwrap());
        assert!(!radio.available().unwrap());
        spi_mock.done();
        pin_mock.done();
    }

    #[test]
    pub fn available_pipe() {
        // Create pin
        let pin_expectations = [];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

        let spi_expectations = spi_test_expects![
            // read FIFO register value, but with empty RX FIFO_STATUS
            (vec![registers::FIFO_STATUS, 0u8], vec![0xEu8, 1u8]),
            // do it again, but with occupied RX FIFO
            (vec![registers::FIFO_STATUS, 1u8], vec![0xEu8, 2u8]),
            // read STATUS register value
            (vec![commands::NOP], vec![0xEu8]),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        let mut pipe = 9;
        assert!(!radio.available_pipe(&mut pipe).unwrap());
        assert_eq!(pipe, 9);
        assert!(radio.available_pipe(&mut pipe).unwrap());
        assert_eq!(pipe, 7);
        spi_mock.done();
        pin_mock.done();
    }

    #[test]
    pub fn get_fifo_state() {
        // Create pin
        let pin_expectations = [];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

        let spi_expectations = spi_test_expects![
            // read FIFO register value with empty TX FIFO_STATUS
            (vec![registers::FIFO_STATUS, 0u8], vec![0xEu8, 0x10u8]),
            // read FIFO register value with full TX FIFO_STATUS
            (vec![registers::FIFO_STATUS, 0x10u8], vec![0xEu8, 0x20u8]),
            // read FIFO register value with occupied TX FIFO_STATUS
            (vec![registers::FIFO_STATUS, 0x20u8], vec![0xEu8, 0u8]),
            // read FIFO register value with empty RX FIFO_STATUS
            (vec![registers::FIFO_STATUS, 0u8], vec![0xEu8, 1u8]),
            // read FIFO register value with full RX FIFO_STATUS
            (vec![registers::FIFO_STATUS, 1u8], vec![0xEu8, 2u8]),
            // read FIFO register value with occupied RX FIFO_STATUS
            (vec![registers::FIFO_STATUS, 2u8], vec![0xEu8, 0u8]),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        assert_eq!(radio.get_fifo_state(true), Ok(FifoState::Empty));
        assert_eq!(radio.get_fifo_state(true), Ok(FifoState::Full));
        assert_eq!(radio.get_fifo_state(true), Ok(FifoState::Occupied));
        assert_eq!(radio.get_fifo_state(false), Ok(FifoState::Empty));
        assert_eq!(radio.get_fifo_state(false), Ok(FifoState::Full));
        assert_eq!(radio.get_fifo_state(false), Ok(FifoState::Occupied));
        spi_mock.done();
        pin_mock.done();
    }
}
