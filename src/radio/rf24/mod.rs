use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};
mod auto_ack;
mod channel;
mod crc_length;
mod data_rate;
mod fifo;
mod pa_level;
mod payload_length;
mod pipe;
mod power;
pub mod registers;
mod status;
use super::{
    EsbAutoAck, EsbChannel, EsbCrcLength, EsbDataRate, EsbFifo, EsbPaLevel, EsbPayloadLength,
    EsbPipe, EsbPower, EsbStatus,
};
use crate::{
    enums::{CrcLength, DataRate, PaLevel},
    EsbRadio,
};

/// An collection of error types to describe hardware malfunctions.
#[derive(Clone, Copy, Debug)]
pub enum Nrf24Error<SPI, DO> {
    /// Represents a SPI transaction error.
    Spi(SPI),
    /// Represents a DigitalOutput error.
    Gpo(DO),
    /// Represents a corruption of binary data (as it was transferred over the SPI bus' MISO)
    BinaryCorruption,
}

/// A private module encapsulating SPI commands for the nRF24L01.
mod commands {
    pub const W_REGISTER: u8 = 0x20;
    pub const ACTIVATE: u8 = 0x50;
    pub const R_RX_PL_WID: u8 = 0x60;
    pub const R_RX_PAYLOAD: u8 = 0x61;
    pub const W_TX_PAYLOAD: u8 = 0xA0;
    pub const W_ACK_PAYLOAD: u8 = 0xA8;
    pub const FLUSH_TX: u8 = 0xE1;
    pub const FLUSH_RX: u8 = 0xE2;
    pub const REUSE_TX_PL: u8 = 0xE3;
    pub const NOP: u8 = 0xFF;
}

/// A private module to encapsulate bit mnemonics
mod mnemonics {
    pub const MASK_RX_DR: u8 = 1 << 6;
    pub const MASK_TX_DS: u8 = 1 << 5;
    pub const MASK_MAX_RT: u8 = 1 << 4;
    pub const EN_DPL: u8 = 1 << 2;
    pub const EN_ACK_PAY: u8 = 1 << 1;
}

pub struct RF24<SPI, DO, DELAY> {
    // private attributes
    _spi: SPI,
    _status: u8,
    _ce_pin: DO,
    _buf: [u8; 33],
    _is_plus_variant: bool,
    _ack_payloads_enabled: bool,
    _dynamic_payloads_enabled: bool,
    _config_reg: u8,
    _wait: DELAY,
    _pipe0_rx_addr: Option<[u8; 5]>,
    _addr_length: u8,
    _tx_delay: u32,
    _payload_length: u8,
}

