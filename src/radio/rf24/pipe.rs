use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};

use crate::{radio::EsbPipe, Nrf24Error, RF24};

use super::registers;

impl<SPI, DO, DELAY> EsbPipe for RF24<SPI, DO, DELAY>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    type PipeErrorType = Nrf24Error<SPI::Error, DO::Error>;

    fn open_rx_pipe(&mut self, pipe: u8, address: &[u8]) -> Result<(), Self::PipeErrorType> {
        if pipe > 5 {
            return Ok(());
        }

        if pipe < 2 {
            // Clamp the address length used: min(self._addr_length, address.len());
            // This means that we only write the bytes that were passed
            let width = if address.len() < self._addr_length as usize {
                address.len()
            } else {
                self._addr_length as usize
            };

            // If this is pipe 0, cache the address.  This is needed because
            // open_writing_pipe() will overwrite the pipe 0 address, so
            // start_listening() will have to restore it.
            if pipe == 0 {
                let mut cached_addr = self._pipe0_rx_addr.unwrap_or_default();
                for i in 0..width {
                    cached_addr[i] = address[i];
                }
                self._pipe0_rx_addr = Some(cached_addr);
            }
            self.spi_write_buf(registers::RX_ADDR_P0 + pipe, &address[..width])?;
        }
        // For pipes 2-5, only write the MSB
        else {
            self.spi_write_byte(registers::RX_ADDR_P0 + pipe, address[0])?;
        }

        self.spi_read(1, registers::EN_RXADDR)?;
        let out = self._buf[1] | (1 << pipe);
        self.spi_write_byte(registers::EN_RXADDR, out)
    }

    fn open_tx_pipe(&mut self, address: &[u8]) -> Result<(), Self::PipeErrorType> {
        self.spi_write_buf(registers::TX_ADDR, address)?;
        self.spi_write_buf(registers::RX_ADDR_P0, address)
    }

    /// If the given `pipe` number is  not in range [0, 5], then this function does nothing.
    fn close_rx_pipe(&mut self, pipe: u8) -> Result<(), Self::PipeErrorType> {
        if pipe > 5 {
            return Ok(());
        }
        self.spi_read(1, registers::EN_RXADDR)?;
        let out = self._buf[1] & !(1 << pipe);
        self.spi_write_byte(registers::EN_RXADDR, out)?;
        if pipe == 0 {
            self._pipe0_rx_addr = None;
        }
        Ok(())
    }

    fn set_address_length(&mut self, length: u8) -> Result<(), Self::PipeErrorType> {
        let width = match length {
            2 => 0,
            3 => 1,
            4 => 2,
            5 => 3,
            _ => 3,
        };
        self.spi_write_byte(registers::SETUP_AW, width)?;
        self._addr_length = width;
        Ok(())
    }

    fn get_address_length(&mut self) -> Result<u8, Self::PipeErrorType> {
        self.spi_read(1, registers::SETUP_AW)?;
        self._addr_length = self._buf[1] + 2;
        Ok(self._addr_length)
    }
}
