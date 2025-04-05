use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};
mod auto_ack;
pub(crate) mod bit_fields;
mod channel;
mod init;
use bit_fields::{Config, Feature};
mod constants;
mod crc_length;
mod data_rate;
mod fifo;
mod pa_level;
mod payload_length;
mod pipe;
mod power;
mod radio;
pub use constants::{commands, mnemonics, registers};
mod details;
mod status;
use super::prelude::{
    EsbAutoAck, EsbChannel, EsbCrcLength, EsbFifo, EsbPaLevel, EsbPower, EsbRadio,
};
use crate::{
    types::{CrcLength, PaLevel},
    StatusFlags,
};

/// An collection of error types to describe hardware malfunctions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Nrf24Error<SPI, DO> {
    /// Represents a SPI transaction error.
    Spi(SPI),
    /// Represents a DigitalOutput error.
    Gpo(DO),
    /// Represents a corruption of binary data (as it was transferred over the SPI bus' MISO)
    BinaryCorruption,
    /// An Error used to prevent an infinite loop in [`RF24::send()`].
    ///
    /// This only occurs when user code neglected to call [`RF24::as_tx()`] at least once
    /// before calling [`RF24::send()`].
    NotAsTxError,
}

/// This struct implements the [`Esb*` traits](mod@crate::radio::prelude)
/// for the nRF24L01 transceiver.
///
/// Additionally, there are some functions implemented that are specific to the nRF24L01.
pub struct RF24<SPI, DO, DELAY> {
    /// The delay (in microseconds) in which [`RF24::as_rx()`] will wait for
    /// ACK packets to complete.
    ///
    /// If the auto-ack feature is disabled, then this can be set as low as 0.
    /// If the auto-ack feature is enabled, then set to 100 microseconds minimum on
    /// generally faster devices (like RPi).
    ///
    /// Since this value can be optimized per the radio's data rate, this value is
    /// automatically adjusted when calling
    /// [`EsbDataRate::set_data_rate()`](fn@crate::radio::prelude::EsbDataRate::set_data_rate).
    /// If setting this to a custom value be sure, to set it *after*
    /// changing the radio's data rate.
    ///
    /// <div class="warning">
    ///
    /// If set to 0, ensure 130 microsecond delay
    /// after calling [`RF24::as_rx()`]
    /// and before transmitting.
    ///
    /// </div>
    ///
    pub tx_delay: u32,
    _spi: SPI,
    /// The CE pin for the radio.
    ///
    /// This really only exposed for advanced manipulation of active TX mode.
    /// It is strongly recommended to enter RX or TX mode using [`RF24::as_rx()`] and
    /// [`RF24::as_tx()`] because those methods guarantee proper radio usage.
    pub ce_pin: DO,
    _delay_impl: DELAY,
    _buf: [u8; 33],
    _status: StatusFlags,
    _config_reg: Config,
    _feature: Feature,
    _pipe0_rx_addr: Option<[u8; 5]>,
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
    /// when instantiating the [`SpiDevice`](trait@embedded-hal::spi::SpiDevice)
    /// object (passed to the `spi` parameter).
    pub fn new(ce_pin: DO, spi: SPI, delay_impl: DELAY) -> RF24<SPI, DO, DELAY> {
        RF24 {
            tx_delay: 250,
            ce_pin,
            _spi: spi,
            _delay_impl: delay_impl,
            _status: StatusFlags::from_bits(0),
            _buf: [0u8; 33],
            _pipe0_rx_addr: None,
            _feature: Feature::from_bits(0)
                .with_address_length(5)
                .with_is_plus_variant(true),
            // 16 bit CRC, enable all IRQ, and power down as TX
            _config_reg: Config::from_bits(0xC),
            _payload_length: 32,
        }
    }

    fn spi_transfer(&mut self, len: u8) -> Result<(), Nrf24Error<SPI::Error, DO::Error>> {
        self._spi
            .transfer_in_place(&mut self._buf[..len as usize])
            .map_err(Nrf24Error::Spi)?;
        self._status = StatusFlags::from_bits(self._buf[0]);
        Ok(())
    }

