use super::{commands, mnemonics, registers, RF24};
use crate::{
    radio::prelude::{EsbFifo, EsbPayloadLength, EsbPipe, EsbRadio, EsbStatus},
    StatusFlags,
};
use embedded_hal::{
    delay::DelayNs,
    digital::{Error, OutputPin},
    spi::SpiDevice,
};

impl<SPI, DO, DELAY> EsbRadio for RF24<SPI, DO, DELAY>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    fn as_rx(&mut self) -> Result<(), Self::Error> {
        self.config_reg = self.config_reg.as_rx();
        self.spi_write_byte(registers::CONFIG, self.config_reg.into_bits())?;
        self.clear_status_flags(StatusFlags::new())?;
        self.ce_pin.set_high().map_err(|e| e.kind())?;

        // Restore the pipe0 address, if exists
        if let Some(addr) = self.pipe0_rx_addr {
            self.spi_write_buf(
                registers::RX_ADDR_P0,
                &addr[..self.feature.address_length() as usize],
            )?;
        } else {
            self.close_rx_pipe(0)?;
        }
        Ok(())
    }

    fn as_tx(&mut self, tx_address: Option<&[u8]>) -> Result<(), Self::Error> {
        self.ce_pin.set_low().map_err(|e| e.kind())?;

        self.delay_impl.delay_us(self.tx_delay);
        if self.feature.ack_payloads() {
            self.flush_tx()?;
        }

        self.config_reg = self.config_reg.as_tx();
        self.spi_write_byte(registers::CONFIG, self.config_reg.into_bits())?;

        let addr_len = self.feature.address_length();
        if let Some(tx_address) = tx_address {
            let len = tx_address.len().min(addr_len as usize);
            self.tx_address[0..len].copy_from_slice(&tx_address[0..len]);

            // use `spi_transfer()` to avoid multiple borrows of self (`spi_write_buf()` and `tx_address`)
            self.buf[0] = registers::TX_ADDR | commands::W_REGISTER;
            self.buf[1..addr_len as usize + 1]
                .copy_from_slice(&self.tx_address[0..addr_len as usize]);
            self.spi_transfer(addr_len + 1)?;
        }

        // use `spi_transfer()` to avoid multiple borrows of self (`spi_write_buf()` and `tx_address`)
        self.buf[0] = registers::RX_ADDR_P0 | commands::W_REGISTER;
        self.buf[1..addr_len as usize + 1].copy_from_slice(&self.tx_address[0..addr_len as usize]);
        self.spi_transfer(addr_len + 1)?;

        self.spi_read(1, registers::EN_RXADDR)?;
        self.spi_write_byte(registers::EN_RXADDR, self.buf[1] | 1)
    }

    fn is_rx(&self) -> bool {
        self.config_reg.is_rx()
    }

    /// See [`EsbRadio::send()`] for implementation-agnostic detail.
    ///
    /// This function calls [`RF24::flush_tx()`] upon entry, but it does not
    /// deactivate the radio's CE pin upon exit.
    fn send(&mut self, buf: &[u8], ask_no_ack: bool) -> Result<bool, Self::Error> {
        self.ce_pin.set_low().map_err(|e| e.kind())?;
        // this function only handles 1 payload at a time
        self.flush_tx()?; // flush the TX FIFO to ensure we are sending the given buf
        if !self.write(buf, ask_no_ack, true)? {
            // write() also clears the status flags and asserts the CE pin
            return Ok(false);
        }
        self.delay_impl.delay_us(10);
        // now block until we get a tx_ds or tx_df event
        while self.status.into_bits() & (mnemonics::MASK_MAX_RT | mnemonics::MASK_TX_DS) == 0 {
            self.spi_read(0, commands::NOP)?;
        }
        Ok(self.status.tx_ds())
    }

    /// See [`EsbRadio::write()`] for implementation-agnostic detail.
    /// Remember, the nRF24L01's active TX mode is activated by the nRF24L01's CE pin.
    ///
    /// <div class="warning">
    ///
    /// To transmit a payload the radio's CE pin must be active for at least 10 microseconds.
    /// The caller is required to ensure the CE pin has been active for at least 10
    /// microseconds when using this function, thus non-blocking behavior.
    ///
    /// </div>
    fn write(&mut self, buf: &[u8], ask_no_ack: bool, start_tx: bool) -> Result<bool, Self::Error> {
        if self.is_rx() {
            // check if in RX mode to prevent improper radio usage
            return Err(Self::Error::NotAsTxError);
        }
        self.clear_status_flags(StatusFlags::from_bits(
            mnemonics::MASK_MAX_RT | mnemonics::MASK_TX_DS,
        ))?;
        let buf_len = buf.len().min(32);
        // to avoid resizing the given buf, we'll have to use self._buf directly
        self.buf[0] = if !ask_no_ack {
            commands::W_TX_PAYLOAD
        } else {
            commands::W_TX_PAYLOAD_NO_ACK
        };
        self.buf[1..buf_len + 1].copy_from_slice(&buf[..buf_len]);
        // ensure payload_length setting is respected
        if !self.feature.dynamic_payloads() && (buf_len as u8) < self.payload_length {
            // pad buf with zeros
            self.buf[buf_len + 1..self.payload_length as usize + 1].fill(0);
            self.spi_transfer(self.payload_length + 1)?;
        } else {
            self.spi_transfer(buf_len as u8 + 1)?;
        }
        if start_tx {
            self.ce_pin.set_high().map_err(|e| e.kind())?;
        }
        Ok(!self.status.tx_full())
    }

    /// See [`EsbRadio::read()`] for implementation-agnostic detail.
    ///
    /// Remember that each call to [`RF24::read()`] fetches data from the
    /// RX FIFO beginning with the first byte from the first available
    /// payload. A payload is not removed from the RX FIFO until it's
    /// entire length (or more) is fetched.
    ///
    /// - If `len` is less than the available payload's
    ///   length, then the payload remains in the RX FIFO.
    /// - If `len` is greater than the first of multiple
    ///   available payloads, then the data saved to the `buf`
    ///   parameter's object will be supplemented with data from the next
    ///   available payload.
    /// - If `len` is greater than the last available
    ///   payload's length, then the last byte in the payload is used as
    ///   padding for the data saved to the `buf` parameter's object.
    ///   The nRF24L01 will repeatedly use the last byte from the last
    ///   payload even when [`RF24::read()`] is called with an empty RX FIFO.
    fn read(&mut self, buf: &mut [u8], len: Option<u8>) -> Result<u8, Self::Error> {
        let buf_len =
            (buf.len().min(32) as u8).min(len.unwrap_or(if self.feature.dynamic_payloads() {
                self.get_dynamic_payload_length()?
            } else {
                self.payload_length
            }));
        if buf_len == 0 {
            return Ok(0);
        }
        self.spi_read(buf_len, commands::R_RX_PAYLOAD)?;
        buf[0..buf_len as usize].copy_from_slice(&self.buf[1..buf_len as usize + 1]);
        let flags = StatusFlags::from_bits(mnemonics::MASK_RX_DR);
        self.clear_status_flags(flags)?;
        Ok(buf_len)
    }

    fn resend(&mut self) -> Result<bool, Self::Error> {
        if self.is_rx() {
            // if in RX  mode, prevent infinite loop below
            return Ok(false);
        }
        self.rewrite()?;
        self.delay_impl.delay_us(10);
        // now block until a tx_ds or tx_df event occurs
        while self.status.into_bits() & 0x30 == 0 {
            self.spi_read(0, commands::NOP)?;
        }
        Ok(self.status.tx_ds())
    }

    fn rewrite(&mut self) -> Result<(), Self::Error> {
        self.ce_pin.set_low().map_err(|e| e.kind())?;
        let flags = StatusFlags::from_bits(mnemonics::MASK_TX_DS | mnemonics::MASK_MAX_RT);
        self.clear_status_flags(flags)?;
        self.spi_read(0, commands::REUSE_TX_PL)?;
        self.ce_pin.set_high().map_err(|e| e.kind())?;
        Ok(())
    }

    fn get_last_arc(&mut self) -> Result<u8, Self::Error> {
        self.spi_read(1, registers::OBSERVE_TX)?;
        Ok(self.buf[1] & 0xF)
    }
}

