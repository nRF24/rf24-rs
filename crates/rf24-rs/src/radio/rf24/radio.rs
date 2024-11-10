use super::{commands, mnemonics, registers, Nrf24Error, RF24};
use crate::{radio::prelude::*, StatusFlags};
use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};

impl<SPI, DO, DELAY> EsbRadio for RF24<SPI, DO, DELAY>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    type RadioErrorType = Nrf24Error<SPI::Error, DO::Error>;

    fn as_rx(&mut self) -> Result<(), Self::RadioErrorType> {
        self._config_reg = self._config_reg.as_rx();
        self.spi_write_byte(registers::CONFIG, self._config_reg.into_bits())?;
        self.clear_status_flags(StatusFlags::new())?;
        self.ce_pin.set_high().map_err(Nrf24Error::Gpo)?;

        // Restore the pipe0 address, if exists
        if let Some(addr) = self._pipe0_rx_addr {
            self.spi_write_buf(
                registers::RX_ADDR_P0,
                &addr[..self._feature.address_length() as usize],
            )?;
        } else {
            self.close_rx_pipe(0)?;
        }
        Ok(())
    }

    fn as_tx(&mut self) -> Result<(), Self::RadioErrorType> {
        self.ce_pin.set_low().map_err(Nrf24Error::Gpo)?;

        self._delay_impl.delay_ns(self.tx_delay * 1000);
        if self._feature.ack_payloads() {
            self.flush_tx()?;
        }

        self._config_reg = self._config_reg.as_tx();
        self.spi_write_byte(registers::CONFIG, self._config_reg.into_bits())?;

        self.spi_read(1, registers::EN_RXADDR)?;
        let out = self._buf[1] | 1;
        self.spi_write_byte(registers::EN_RXADDR, out)
    }

    fn is_rx(&self) -> bool {
        self._config_reg.is_rx()
    }

    /// See [`EsbRadio::send()`] for implementation-agnostic detail.
    ///
    /// This function calls [`RF24::flush_tx()`] upon entry, but it does not
    /// deactivate the radio's CE pin upon exit.
    fn send(&mut self, buf: &[u8], ask_no_ack: bool) -> Result<bool, Self::RadioErrorType> {
        self.ce_pin.set_low().map_err(Nrf24Error::Gpo)?;
        // this function only handles 1 payload at a time
        self.flush_tx()?; // flush the TX FIFO to ensure we are sending the given buf
        if !self.write(buf, ask_no_ack, true)? {
            // write() also clears the status flags and asserts the CE pin
            return Ok(false);
        }
        self._delay_impl.delay_ns(10000);
        // now block until we get a tx_ds or tx_df event
        while self._status.into_bits() & 0x30 == 0 {
            self.spi_read(0, commands::NOP)?;
        }
        Ok(self._status.tx_ds())
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
    fn write(
        &mut self,
        buf: &[u8],
        ask_no_ack: bool,
        start_tx: bool,
    ) -> Result<bool, Self::RadioErrorType> {
        if self.is_rx() {
            // check if in RX mode to prevent improper radio usage
            return Err(Self::RadioErrorType::NotAsTxError);
        }
        self.clear_status_flags(StatusFlags::from_bits(
            mnemonics::MASK_MAX_RT | mnemonics::MASK_TX_DS,
        ))?;
        if self._status.tx_full() {
            // TX FIFO is full already
            return Ok(false);
        }
        let mut buf_len = buf.len().min(32) as u8;
        // to avoid resizing the given buf, we'll have to use self._buf directly
        self._buf[0] = if !ask_no_ack {
            commands::W_TX_PAYLOAD
        } else {
            commands::W_TX_PAYLOAD_NO_ACK
        };
        self._buf[1..(buf_len + 1) as usize].copy_from_slice(&buf[..buf_len as usize]);
        // ensure payload_length setting is respected
        if !self._feature.dynamic_payloads() && buf_len < self._payload_length {
            // pad buf with zeros
            for i in (buf_len + 1)..(self._payload_length + 1) {
                self._buf[i as usize] = 0;
            }
            buf_len = self._payload_length;
        }
        self.spi_transfer(buf_len + 1)?;
        if start_tx {
            self.ce_pin.set_high().map_err(Nrf24Error::Gpo)?;
        }
        Ok(true)
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
    fn read(&mut self, buf: &mut [u8], len: Option<u8>) -> Result<u8, Self::RadioErrorType> {
        let buf_len =
            (buf.len().min(32) as u8).min(len.unwrap_or(if self._feature.dynamic_payloads() {
                self.get_dynamic_payload_length()?
            } else {
                self._payload_length
            }));
        if buf_len == 0 {
            return Ok(0);
        }
        self.spi_read(buf_len, commands::R_RX_PAYLOAD)?;
        for i in 0..buf_len {
            buf[i as usize] = self._buf[i as usize + 1];
        }
        let flags = StatusFlags::from_bits(mnemonics::MASK_RX_DR);
        self.clear_status_flags(flags)?;
        Ok(buf_len)
    }

    fn resend(&mut self) -> Result<bool, Self::RadioErrorType> {
        if self.is_rx() {
            // if in RX  mode, prevent infinite loop below
            return Ok(false);
        }
        self.rewrite()?;
        self._delay_impl.delay_ns(10000);
        // now block until a tx_ds or tx_df event occurs
        while self._status.into_bits() & 0x30 == 0 {
            self.spi_read(0, commands::NOP)?;
        }
        Ok(self._status.tx_ds())
    }

    fn rewrite(&mut self) -> Result<(), Self::RadioErrorType> {
        self.ce_pin.set_low().map_err(Nrf24Error::Gpo)?;
        let flags = StatusFlags::from_bits(mnemonics::MASK_TX_DS | mnemonics::MASK_MAX_RT);
        self.clear_status_flags(flags)?;
        self.spi_read(0, commands::REUSE_TX_PL)?;
        self.ce_pin.set_high().map_err(Nrf24Error::Gpo)
    }

    fn get_last_arc(&mut self) -> Result<u8, Self::RadioErrorType> {
        self.spi_read(1, registers::OBSERVE_TX)?;
        Ok(self._buf[1] & 0xF)
    }
}

/////////////////////////////////////////////////////////////////////////////////
/// unit tests
#[cfg(test)]
mod test {
    extern crate std;
    use super::{commands, registers, RF24};
    use crate::radio::{prelude::*, rf24::mnemonics};
    use crate::spi_test_expects;
    use embedded_hal_mock::eh1::delay::NoopDelay;
    use embedded_hal_mock::eh1::digital::{
        Mock as PinMock, State as PinState, Transaction as PinTransaction,
    };
    use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
    use std::vec;

    #[test]
    pub fn as_rx() {
        // Create pin
        let pin_expectations = [PinTransaction::set(PinState::High)];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

        let spi_expectations = spi_test_expects![
            // assert PRIM_RX flag
            (
                vec![registers::CONFIG | commands::W_REGISTER, 0xDu8],
                vec![0xEu8, 0u8],
            ),
            // clear_status_flags()
            (
                vec![registers::STATUS | commands::W_REGISTER, 0x70u8],
                vec![0xEu8, 0u8],
            ),
            // close_rx_pipe(0)
            (vec![registers::EN_RXADDR, 0u8], vec![0xEu8, 1u8]),
            (
                vec![registers::EN_RXADDR | commands::W_REGISTER, 0u8],
                vec![0xEu8, 0u8],
            ),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        radio.as_rx().unwrap();
        spi_mock.done();
        pin_mock.done();
    }

    #[test]
    pub fn as_rx_open_pipe0() {
        // Create pin
        let pin_expectations = [PinTransaction::set(PinState::High)];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

        let mut buf_expected = [0x55u8; 6];
        buf_expected[0] = registers::RX_ADDR_P0 | commands::W_REGISTER;

        let spi_expectations = spi_test_expects![
            // open_rx_pipe(0)
            (
                buf_expected.clone().to_vec(),
                vec![0xEu8, 0u8, 0u8, 0u8, 0u8, 0u8],
            ),
            // set EN_RXADDR
            (vec![registers::EN_RXADDR, 0u8], vec![0xEu8, 0u8]),
            (
                vec![registers::EN_RXADDR | commands::W_REGISTER, 1u8],
                vec![0xEu8, 0u8],
            ),
            // assert PRIM_RX flag
            (
                vec![registers::CONFIG | commands::W_REGISTER, 0xDu8],
                vec![0xEu8, 0u8],
            ),
            // clear_status_flags()
            (
                vec![registers::STATUS | commands::W_REGISTER, 0x70u8],
                vec![0xEu8, 0u8],
            ),
            // write cached _pipe0_rx_addr
            (buf_expected.to_vec(), vec![0xEu8, 0u8, 0u8, 0u8, 0u8, 0u8]),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        let address = [0x55u8; 5];
        radio.open_rx_pipe(0, &address).unwrap();
        radio.as_rx().unwrap();
        spi_mock.done();
        pin_mock.done();
    }

    #[test]
    pub fn as_tx() {
        // Create pin
        let pin_expectations = [PinTransaction::set(PinState::Low)];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

        let spi_expectations = spi_test_expects![
            // flush_tx() of artifact ACK payloads
            (vec![commands::FLUSH_TX], vec![0xEu8]),
            // clear PRIM_RX flag
            (
                vec![registers::CONFIG | commands::W_REGISTER, 0xCu8],
                vec![0xEu8, 0u8],
            ),
            // open pipe 0 for TX (regardless of auto-ack)
            (vec![registers::EN_RXADDR, 0u8], vec![0xEu8, 0u8]),
            (
                vec![registers::EN_RXADDR | commands::W_REGISTER, 1u8],
                vec![0xEu8, 0u8],
            ),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        radio._feature = radio._feature.with_ack_payloads(true);
        radio.as_tx().unwrap();
        spi_mock.done();
        pin_mock.done();
    }

    #[test]
    pub fn send() {
        // Create pin
        let pin_expectations = [
            PinTransaction::set(PinState::Low),
            PinTransaction::set(PinState::High),
            PinTransaction::set(PinState::Low),
            PinTransaction::set(PinState::Low),
        ];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

        let mut buf = [0u8; 33];
        buf[0] = commands::W_TX_PAYLOAD;
        for i in 0..8 {
            buf[i + 1] = 0x55;
        }

        let spi_expectations = spi_test_expects![
            // flush_tx()
            (vec![commands::FLUSH_TX], vec![0xEu8]),
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
                vec![0xFu8, 0u8],
            ),
            // flush_tx()
            (vec![commands::FLUSH_TX], vec![0xEu8]),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        let payload = [0x55; 8];
        assert!(radio.send(&payload, false).unwrap());
        // again using simulated full TX FIFO
        assert!(!radio.send(&payload, false).unwrap());
        radio._config_reg = radio._config_reg.as_rx(); // simulate RX mode
        assert!(radio.send(&payload, false).is_err());
        spi_mock.done();
        pin_mock.done();
    }

    #[test]
    fn ask_no_ack() {
        // Create pin
        let pin_expectations = [];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

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
                vec![0xEu8, 0u8],
            ),
            // write dynamically sized payload
            (dyn_buf.to_vec(), vec![0u8; 9]),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        assert!(radio.write(&payload, true, false).unwrap());
        // upload a dynamically sized payload
        radio._feature = radio._feature.with_dynamic_payloads(true);
        assert!(radio.write(&payload, true, false).unwrap());
        spi_mock.done();
        pin_mock.done();
    }

    #[test]
    pub fn read() {
        // Create pin
        let pin_expectations = [];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();
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
                vec![0xEu8, 0u8],
            ),
            // read dynamic payload length
            (vec![commands::R_RX_PL_WID, 0u8], vec![0xEu8, 32u8]),
            // read RX payload
            (buf_dynamic.clone().to_vec(), vec![0xAAu8; 33]),
            // clear the rx_dr event
            (
                vec![
                    registers::STATUS | commands::W_REGISTER,
                    mnemonics::MASK_RX_DR,
                ],
                vec![0xEu8, 0u8],
            ),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        let mut payload = [0u8; 32];
        assert_eq!(32u8, radio.read(&mut payload, None).unwrap());
        assert_eq!(payload, [0x55u8; 32]);
        assert_eq!(0u8, radio.read(&mut payload, Some(0)).unwrap());
        radio._feature = radio._feature.with_dynamic_payloads(true);
        assert_eq!(32u8, radio.read(&mut payload, None).unwrap());
        assert_eq!(payload, [0xAA; 32]);
        spi_mock.done();
        pin_mock.done();
    }

    #[test]
    pub fn resend() {
        // Create pin
        let pin_expectations = [
            PinTransaction::set(PinState::Low),
            PinTransaction::set(PinState::High),
        ];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

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
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        assert!(radio.resend().unwrap());
        radio._config_reg = radio._config_reg.as_rx(); // simulate RX mode
        assert!(!radio.resend().unwrap());
        spi_mock.done();
        pin_mock.done();
    }

    #[test]
    pub fn get_last_arc() {
        // Create pin
        let pin_expectations = [];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

        let spi_expectations = spi_test_expects![
            // get the ARC value from OBSERVE_TX register
            (vec![registers::OBSERVE_TX, 0u8], vec![0xEu8, 0xFFu8]),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        assert_eq!(radio.get_last_arc().unwrap(), 15u8);
        spi_mock.done();
        pin_mock.done();
    }
}