    /// This is also used to write SPI commands that consist of 1 byte:
    /// ```ignore
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
        self._buf[0] = command | commands::W_REGISTER;
        let buf_len = buf.len();
        self._buf[1..(buf_len + 1)].copy_from_slice(&buf[..buf_len]);
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
    /// The bool that this function returns is only valid _after_ calling
    /// [`init()`](fn@crate::radio::prelude::EsbInit::init).
    pub fn is_plus_variant(&self) -> bool {
        self._feature.is_plus_variant()
    }

    pub fn rpd(&mut self) -> Result<bool, Nrf24Error<SPI::Error, DO::Error>> {
        self.spi_read(1, registers::RPD)?;
        Ok(self._buf[1] & 1 == 1)
    }

    pub fn start_carrier_wave(
        &mut self,
        level: PaLevel,
        channel: u8,
    ) -> Result<(), Nrf24Error<SPI::Error, DO::Error>> {
        self.as_tx()?;
        self.spi_read(1, registers::RF_SETUP)?;
        self.spi_write_byte(registers::RF_SETUP, self._buf[1] | 0x90)?;
        if self._feature.is_plus_variant() {
            self.set_auto_ack(false)?;
            self.set_auto_retries(0, 0)?;
            let buf = [0xFF; 32];

            // use spi_write_buf() instead of open_tx_pipe() to bypass
            // truncation of the address with the current RF24::addr_width value
            self.spi_write_buf(registers::TX_ADDR, &buf[..5])?;
            self.flush_tx()?; // so we can write to top level

            self.spi_write_buf(commands::W_TX_PAYLOAD, &buf)?;

            self.set_crc_length(CrcLength::Disabled)?;
        }
        self.set_pa_level(level)?;
        self.set_channel(channel)?;
        self.ce_pin.set_high().map_err(Nrf24Error::Gpo)?;
        if self._feature.is_plus_variant() {
            self._delay_impl.delay_ns(1000000); // datasheet says 1 ms is ok in this instance
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
        self.spi_write_byte(registers::RF_SETUP, self._buf[1] & !0x90)?;
        self.ce_pin.set_low().map_err(Nrf24Error::Gpo)
    }

    /// Control the builtin LNA feature on nRF24L01 (older non-plus variants) and Si24R1
    /// (cheap chinese clones of the nRF24L01).
    ///
    /// This is enabled by default (regardless of chip variant).
    /// See [`PaLevel`] for effective behavior.
    ///
    /// This function has no effect on nRF24L01+ modules and PA/LNA variants because
    /// the LNA feature is always enabled.
    pub fn set_lna(&mut self, enable: bool) -> Result<(), Nrf24Error<SPI::Error, DO::Error>> {
        self.spi_read(1, registers::RF_SETUP)?;
        let out = self._buf[1] & !1 | enable as u8;
        self.spi_write_byte(registers::RF_SETUP, out)
    }
}

/////////////////////////////////////////////////////////////////////////////////
/// unit tests
#[cfg(test)]
mod test {
    extern crate std;
    use super::{commands, mnemonics, registers};
    use crate::{spi_test_expects, test::mk_radio};
    use embedded_hal_mock::eh1::{
        digital::{State as PinState, Transaction as PinTransaction},
        spi::Transaction as SpiTransaction,
    };
    use std::vec;