/////////////////////////////////////////////////////////////////////////////////
/// unit tests
#[cfg(test)]
mod test {
    extern crate std;
    use super::{commands, mnemonics, registers, EsbPipe, EsbRadio};
    use crate::{spi_test_expects, test::mk_radio};
    use embedded_hal_mock::eh1::{
        digital::{State as PinState, Transaction as PinTransaction},
        spi::Transaction as SpiTransaction,
    };
    use std::vec;

    #[test]
    fn as_rx() {
        let ce_expectations = [PinTransaction::set(PinState::High)];
        let spi_expectations = spi_test_expects![
            // assert PRIM_RX flag
            (
                vec![registers::CONFIG | commands::W_REGISTER, 0xD],
                vec![0xEu8, 0],
            ),
            // clear_status_flags()
            (
                vec![registers::STATUS | commands::W_REGISTER, 0x70],
                vec![0xEu8, 0],
            ),
            // close_rx_pipe(0)
            (vec![registers::EN_RXADDR, 0u8], vec![0xEu8, 1]),
            (
                vec![registers::EN_RXADDR | commands::W_REGISTER, 0],
                vec![0xEu8, 0],
            ),
        ];
        let mocks = mk_radio(&ce_expectations, &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        radio.as_rx().unwrap();
        spi.done();
        ce_pin.done();
    }

    #[test]
    fn as_rx_open_pipe0() {
        let ce_expectations = [
            PinTransaction::set(PinState::High),
            PinTransaction::set(PinState::Low),
            PinTransaction::set(PinState::High),
        ];

        let as_rx_expectations = spi_test_expects![
            // assert PRIM_RX flag
            (
                vec![registers::CONFIG | commands::W_REGISTER, 0xD],
                vec![0xEu8, 0],
            ),
            // clear_status_flags()
            (
                vec![registers::STATUS | commands::W_REGISTER, 0x70],
                vec![0xEu8, 0],
            ),
            // write cached pipe0_rx_addr
            (
                vec![
                    registers::RX_ADDR_P0 | commands::W_REGISTER,
                    0x55,
                    0x55,
                    0x55,
                    0x55,
                    0x55
                ],
                vec![0xEu8, 0, 0, 0, 0, 0]
            ),
        ];

        let mut spi_expectations = spi_test_expects![
            // RX address not immediately written to pipe 0 while in TX mode.
            // so, the SPI transaction is skipped.
            // set EN_RXADDR
            (vec![registers::EN_RXADDR, 0], vec![0xEu8, 0]),
            (
                vec![registers::EN_RXADDR | commands::W_REGISTER, 1],
                vec![0xEu8, 0],
            ),
        ]
        .to_vec();
        spi_expectations.extend(as_rx_expectations.clone());

        // switch back to TX to ensure proper addresses are used
        spi_expectations.extend(spi_test_expects![
            // clear PRIM_RX flag
            (
                vec![registers::CONFIG | commands::W_REGISTER, 0xC],
                vec![0xEu8, 0],
            ),
            // set cached TX address to RX pipe 0 and prepare pipe 0 for auto-ack with same address
            (
                vec![
                    registers::RX_ADDR_P0 | commands::W_REGISTER,
                    0xE7,
                    0xE7,
                    0xE7,
                    0xE7,
                    0xE7
                ],
                vec![0xEu8, 0, 0, 0, 0, 0]
            ),
            // open pipe 0 for TX (regardless of auto-ack)
            (vec![registers::EN_RXADDR, 0u8], vec![0xEu8, 0]),
            (
                vec![registers::EN_RXADDR | commands::W_REGISTER, 1],
                vec![0xEu8, 0],
            ),
        ]);

        // switch back to RX mode to ensure pipe 0 address is restored
        spi_expectations.extend(as_rx_expectations);

        let mocks = mk_radio(&ce_expectations, &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        // starting in TX mode
        assert!(!radio.is_rx());
        let address = [0x55; 5];
        radio.open_rx_pipe(0, &address).unwrap();
        radio.as_rx().unwrap();
        radio.as_tx(None).unwrap();
        radio.as_rx().unwrap();
        spi.done();
        ce_pin.done();
    }

    #[test]
    fn as_tx() {
        let ce_expectations = [PinTransaction::set(PinState::Low)];
        let mut spi_expectations = spi_test_expects![
            // flush_tx() of artifact ACK payloads
            (vec![commands::FLUSH_TX], vec![0xEu8]),
            // clear PRIM_RX flag
            (
                vec![registers::CONFIG | commands::W_REGISTER, 0xC],
                vec![0xEu8, 0],
            ),
        ]
        .to_vec();
        // set cached TX address to RX pipe 0 and prepare pipe 0 for auto-ack with same address
        for reg in [registers::TX_ADDR, registers::RX_ADDR_P0] {
            spi_expectations.extend(spi_test_expects![(
                vec![reg | commands::W_REGISTER, 0xEA, 0xEA, 0xEA, 0xEA, 0xEA],
                vec![0xEu8, 0, 0, 0, 0, 0]
            ),]);
        }
        // open pipe 0 for TX (regardless of auto-ack)
        spi_expectations.extend(spi_test_expects![
            (vec![registers::EN_RXADDR, 0u8], vec![0xEu8, 0]),
            (
                vec![registers::EN_RXADDR | commands::W_REGISTER, 1],
                vec![0xEu8, 0],
            ),
        ]);

        let mocks = mk_radio(&ce_expectations, &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        radio.feature = radio.feature.with_ack_payloads(true);
        let tx_address = [0xEA; 5];
        radio.as_tx(Some(&tx_address)).unwrap();
        spi.done();
        ce_pin.done();
    }

    #[test]
    fn send() {
        let ce_expectations = [
            PinTransaction::set(PinState::Low),
            PinTransaction::set(PinState::High),
            PinTransaction::set(PinState::Low),
            PinTransaction::set(PinState::High),
            PinTransaction::set(PinState::Low),
        ];

        let mut buf = [0u8; 33];
        buf[0] = commands::W_TX_PAYLOAD;
        buf[1..9].copy_from_slice(&[0x55; 8]);

        let spi_expectations = spi_test_expects![
            // flush_tx()
            (vec![commands::FLUSH_TX], vec![0xEu8]),
            // clear_status_flags()
            (
                vec![
                    registers::STATUS | commands::W_REGISTER,
                    mnemonics::MASK_MAX_RT | mnemonics::MASK_TX_DS,
                ],
                vec![0xEu8, 0],
            ),
            // write payload
            (buf.to_vec(), vec![0u8; 33]),
            // spoof a tx_ds event from a NOP write
            (vec![commands::NOP], vec![0xE | mnemonics::MASK_TX_DS]),
            // flush_tx()
            (vec![commands::FLUSH_TX], vec![0xEu8]),
            // clear_status_flags()
            (
                vec![
                    registers::STATUS | commands::W_REGISTER,
                    mnemonics::MASK_MAX_RT | mnemonics::MASK_TX_DS,
                ],
                vec![0xFu8, 0],
            ),
            // spoof full TX FIFO
            // write payload
            (buf.to_vec(), vec![0xFu8; 33]),
            // flush_tx()
            (vec![commands::FLUSH_TX], vec![0xEu8]),
        ];

        let mocks = mk_radio(&ce_expectations, &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        let payload = [0x55; 8];
        assert!(radio.send(&payload, false).unwrap());
        // again using simulated full TX FIFO
        assert!(!radio.send(&payload, false).unwrap());
        radio.config_reg = radio.config_reg.as_rx(); // simulate RX mode
        assert!(radio.send(&payload, false).is_err());
        spi.done();
        ce_pin.done();
    }

    #[test]
    fn ask_no_ack() {
        let mut buf = [0u8; 33];
        buf[0] = commands::W_TX_PAYLOAD_NO_ACK;
        let payload = [0x55; 8];
        buf[1..9].copy_from_slice(&payload);
        let mut dyn_buf = [0x55; 9];
        dyn_buf[0] = commands::W_TX_PAYLOAD_NO_ACK;

        let spi_expectations = spi_test_expects![
            // clear_status_flags()
            (
                vec![
                    registers::STATUS | commands::W_REGISTER,
                    mnemonics::MASK_MAX_RT | mnemonics::MASK_TX_DS,
                ],
                vec![0xEu8, 0u8],
            ),
            // write payload
            (buf.to_vec(), vec![0u8; 33]),
            // clear_status_flags()
            (
                vec![
                    registers::STATUS | commands::W_REGISTER,
                    mnemonics::MASK_MAX_RT | mnemonics::MASK_TX_DS,
                ],
                vec![0xEu8, 0],
            ),
            // write dynamically sized payload
            (dyn_buf.to_vec(), vec![0u8; 9]),
        ];

        let mocks = mk_radio(&[], &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        assert!(radio.write(&payload, true, false).unwrap());
        // upload a dynamically sized payload
        radio.feature = radio.feature.with_dynamic_payloads(true);
        assert!(radio.write(&payload, true, false).unwrap());
        spi.done();
        ce_pin.done();
    }

    #[test]
    fn read() {
        let mut buf_static = [0u8; 33];
        buf_static[0] = commands::R_RX_PAYLOAD;
        let mut buf_dynamic = [0x55u8; 33];
        buf_dynamic[0] = commands::R_RX_PAYLOAD;
        buf_dynamic[1] = 32;

        let spi_expectations = spi_test_expects![
            // read RX payload
            (buf_static.clone().to_vec(), vec![0x55u8; 33]),
            // clear the rx_dr event
            (
                vec![
                    registers::STATUS | commands::W_REGISTER,
                    mnemonics::MASK_RX_DR,
                ],
                vec![0xEu8, 0],
            ),
            // read dynamic payload length
            (vec![commands::R_RX_PL_WID, 0], vec![0xEu8, 32]),
            // read RX payload
            (buf_dynamic.clone().to_vec(), vec![0xAAu8; 33]),
            // clear the rx_dr event
            (
                vec![
                    registers::STATUS | commands::W_REGISTER,
                    mnemonics::MASK_RX_DR,
                ],
                vec![0xEu8, 0],
            ),
        ];

        let mocks = mk_radio(&[], &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        let mut payload = [0; 32];
        assert_eq!(32u8, radio.read(&mut payload, None).unwrap());
        assert_eq!(payload, [0x55; 32]);
        assert_eq!(0u8, radio.read(&mut payload, Some(0)).unwrap());
        radio.feature = radio.feature.with_dynamic_payloads(true);
        assert_eq!(32u8, radio.read(&mut payload, None).unwrap());
        assert_eq!(payload, [0xAA; 32]);
        spi.done();
        ce_pin.done();
    }

    #[test]
    fn resend() {
        let ce_expectations = [
            PinTransaction::set(PinState::Low),
            PinTransaction::set(PinState::High),
        ];
        let spi_expectations = spi_test_expects![
            // clear the tx_df and tx_ds events
            (
                vec![
                    registers::STATUS | commands::W_REGISTER,
                    mnemonics::MASK_MAX_RT | mnemonics::MASK_TX_DS,
                ],
                vec![0xEu8, 0u8],
            ),
            // assert the REUSE_TX_PL flag
            (vec![commands::REUSE_TX_PL], vec![0xEu8]),
            // spoof a tx_ds event from a NOP write
            (vec![commands::NOP], vec![0xE | mnemonics::MASK_TX_DS]),
        ];

        let mocks = mk_radio(&ce_expectations, &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        assert!(radio.resend().unwrap());
        radio.config_reg = radio.config_reg.as_rx(); // simulate RX mode
        assert!(!radio.resend().unwrap());
        spi.done();
        ce_pin.done();
    }

    #[test]
    fn get_last_arc() {
        let spi_expectations = spi_test_expects![
            // get the ARC value from OBSERVE_TX register
            (vec![registers::OBSERVE_TX, 0], vec![0xEu8, 0xFF]),
        ];

        let mocks = mk_radio(&[], &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        assert_eq!(radio.get_last_arc().unwrap(), 15);
        spi.done();
        ce_pin.done();
    }
}
