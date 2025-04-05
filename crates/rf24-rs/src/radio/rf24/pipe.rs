use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};

use crate::radio::{prelude::EsbPipe, Nrf24Error, RF24};

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
            // Clamp the address length used: min(self._address_length, address.len());
            // This means that we only write the bytes that were passed
            let width = address.len().min(self._feature.address_length() as usize);

            // If this is pipe 0, cache the address.  This is needed because
            // open_tx_pipe() will overwrite the pipe 0 address, so
            // as_rx() will have to restore it.
            if pipe == 0 {
                let mut cached_addr = self._pipe0_rx_addr.unwrap_or_default();
                cached_addr[..width].copy_from_slice(&address[..width]);
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
        let width = length.clamp(2, 5);
        self.spi_write_byte(registers::SETUP_AW, width - 2)?;
        self._feature.set_address_length(width);
        Ok(())
    }

    fn get_address_length(&mut self) -> Result<u8, Self::PipeErrorType> {
        self.spi_read(1, registers::SETUP_AW)?;
        let addr_length = self._buf[1].min(0xFD) + 2;
        self._feature.set_address_length(addr_length);
        Ok(addr_length)
    }
}

/////////////////////////////////////////////////////////////////////////////////
/// unit tests
#[cfg(test)]
mod test {
    extern crate std;
    use super::{registers, EsbPipe};
    use crate::{radio::rf24::commands, spi_test_expects, test::mk_radio};
    use embedded_hal_mock::eh1::spi::Transaction as SpiTransaction;
    use std::vec;

    #[test]
    pub fn open_rx_pipe5() {
        let spi_expectations = spi_test_expects![
            // open_rx_pipe(5)
            (
                vec![(registers::RX_ADDR_P0 + 5) | commands::W_REGISTER, 0x55u8],
                vec![0xEu8, 0u8],
            ),
            // set EN_RXADDR
            (vec![registers::EN_RXADDR, 0u8], vec![0xEu8, 1u8]),
            (
                vec![registers::EN_RXADDR | commands::W_REGISTER, 0x21u8],
                vec![0xEu8, 0u8],
            ),
        ];
        let mocks = mk_radio(&[], &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        let address = [0x55u8; 5];
        radio.open_rx_pipe(9, &address).unwrap();
        radio.open_rx_pipe(5, &address).unwrap();
        spi.done();
        ce_pin.done();
    }

    #[test]
    pub fn open_tx_pipe() {
        let mut expected_buf = [0x55u8; 6];
        expected_buf[0] = registers::TX_ADDR | commands::W_REGISTER;
        let mut p0_buf = [0x55u8; 6];
        p0_buf[0] = registers::RX_ADDR_P0 | commands::W_REGISTER;
        let mut response = [0u8; 6];
        response[0] = 0xEu8;

        let spi_expectations = spi_test_expects![
            // open_rx_pipe(5)
            (expected_buf.to_vec(), response.clone().to_vec()),
            (p0_buf.to_vec(), response.to_vec()),
        ];
        let mocks = mk_radio(&[], &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        let address = [0x55u8; 5];
        radio.open_tx_pipe(&address).unwrap();
        radio.close_rx_pipe(9).unwrap();
        spi.done();
        ce_pin.done();
    }

    #[test]
    pub fn set_address_length() {
        let spi_expectations = spi_test_expects![
            // for 2 byte addresses
            // write the SETUP_AW register value
            (
                vec![registers::SETUP_AW | commands::W_REGISTER, 0u8],
                vec![0xEu8, 0u8],
            ),
            // read the SETUP_AW register value
            (vec![registers::SETUP_AW, 0u8], vec![0xEu8, 0u8]),
            // for 3 byte addresses
            // write the SETUP_AW register value
            (
                vec![registers::SETUP_AW | commands::W_REGISTER, 1u8],
                vec![0xEu8, 0u8],
            ),
            // read the SETUP_AW register value
            (vec![registers::SETUP_AW, 0u8], vec![0xEu8, 1u8]),
            // for 4 byte addresses
            // write the SETUP_AW register value
            (
                vec![registers::SETUP_AW | commands::W_REGISTER, 2u8],
                vec![0xEu8, 0u8],
            ),
            // read the SETUP_AW register value
            (vec![registers::SETUP_AW, 0u8], vec![0xEu8, 2u8]),
            // for 5 byte addresses
            // write the SETUP_AW register value
            (
                vec![registers::SETUP_AW | commands::W_REGISTER, 3u8],
                vec![0xEu8, 0u8],
            ),
            // read the SETUP_AW register value
            (vec![registers::SETUP_AW, 0u8], vec![0xEu8, 3u8]),
        ];
        let mocks = mk_radio(&[], &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        radio.set_address_length(2).unwrap();
        assert_eq!(radio.get_address_length().unwrap(), 2u8);
        radio.set_address_length(3).unwrap();
        assert_eq!(radio.get_address_length().unwrap(), 3u8);
        radio.set_address_length(4).unwrap();
        assert_eq!(radio.get_address_length().unwrap(), 4u8);
        radio.set_address_length(5).unwrap();
        assert_eq!(radio.get_address_length().unwrap(), 5u8);
        spi.done();
        ce_pin.done();
    }
}
