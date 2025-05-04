use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};

use crate::radio::{prelude::EsbPipe, RF24};

use super::registers;

impl<SPI, DO, DELAY> EsbPipe for RF24<SPI, DO, DELAY>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    fn open_rx_pipe(&mut self, pipe: u8, address: &[u8]) -> Result<(), Self::Error> {
        if pipe > 5 {
            return Ok(());
        }

        if pipe < 2 {
            // Clamp the address length used: min(self._address_length, address.len());
            // This means that we only write the bytes that were passed
            let width = address.len().min(self.feature.address_length() as usize);

            // If this is pipe 0, cache the address.  This is needed because
            // open_tx_pipe() will overwrite the pipe 0 address, so
            // as_rx() will have to restore it.
            if pipe == 0 {
                let mut cached_addr = self.pipe0_rx_addr.unwrap_or_default();
                cached_addr[..width].copy_from_slice(&address[..width]);
                self.pipe0_rx_addr = Some(cached_addr);
            }
            if self.config_reg.is_rx() || pipe != 0 {
                // skip this if radio is in TX mode and the specified `pipe` is 0
                // NOTE: as_rx() will restore the cached address for pipe 0 (if any)
                self.spi_write_buf(registers::RX_ADDR_P0 + pipe, &address[..width])?;
            }
        }
        // For pipes 2-5, only write the MSB
        else {
            self.spi_write_byte(registers::RX_ADDR_P0 + pipe, address[0])?;
        }

        self.spi_read(1, registers::EN_RXADDR)?;
        let out = self.buf[1] | (1 << pipe);
        self.spi_write_byte(registers::EN_RXADDR, out)
    }

    /// If the given `pipe` number is  not in range [0, 5], then this function does nothing.
    fn close_rx_pipe(&mut self, pipe: u8) -> Result<(), Self::Error> {
        if pipe > 5 {
            return Ok(());
        }
        self.spi_read(1, registers::EN_RXADDR)?;
        let out = self.buf[1] & !(1 << pipe);
        self.spi_write_byte(registers::EN_RXADDR, out)?;
        if pipe == 0 {
            self.pipe0_rx_addr = None;
        }
        Ok(())
    }

    fn set_address_length(&mut self, length: u8) -> Result<(), Self::Error> {
        let width = length.clamp(2, 5);
        self.spi_write_byte(registers::SETUP_AW, width - 2)?;
        self.feature.set_address_length(width);
        Ok(())
    }

    fn get_address_length(&mut self) -> Result<u8, Self::Error> {
        self.spi_read(1, registers::SETUP_AW)?;
        let addr_length = self.buf[1].min(0xFD) + 2;
        self.feature.set_address_length(addr_length);
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
    pub fn open_rx_pipe() {
        let spi_expectations = spi_test_expects![
            // open_rx_pipe(5)
            (
                vec![(registers::RX_ADDR_P0 + 5) | commands::W_REGISTER, 0x55],
                vec![0xEu8, 0],
            ),
            // set EN_RXADDR
            (vec![registers::EN_RXADDR, 0], vec![0xEu8, 1]),
            (
                vec![registers::EN_RXADDR | commands::W_REGISTER, 0x21],
                vec![0xEu8, 0],
            ),
            // open_rx_pipe(0)
            (
                vec![
                    registers::RX_ADDR_P0 | commands::W_REGISTER,
                    0x55,
                    0x55,
                    0x55,
                    0x55,
                    0x55
                ],
                vec![0xEu8, 0, 0, 0, 0, 0],
            ),
            // set EN_RXADDR
            (vec![registers::EN_RXADDR, 0], vec![0xEu8, 2]),
            (
                vec![registers::EN_RXADDR | commands::W_REGISTER, 3],
                vec![0xEu8, 0],
            ),
        ];
        let mocks = mk_radio(&[], &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        let address = [0x55; 5];
        radio.open_rx_pipe(9, &address).unwrap();
        radio.open_rx_pipe(5, &address).unwrap();
        radio.close_rx_pipe(9).unwrap();
        radio.config_reg = radio.config_reg.as_rx();
        radio.open_rx_pipe(0, &address).unwrap();
        spi.done();
        ce_pin.done();
    }

    #[test]
    fn open_rx_pipe0() {
        let spi_expectations = spi_test_expects![
            // open_rx_pipe(5)
            (
                vec![(registers::RX_ADDR_P0 + 5) | commands::W_REGISTER, 0x55],
                vec![0xEu8, 0],
            ),
            // set EN_RXADDR
            (vec![registers::EN_RXADDR, 0], vec![0xEu8, 1]),
            (
                vec![registers::EN_RXADDR | commands::W_REGISTER, 0x21],
                vec![0xEu8, 0],
            ),
        ];
        let mocks = mk_radio(&[], &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        let address = [0x55u8; 5];
        radio.open_rx_pipe(9, &address).unwrap();
        radio.open_rx_pipe(5, &address).unwrap();
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
                vec![registers::SETUP_AW | commands::W_REGISTER, 0],
                vec![0xEu8, 0],
            ),
            // read the SETUP_AW register value
            (vec![registers::SETUP_AW, 0], vec![0xEu8, 0]),
            // for 3 byte addresses
            // write the SETUP_AW register value
            (
                vec![registers::SETUP_AW | commands::W_REGISTER, 1],
                vec![0xEu8, 0],
            ),
            // read the SETUP_AW register value
            (vec![registers::SETUP_AW, 0], vec![0xEu8, 1]),
            // for 4 byte addresses
            // write the SETUP_AW register value
            (
                vec![registers::SETUP_AW | commands::W_REGISTER, 2],
                vec![0xEu8, 0],
            ),
            // read the SETUP_AW register value
            (vec![registers::SETUP_AW, 0], vec![0xEu8, 2]),
            // for 5 byte addresses
            // write the SETUP_AW register value
            (
                vec![registers::SETUP_AW | commands::W_REGISTER, 3],
                vec![0xEu8, 0],
            ),
            // read the SETUP_AW register value
            (vec![registers::SETUP_AW, 0], vec![0xEu8, 3]),
        ];
        let mocks = mk_radio(&[], &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        radio.set_address_length(2).unwrap();
        assert_eq!(radio.get_address_length().unwrap(), 2);
        radio.set_address_length(3).unwrap();
        assert_eq!(radio.get_address_length().unwrap(), 3);
        radio.set_address_length(4).unwrap();
        assert_eq!(radio.get_address_length().unwrap(), 4);
        radio.set_address_length(5).unwrap();
        assert_eq!(radio.get_address_length().unwrap(), 5);
        spi.done();
        ce_pin.done();
    }
}