impl<SPI, DO, DELAY> RF24<SPI, DO, DELAY>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    /// Instantiate an [`RF24`] object for use on the specified
    /// `spi` bus with the given `ce_pin`.
    ///
    /// The radio's CSN pin (aka Chip Select pin) shall be defined
    /// when instantiating the [`SpiDevice`] object (passed to the
    /// `spi` parameter).
    pub fn new(ce_pin: DO, spi: SPI, delay_impl: DELAY) -> RF24<SPI, DO, DELAY> {
        RF24 {
            _status: 0,
            _ce_pin: ce_pin,
            _spi: spi,
            _buf: [0 as u8; 33],
            _is_plus_variant: true,
            _ack_payloads_enabled: false,
            _dynamic_payloads_enabled: false,
            _config_reg: 0,
            _wait: delay_impl,
            _pipe0_rx_addr: None,
            _addr_length: 5,
            _tx_delay: 250,
            _payload_length: 32,
        }
    }

    fn spi_transfer(&mut self, len: u8) -> Result<(), Nrf24Error<SPI::Error, DO::Error>> {
        self._spi
            .transfer_in_place(&mut self._buf[..len as usize])
            .map_err(Nrf24Error::Spi)?;
        self._status = self._buf[0];
        Ok(())
    }

    /// This is also used to write SPI commands that consist of 1 byte:
    /// ```
    /// self.spi_read(0, commands::NOP)?;
    /// // STATUS register is now stored in self._status
    /// ```
    fn spi_read(&mut self, len: u8, command: u8) -> Result<(), Nrf24Error<SPI::Error, DO::Error>> {
        self._buf[0] = command;
        self.spi_transfer(len + 1)
    }

    fn spi_write_byte(
        &mut self,
        command: u8,
        byte: u8,
    ) -> Result<(), Nrf24Error<SPI::Error, DO::Error>> {
        self._buf[0] = command | commands::W_REGISTER;
        self._buf[1] = byte;
        self.spi_transfer(2)
    }

    fn spi_write_buf(
        &mut self,
        command: u8,
        buf: &[u8],
    ) -> Result<(), Nrf24Error<SPI::Error, DO::Error>> {
        self._buf[0] = command;
        // buffers in rust are stored in Big Endian memory.
        // the nRF24L01 expects multi-byte SPI transactions to be Little Endian.
        // So, reverse the byteOrder when loading the user's `buf`` into the lib's `_buf``
        let buf_len = buf.len();
        for i in buf_len..0 {
            self._buf[i + 1 - buf_len] = buf[i];
        }
        self.spi_transfer(buf_len as u8 + 1)
    }

    /// A private function to write a special SPI command specific to older
    /// non-plus variants of the nRF24L01 radio module. It has no effect on plus variants.
    fn toggle_features(&mut self) -> Result<(), Nrf24Error<SPI::Error, DO::Error>> {
        self._buf[0] = commands::ACTIVATE;
        self._buf[1] = 0x73;
        self.spi_transfer(2)
    }

    /// Is this radio a nRF24L01+ variant?
    ///
    /// The bool that this function returns is only valid _after_ calling [`RF24::init()`].
    pub fn is_plus_variant(&mut self) -> bool {
        self._is_plus_variant
    }

    pub fn test_rpd(&mut self) -> Result<bool, Nrf24Error<SPI::Error, DO::Error>> {
        self.spi_read(1, registers::RPD)?;
        Ok(self._buf[1] & 1 == 1)
    }

    pub fn start_carrier_wave(
        &mut self,
        level: PaLevel,
        channel: u8,
    ) -> Result<(), Nrf24Error<SPI::Error, DO::Error>> {
        self.stop_listening()?;
        self.spi_read(1, registers::RF_SETUP)?;
        self.spi_write_byte(registers::RF_SETUP, self._buf[1] | 0x84)?;
        if self._is_plus_variant {
            self.set_auto_ack(false)?;
            self.set_auto_retries(0, 0)?;
            let buf = [0xFF; 32];

            // use write_register() instead of openWritingPipe() to bypass
            // truncation of the address with the current RF24::addr_width value
            self.spi_write_buf(registers::TX_ADDR, &buf[..5])?;
            self.flush_tx()?; // so we can write to top level

            // use write_register() instead of write_payload() to bypass
            // truncation of the payload with the current RF24::payload_size value
            self.spi_write_buf(commands::W_TX_PAYLOAD, &buf)?;

            self.set_crc_length(CrcLength::DISABLED)?;
        }
        self.set_pa_level(level)?;
        self.set_channel(channel)?;
        self._ce_pin.set_high().map_err(Nrf24Error::Gpo)?;
        if self._is_plus_variant {
            self._wait.delay_ms(1); // datasheet says 1 ms is ok in this instance
            self._ce_pin.set_low().map_err(Nrf24Error::Gpo)?;
            self.rewrite()?;
        }
        Ok(())
    }

    pub fn stop_carrier_wave(&mut self) -> Result<(), Nrf24Error<SPI::Error, DO::Error>> {
        /*
         * A note from the datasheet:
         * Do not use REUSE_TX_PL together with CONT_WAVE=1. When both these
         * registers are set the chip does not react when setting CE low. If
         * however, both registers are set PWR_UP = 0 will turn TX mode off.
         */
        self.power_down()?; // per datasheet recommendation (just to be safe)
        self.spi_read(1, registers::RF_SETUP)?;
        self.spi_write_byte(registers::RF_SETUP, self._buf[1] & !0x84)?;
        self._ce_pin.set_low().map_err(Nrf24Error::Gpo)
    }
}

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
        self._wait.delay_ms(5);

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

        self._wait.delay_us(self._tx_delay);
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
        self._wait.delay_us(10);
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
        // TODO: reversed byte order may need attention here
        self._buf[0] = commands::W_TX_PAYLOAD | ((ask_no_ack as u8) << 4);
        for i in buf_len..0 {
            self._buf[i + 1 - buf_len] = buf[i];
        }
        // ensure payload_length setting is respected
        if !self._dynamic_payloads_enabled && buf_len < self._payload_length as usize {
            // pad buf with zeros
            for i in self._payload_length as usize..buf_len {
                self._buf[i + 1 - self._payload_length as usize] = 0;
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
        self._wait.delay_us(10);
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
