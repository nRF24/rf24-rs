use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};
mod auto_ack;
mod channel;
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
mod status;
use super::prelude::{
    EsbAutoAck, EsbChannel, EsbCrcLength, EsbFifo, EsbPaLevel, EsbPower, EsbRadio,
};
use crate::enums::{CrcLength, PaLevel};

/// An collection of error types to describe hardware malfunctions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Nrf24Error<SPI, DO> {
    /// Represents a SPI transaction error.
    Spi(SPI),
    /// Represents a DigitalOutput error.
    Gpo(DO),
    /// Represents a corruption of binary data (as it was transferred over the SPI bus' MISO)
    BinaryCorruption,
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
    _delay_impl: DELAY,
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
            _delay_impl: delay_impl,
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
        for i in 0..buf_len {
            self._buf[i + 1] = buf[i];
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
        self.spi_write_byte(registers::RF_SETUP, self._buf[1] | 0x90)?;
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
            self._delay_impl.delay_ms(1); // datasheet says 1 ms is ok in this instance
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
        self._ce_pin.set_low().map_err(Nrf24Error::Gpo)
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
    use super::{commands, registers, RF24};
    use crate::radio::rf24::mnemonics;
    use embedded_hal_mock::eh1::delay::NoopDelay;
    use embedded_hal_mock::eh1::digital::{
        Mock as PinMock, State as PinState, Transaction as PinTransaction,
    };
    use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
    use std::vec;

    #[test]
    pub fn test_rpd() {
        // Create pin
        let pin_expectations = [];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

        let spi_expectations = [
            // get the RPD register value
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::RPD, 0u8], vec![0xEu8, 0xFFu8]),
            SpiTransaction::transaction_end(),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        assert!(radio.test_rpd().unwrap());
        spi_mock.done();
        pin_mock.done();
    }

    #[test]
    pub fn start_carrier_wave() {
        // Create pin
        let pin_expectations = [
            PinTransaction::set(PinState::Low),
            PinTransaction::set(PinState::High),
            PinTransaction::set(PinState::Low),
            PinTransaction::set(PinState::High),
        ];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

        let mut buf = [0xFFu8; 33];
        buf[0] = commands::W_TX_PAYLOAD;
        let mut address = [0xFFu8; 6];
        address[0] = registers::TX_ADDR | commands::W_REGISTER;

        let spi_expectations = [
            // stop_listening()
            // clear PRIM_RX flag
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::CONFIG | commands::W_REGISTER, 0u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
            // open pipe 0 for TX (regardless of auto-ack)
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::EN_RXADDR, 0u8], vec![0xEu8, 0u8]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::EN_RXADDR | commands::W_REGISTER, 1u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
            // set special flags in RF_SETUP register value
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::RF_SETUP, 0u8], vec![0xEu8, 0u8]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::RF_SETUP | commands::W_REGISTER, 0x90u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
            // disable auto-ack
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::EN_AA | commands::W_REGISTER, 0u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
            // disable auto-retries
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::SETUP_RETR | commands::W_REGISTER, 0u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
            // set TX address
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(address.to_vec(), vec![0u8; 6]),
            SpiTransaction::transaction_end(),
            // flush_tx()
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![commands::FLUSH_TX], vec![0xEu8]),
            SpiTransaction::transaction_end(),
            // set TX payload
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(buf.to_vec(), vec![0u8; 33]),
            SpiTransaction::transaction_end(),
            // set_crc_length(disabled)
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::CONFIG, 0u8], vec![0xEu8, 0xCu8]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::CONFIG | commands::W_REGISTER, 0u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
            // set_pa_level()
            // set special flags in RF_SETUP register value
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::RF_SETUP, 0u8], vec![0xEu8, 0x91u8]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::RF_SETUP | commands::W_REGISTER, 0x97u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
            // set_channel(125)
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::RF_CH | commands::W_REGISTER, 125u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
            // clear the tx_df and tx_ds events
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![
                    registers::STATUS | commands::W_REGISTER,
                    mnemonics::MASK_MAX_RT | mnemonics::MASK_TX_DS,
                ],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
            // assert the REUSE_TX_PL flag
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![commands::REUSE_TX_PL], vec![0xEu8]),
            SpiTransaction::transaction_end(),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        radio.start_carrier_wave(crate::PaLevel::MAX, 0xFF).unwrap();
        spi_mock.done();
        pin_mock.done();
    }

    #[test]
    pub fn stop_carrier_wave() {
        // Create pin
        let pin_expectations = [
            PinTransaction::set(PinState::Low),
            // CE is set LOW twice due to how it behaves during transmission of
            // constant carrier wave. See comment in start_carrier_wave()
            PinTransaction::set(PinState::Low),
        ];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

        let spi_expectations = [
            // power_down()
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::CONFIG | commands::W_REGISTER, 0u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
            // clear special flags in RF_SETUP register
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::RF_SETUP, 0u8], vec![0xEu8, 0x90u8]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::RF_SETUP | commands::W_REGISTER, 0u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        radio.stop_carrier_wave().unwrap();
        spi_mock.done();
        pin_mock.done();
    }

    #[test]
    pub fn set_lna() {
        // Create pin
        let pin_expectations = [];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

        let spi_expectations = [
            // clear the LNA_CUR flag in RF-SETUP
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::RF_SETUP, 0u8], vec![0xEu8, 1u8]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::RF_SETUP | commands::W_REGISTER, 0u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        radio.set_lna(false).unwrap();
        spi_mock.done();
        pin_mock.done();
    }
}