    #[test]
    pub fn test_rpd() {
        let spi_expectations = spi_test_expects![
            // get the RPD register value
            (vec![registers::RPD, 0u8], vec![0xEu8, 0xFFu8]),
        ];
        let mocks = mk_radio(&[], &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        assert!(radio.rpd().unwrap());
        spi.done();
        ce_pin.done();
    }

    pub fn start_carrier_wave_parametrized(is_plus_variant: bool) {
        let mut ce_expectations = [
            PinTransaction::set(PinState::Low),
            PinTransaction::set(PinState::High),
        ]
        .to_vec();
        if is_plus_variant {
            ce_expectations.extend([
                PinTransaction::set(PinState::Low),
                PinTransaction::set(PinState::High),
            ]);
        }

        let mut buf = [0xFFu8; 33];
        buf[0] = commands::W_TX_PAYLOAD;
        let mut address = [0xFFu8; 6];
        address[0] = registers::TX_ADDR | commands::W_REGISTER;

        let mut spi_expectations = spi_test_expects![
            // as_tx()
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
            // set special flags in RF_SETUP register value
            (vec![registers::RF_SETUP, 0u8], vec![0xEu8, 0u8]),
            (
                vec![registers::RF_SETUP | commands::W_REGISTER, 0x90u8],
                vec![0xEu8, 0u8],
            ),
        ]
        .to_vec();
        if is_plus_variant {
            spi_expectations.extend(spi_test_expects![
                // disable auto-ack
                (
                    vec![registers::EN_AA | commands::W_REGISTER, 0u8],
                    vec![0xEu8, 0u8],
                ),
                // disable auto-retries
                (
                    vec![registers::SETUP_RETR | commands::W_REGISTER, 0u8],
                    vec![0xEu8, 0u8],
                ),
                // set TX address
                (address.to_vec(), vec![0u8; 6]),
                // flush_tx()
                (vec![commands::FLUSH_TX], vec![0xEu8]),
                // set TX payload
                (buf.to_vec(), vec![0u8; 33]),
                // set_crc_length(disabled)
                (vec![registers::CONFIG, 0u8], vec![0xEu8, 0xCu8]),
                (
                    vec![registers::CONFIG | commands::W_REGISTER, 0u8],
                    vec![0xEu8, 0u8],
                ),
            ]);
        }
        spi_expectations.extend(spi_test_expects![
            // set_pa_level()
            // set special flags in RF_SETUP register value
            (vec![registers::RF_SETUP, 0u8], vec![0xEu8, 0x91u8]),
            (
                vec![registers::RF_SETUP | commands::W_REGISTER, 0x97u8],
                vec![0xEu8, 0u8],
            ),
            // set_channel(125)
            (
                vec![registers::RF_CH | commands::W_REGISTER, 125u8],
                vec![0xEu8, 0u8],
            ),
        ]);
        if is_plus_variant {
            spi_expectations.extend(spi_test_expects![
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
            ]);
        }

        let mocks = mk_radio(&ce_expectations, &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        radio._feature = radio._feature.with_is_plus_variant(is_plus_variant);
        radio.start_carrier_wave(crate::PaLevel::Max, 0xFF).unwrap();
        spi.done();
        ce_pin.done();
    }

    #[test]
    fn start_carrier_wave_plus_variant() {
        start_carrier_wave_parametrized(true);
    }

    #[test]
    fn start_carrier_wave_non_plus_variant() {
        start_carrier_wave_parametrized(false);
    }

    #[test]
    pub fn stop_carrier_wave() {
        let ce_expectations = [
            PinTransaction::set(PinState::Low),
            // CE is set LOW twice due to how it behaves during transmission of
            // constant carrier wave. See comment in start_carrier_wave()
            PinTransaction::set(PinState::Low),
        ];

        let spi_expectations = spi_test_expects![
            // power_down()
            (
                vec![registers::CONFIG | commands::W_REGISTER, 0xCu8],
                vec![0xEu8, 0u8],
            ),
            // clear special flags in RF_SETUP register
            (vec![registers::RF_SETUP, 0u8], vec![0xEu8, 0x90u8]),
            (
                vec![registers::RF_SETUP | commands::W_REGISTER, 0u8],
                vec![0xEu8, 0u8],
            ),
        ];
        let mocks = mk_radio(&ce_expectations, &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        radio.stop_carrier_wave().unwrap();
        spi.done();
        ce_pin.done();
    }

    #[test]
    pub fn set_lna() {
        let spi_expectations = spi_test_expects![
            // clear the LNA_CUR flag in RF-SETUP
            (vec![registers::RF_SETUP, 0u8], vec![0xEu8, 1u8]),
            (
                vec![registers::RF_SETUP | commands::W_REGISTER, 0u8],
                vec![0xEu8, 0u8],
            ),
        ];
        let mocks = mk_radio(&[], &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        radio.set_lna(false).unwrap();
        spi.done();
        ce_pin.done();
    }
}
