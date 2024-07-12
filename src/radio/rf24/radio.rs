use super::{commands, mnemonics, registers, Nrf24Error, RF24};
use crate::radio::prelude::*;
use crate::DataRate;
use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};

impl<SPI, DO, DELAY> EsbRadio for RF24<SPI, DO, DELAY>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    type RadioErrorType = Nrf24Error<SPI::Error, DO::Error>;

    /// Initialize the radio's hardware using the [`SpiDevice`] and [`OutputPin`] given
    /// to [`RF24::new()`].
    fn init(&mut self) -> Result<(), Self::RadioErrorType> {
        self._ce_pin.set_low().map_err(Nrf24Error::Gpo)?;

        // Must allow the radio time to settle else configuration bits will not necessarily stick.
        // This is actually only required following power up but some settling time also appears to
        // be required after resets too. For full coverage, we'll always assume the worst.
        // Enabling 16b CRC is by far the most obvious case if the wrong timing is used - or skipped.
        // Technically we require 4.5ms + 14us as a worst case. We'll just call it 5ms for good measure.
        // WARNING: Delay is based on P-variant whereby non-P *may* require different timing.
        self._delay_impl.delay_ns(5000000);

        // Set 1500uS (minimum for 32 Byte payload in 250Kbps) timeouts, to make testing a little easier
        // WARNING: If this is ever lowered, either 250KBS mode with AutoAck is broken or maximum packet
        // sizes must never be used. See datasheet for a more complete explanation.
        self.set_auto_retries(5, 15)?;

        // Then set the data rate to the slowest (and most reliable) speed supported by all hardware.
        self.set_data_rate(DataRate::Mbps1)?;

        // detect if is a plus variant & use old toggle features command accordingly
        self.spi_read(1, registers::FEATURE)?;
        let before_toggle = self._buf[1];
        self.toggle_features()?;
        self.spi_read(1, registers::FEATURE)?;
        let after_toggle = self._buf[1];
        self._is_plus_variant = before_toggle == after_toggle;
        if after_toggle > 0 {
            if !self._is_plus_variant {
                // module did not experience power-on-reset, so now features are disabled
                // toggle them back on
                self.toggle_features()?;
            }
            // allow use of ask_no_ack parameter and dynamic payloads by default
            self.spi_write_byte(registers::FEATURE, 0)?;
        }
        self._ack_payloads_enabled = false; // ack payloads disabled by default

        // disable dynamic payloads by default (for all pipes)
        self.spi_write_byte(registers::DYNPD, 0)?;
        self._dynamic_payloads_enabled = false;

        // enable auto-ack on all pipes
        self.spi_write_byte(registers::EN_AA, 0x3F)?;

        // only open RX pipes 0 & 1
        self.spi_write_byte(registers::EN_RXADDR, 3)?;

        // set static payload size to 32 (max) bytes by default
        self.set_payload_length(32)?;
        // set default address length to (max) 5 bytes
        self.set_address_length(5)?;

        // This channel should be universally safe and not bleed over into adjacent spectrum.
        self.set_channel(76)?;

        // Reset current status
        // Notice reset and flush is the last thing we do
        self.clear_status_flags(true, true, true)?;

        // Flush buffers
        self.flush_rx()?;
        self.flush_tx()?;

        // Clear CONFIG register:
        //      Reflect all IRQ events on IRQ pin
        //      Enable PTX
        //      Power Up
        //      16-bit CRC (CRC required by auto-ack)
        // Do not write CE high so radio will remain in standby-I mode
        // PTX should use only 22uA of power
        self._config_reg = 12;
        self.spi_write_byte(registers::CONFIG, self._config_reg)?;

        self.power_up(None)?;

        // if config is not set correctly then there was a bad response from module
        self.spi_read(1, registers::CONFIG)?;
        return if self._buf[1] == self._config_reg {
            Ok(())
        } else {
            Err(Nrf24Error::BinaryCorruption)
        };
    }

    fn start_listening(&mut self) -> Result<(), Self::RadioErrorType> {
        self._config_reg |= 1;
        self.spi_write_byte(registers::CONFIG, self._config_reg)?;
        self.clear_status_flags(true, true, true)?;
        self._ce_pin.set_high().map_err(Nrf24Error::Gpo)?;

        // Restore the pipe0 address, if exists
        if let Some(addr) = self._pipe0_rx_addr {
            self.spi_write_buf(registers::RX_ADDR_P0, &addr[..self._addr_length as usize])?;
        } else {
            self.close_rx_pipe(0)?;
        }
        Ok(())
    }

    fn stop_listening(&mut self) -> Result<(), Self::RadioErrorType> {
        self._ce_pin.set_low().map_err(Nrf24Error::Gpo)?;

        self._delay_impl.delay_ns(self._tx_delay * 1000);
        if self._ack_payloads_enabled {
            self.flush_tx()?;
        }

        self._config_reg &= !1;
        self.spi_write_byte(registers::CONFIG, self._config_reg)?;

        self.spi_read(1, registers::EN_RXADDR)?;
        let out = self._buf[1] | 1;
        self.spi_write_byte(registers::EN_RXADDR, out)
    }

    /// See [`EsbRadio::send()`] for implementation-agnostic detail.
    ///
    /// This function calls [`RF24::flush_tx()`] upon entry, but it does not
    /// deactivate the radio's CE pin upon exit.
    fn send(&mut self, buf: &[u8], ask_no_ack: bool) -> Result<bool, Self::RadioErrorType> {
        self._ce_pin.set_low().map_err(Nrf24Error::Gpo)?;
        // this function only handles 1 payload at a time
        self.flush_tx()?; // flush the TX FIFO to ensure we are sending the given buf
        if !self.write(buf, ask_no_ack, true)? {
            return Ok(false);
        }
        self._delay_impl.delay_ns(10000);
        // now block until we get a tx_ds or tx_df event
        while self._status & 0x30 == 0 {
            self.spi_read(0, commands::NOP)?;
        }
        Ok(self._status & mnemonics::MASK_TX_DS == mnemonics::MASK_TX_DS)
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
        self.clear_status_flags(true, true, true)?;
        if self._status & 1 == 1 {
            // TX FIFO is full already
            return Ok(false);
        }
        let mut buf_len = {
            let len = buf.len();
            if len > 32 {
                32
            } else {
                len
            }
        };
        // to avoid resizing the given buf, we'll have to use self._buf directly
        self._buf[0] = commands::W_TX_PAYLOAD | ((ask_no_ack as u8) << 4);
        for i in 0..buf_len {
            self._buf[i + 1] = buf[i];
        }
        // ensure payload_length setting is respected
        if !self._dynamic_payloads_enabled && buf_len < self._payload_length as usize {
            // pad buf with zeros
            for i in buf_len..self._payload_length as usize {
                self._buf[i + 1] = 0;
            }
            buf_len = self._payload_length as usize;
        }
        self.spi_transfer(buf_len as u8 + 1)?;
        if start_tx {
            self._ce_pin.set_high().map_err(Nrf24Error::Gpo)?;
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
    fn read(&mut self, buf: &mut [u8], len: u8) -> Result<(), Self::RadioErrorType> {
        let buf_len = {
            let max_len = buf.len() as u8;
            if len > max_len {
                max_len
            } else if len > 32 {
                32u8
            } else {
                len
            }
        };
        if buf_len == 0 {
            return Ok(());
        }
        self.spi_read(buf_len, commands::R_RX_PAYLOAD)?;
        for i in 0..buf_len {
            buf[i as usize] = self._buf[i as usize + 1];
        }
        self.clear_status_flags(true, false, false)?;
        Ok(())
    }

    fn resend(&mut self) -> Result<bool, Self::RadioErrorType> {
        self.rewrite()?;
        self._delay_impl.delay_ns(10000);
        // now block until a tx_ds or tx_df event occurs
        while self._status & 0x30 == 0 {
            self.spi_read(0, commands::NOP)?;
        }
        Ok(self._status & mnemonics::MASK_TX_DS == mnemonics::MASK_TX_DS)
    }

    fn rewrite(&mut self) -> Result<(), Self::RadioErrorType> {
        self._ce_pin.set_low().map_err(Nrf24Error::Gpo)?;
        self.clear_status_flags(false, true, true)?;
        self.spi_read(0, commands::REUSE_TX_PL)?;
        self._ce_pin.set_high().map_err(Nrf24Error::Gpo)
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
    use crate::radio::prelude::*;
    use crate::radio::rf24::mnemonics;
    use crate::spi_test_expects;
    use embedded_hal_mock::eh1::delay::NoopDelay;
    use embedded_hal_mock::eh1::digital::{
        Mock as PinMock, State as PinState, Transaction as PinTransaction,
    };
    use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
    use std::vec;

    #[test]
    pub fn init() {
        // Create pin
        let pin_expectations = [PinTransaction::set(PinState::Low)];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

        let spi_expectations = spi_test_expects![
            // set_auto_retries()
            (
                vec![registers::SETUP_RETR | commands::W_REGISTER, 0x5Fu8],
                vec![0xEu8, 0u8],
            ),
            (vec![registers::RF_SETUP, 0u8], vec![0xEu8, 7u8]),
            (
                vec![registers::RF_SETUP | commands::W_REGISTER, 7u8],
                vec![0xEu8, 0u8],
            ),
            // we're mocking a non-plus variant here for added coverage
            // read FEATURE register
            (vec![registers::FEATURE, 0u8], vec![0xEu8, 0u8]),
            // toggle_features()
            (vec![commands::ACTIVATE, 0x73u8], vec![0xEu8, 0u8]),
            // read FEATURE register
            (vec![registers::FEATURE, 0u8], vec![0xEu8, 7u8]),
            // toggle_features()
            (vec![commands::ACTIVATE, 0x73u8], vec![0xEu8, 0u8]),
            // we're also mocking a non-plus radio that didn't reset on boot,
            // so lib wil clear the FEATURE register
            // write FEATURE register
            (
                vec![registers::FEATURE | commands::W_REGISTER, 0u8],
                vec![0xEu8, 0u8],
            ),
            // disable dynamic payloads
            (
                vec![registers::DYNPD | commands::W_REGISTER, 0u8],
                vec![0xEu8, 0u8],
            ),
            // enable auto-ack
            (
                vec![registers::EN_AA | commands::W_REGISTER, 0x3Fu8],
                vec![0xEu8, 0u8],
            ),
            // open pipes 0 & 1
            (
                vec![registers::EN_RXADDR | commands::W_REGISTER, 3u8],
                vec![0xEu8, 0u8],
            ),
            // set payload length to 32 bytes on all pipes
            (
                vec![registers::RX_PW_P0 + 0 | commands::W_REGISTER, 32u8],
                vec![0xEu8, 0u8],
            ),
            (
                vec![registers::RX_PW_P0 + 1 | commands::W_REGISTER, 32u8],
                vec![0xEu8, 0u8],
            ),
            (
                vec![registers::RX_PW_P0 + 2 | commands::W_REGISTER, 32u8],
                vec![0xEu8, 0u8],
            ),
            (
                vec![registers::RX_PW_P0 + 3 | commands::W_REGISTER, 32u8],
                vec![0xEu8, 0u8],
            ),
            (
                vec![registers::RX_PW_P0 + 4 | commands::W_REGISTER, 32u8],
                vec![0xEu8, 0u8],
            ),
            (
                vec![registers::RX_PW_P0 + 5 | commands::W_REGISTER, 32u8],
                vec![0xEu8, 0u8],
            ),
            // set_address_length(5)
            (
                vec![registers::SETUP_AW | commands::W_REGISTER, 3u8],
                vec![0xEu8, 0u8],
            ),
            // set_channel(76)
            (
                vec![registers::RF_CH | commands::W_REGISTER, 76u8],
                vec![0xEu8, 0u8],
            ),
            // clear_status_flags()
            (
                vec![registers::STATUS | commands::W_REGISTER, 0x70u8],
                vec![0xEu8, 0u8],
            ),
            // flush_rx()
            (vec![commands::FLUSH_RX], vec![0xEu8]),
            // flush_tx()
            (vec![commands::FLUSH_TX], vec![0xEu8]),
            // set CONFIG register
            (
                vec![registers::CONFIG | commands::W_REGISTER, 12u8],
                vec![0xEu8, 0u8],
            ),
            // power_up()
            (
                vec![registers::CONFIG | commands::W_REGISTER, 14u8],
                vec![0xEu8, 0u8],
            ),
            // read CONFIG to test for binary corruption on SPI lines
            (vec![registers::CONFIG, 0u8], vec![0xEu8, 14u8]),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        radio.init().unwrap();
        assert!(!radio.is_plus_variant());
        spi_mock.done();
        pin_mock.done();
    }

    #[test]
    pub fn start_listening() {
        // Create pin
        let pin_expectations = [PinTransaction::set(PinState::High)];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

        let spi_expectations = spi_test_expects![
            // assert PRIM_RX flag
            (
                vec![registers::CONFIG | commands::W_REGISTER, 1u8],
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
        radio.start_listening().unwrap();
        spi_mock.done();
        pin_mock.done();
    }

    #[test]
    pub fn start_listening_open_pipe0() {
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
                vec![registers::CONFIG | commands::W_REGISTER, 1u8],
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
        radio.start_listening().unwrap();
        spi_mock.done();
        pin_mock.done();
    }

    #[test]
    pub fn stop_listening() {
        // Create pin
        let pin_expectations = [PinTransaction::set(PinState::Low)];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

        let spi_expectations = spi_test_expects![
            // enable ACK payloads
            // read/write FEATURE register
            (vec![registers::FEATURE, 0u8], vec![0xEu8, 0u8]),
            (
                vec![
                    registers::FEATURE | commands::W_REGISTER,
                    mnemonics::EN_ACK_PAY | mnemonics::EN_DPL,
                ],
                vec![0xEu8, 0u8],
            ),
            // read/write DYNPD register
            (vec![registers::DYNPD, 0u8], vec![0xEu8, 0u8]),
            (
                vec![registers::DYNPD | commands::W_REGISTER, 3u8],
                vec![0xEu8, 0u8],
            ),
            // flush_tx() of artifact ACK payloads
            (vec![commands::FLUSH_TX], vec![0xEu8]),
            // clear PRIM_RX flag
            (
                vec![registers::CONFIG | commands::W_REGISTER, 0u8],
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
        radio.allow_ack_payloads(true).unwrap();
        radio.stop_listening().unwrap();
        spi_mock.done();
        pin_mock.done();
    }

    #[test]
    pub fn send() {
        // Create pin
        let pin_expectations = [
            PinTransaction::set(PinState::Low),
            PinTransaction::set(PinState::High),
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
            // flush_tx() of artifact ACK payloads
            (vec![commands::FLUSH_TX], vec![0xEu8]),
            // clear_status_flags()
            (
                vec![
                    registers::STATUS | commands::W_REGISTER,
                    mnemonics::MASK_MAX_RT | mnemonics::MASK_RX_DR | mnemonics::MASK_TX_DS,
                ],
                vec![0xEu8, 0u8],
            ),
            // write payload
            (buf.to_vec(), vec![0u8; 33]),
            // spoof a tx_ds event from a NOP write
            (vec![commands::NOP], vec![0xE | mnemonics::MASK_TX_DS]),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        let payload = [0x55; 8];
        assert!(radio.send(&payload, false).unwrap());
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
        let mut buf = [0u8; 33];
        buf[0] = commands::R_RX_PAYLOAD;

        let spi_expectations = spi_test_expects![
            // read RX payload
            (buf.clone().to_vec(), vec![0x55u8; 33]),
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
        radio.read(&mut payload, 32).unwrap();
        assert_eq!(payload, [0x55u8; 32]);
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
