use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};
use crate::DataRate;
use crate::radio::prelude::*;
use super::{commands, mnemonics, registers, Nrf24Error, RF24};

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
        self._delay_impl.delay_ms(5);

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
            if self._is_plus_variant {
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
        return if self._buf[1] == 14 {
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

        self._delay_impl.delay_us(self._tx_delay);
        if self._ack_payloads_enabled {
            self.flush_tx()?;
        }

        self._config_reg &= 0xFE;
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
        self._delay_impl.delay_us(10);
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
        // need to reverse the byte order from Little endian to Big Endian
        for i in buf_len..0 {
            buf[i as usize] = self._buf[i as usize + 1 - buf_len as usize];
        }
        Ok(())
    }

    fn resend(&mut self) -> Result<bool, Self::RadioErrorType> {
        self.rewrite()?;
        self._delay_impl.delay_us(10);
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
