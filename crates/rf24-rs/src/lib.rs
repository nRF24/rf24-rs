#![doc(
    html_logo_url = "https://raw.githubusercontent.com/nRF24/rf24-rs/main/docs/src/images/logo-square.png"
)]
#![doc(html_favicon_url = "https://github.com/nRF24/rf24-rs/raw/main/docs/src/images/favicon.ico")]
#![doc = include_str!("../README.md")]
//!
//! ## Basic API
//!
//! - [`RF24::new()`](fn@crate::radio::RF24::new)
//! - [`RF24::init()`](radio/struct.RF24.html#method.init)
//! - [`RF24::is_rx()`](radio/struct.RF24.html#method.is_rx)
//! - [`RF24::as_rx()`](radio/struct.RF24.html#method.as_rx)
//! - [`RF24::as_tx()`](radio/struct.RF24.html#method.as_tx)
//! - [`RF24::open_tx_pipe()`](radio/struct.RF24.html#method.open_tx_pipe)
//! - [`RF24::open_rx_pipe()`](radio/struct.RF24.html#method.open_rx_pipe)
//! - [`RF24::close_rx_pipe()`](radio/struct.RF24.html#method.close_rx_pipe)
//! - [`RF24::available()`](radio/struct.RF24.html#method.available)
//! - [`RF24::available_pipe()`](radio/struct.RF24.html#method.available_pipe)
//! - [`RF24::read()`](radio/struct.RF24.html#method.read)
//! - [`RF24::send()`](radio/struct.RF24.html#method.send)
//! - [`RF24::resend()`](radio/struct.RF24.html#method.resend)
//! - [`RF24::set_channel()`](radio/struct.RF24.html#method.set_channel)
//! - [`RF24::get_channel()`](radio/struct.RF24.html#method.get_channel)
//!
//! ## Advanced API
//!
//! - [`RF24::write_ack_payload()`](radio/struct.RF24.html#method.write_ack_payload)
//! - [`RF24::write()`](radio/struct.RF24.html#method.write)
//! - [`RF24::rewrite()`](radio/struct.RF24.html#method.rewrite)
//! - [`RF24::get_fifo_state()`](radio/struct.RF24.html#method.get_fifo_state)
//! - [`RF24::clear_status_flags()`](radio/struct.RF24.html#method.clear_status_flags)
//! - [`RF24::update()`](radio/struct.RF24.html#method.update)
//! - [`RF24::get_status_flags()`](radio/struct.RF24.html#method.get_status_flags)
//! - [`RF24::flush_rx()`](radio/struct.RF24.html#method.flush_rx)
//! - [`RF24::flush_tx()`](radio/struct.RF24.html#method.flush_tx)
//! - [`RF24::start_carrier_wave()`](fn@crate::radio::RF24::start_carrier_wave)
//! - [`RF24::stop_carrier_wave()`](fn@crate::radio::RF24::stop_carrier_wave)
//! - [`RF24::rpd()`](fn@crate::radio::RF24::rpd)
//! - [`RF24::get_last_arc()`](radio/struct.RF24.html#method.get_last_arc)
//! - [`RF24::get_dynamic_payload_length()`](radio/struct.RF24.html#method.get_dynamic_payload_length)
//!
//! ## Configuration API
//!
//! - [`RF24::with_config()`](radio/struct.RF24.html#method.with_config)
//! - [`RF24::set_status_flags()`](radio/struct.RF24.html#method.set_status_flags)
//! - [`RF24::set_auto_ack()`](radio/struct.RF24.html#method.set_auto_ack)
//! - [`RF24::set_auto_ack_pipe()`](radio/struct.RF24.html#method.set_auto_ack_pipe)
//! - [`RF24::set_auto_retries()`](radio/struct.RF24.html#method.set_auto_retries)
//! - [`RF24::set_dynamic_payloads()`](radio/struct.RF24.html#method.set_dynamic_payloads)
//! - [`RF24::allow_ask_no_ack()`](radio/struct.RF24.html#method.allow_ask_no_ack)
//! - [`RF24::allow_ack_payloads()`](radio/struct.RF24.html#method.set_ack_payloads)
//! - [`RF24::set_address_length()`](radio/struct.RF24.html#method.set_address_length)
//! - [`RF24::get_address_length()`](radio/struct.RF24.html#method.get_address_length)
//! - [`RF24::set_payload_length()`](radio/struct.RF24.html#method.set_payload_length)
//! - [`RF24::get_payload_length()`](radio/struct.RF24.html#method.get_payload_length)
//! - [`RF24::set_data_rate()`](radio/struct.RF24.html#method.set_data_rate)
//! - [`RF24::get_data_rate()`](radio/struct.RF24.html#method.get_data_rate)
//! - [`RF24::set_pa_level()`](radio/struct.RF24.html#method.set_pa_level)
//! - [`RF24::get_pa_level()`](radio/struct.RF24.html#method.get_pa_level)
//! - [`RF24::set_lna()`](fn@crate::radio::RF24::set_lna)
//! - [`RF24::set_crc_length()`](radio/struct.RF24.html#method.set_crc_length)
//! - [`RF24::get_crc_length()`](radio/struct.RF24.html#method.get_crc_length)
//! - [`RF24::is_powered()`](radio/struct.RF24.html#method.is_powered)
//! - [`RF24::power_up()`](radio/struct.RF24.html#method.power_up)
//! - [`RF24::power_down()`](radio/struct.RF24.html#method.power_down)
//! - [`RF24::tx_delay`](value@crate::radio::RF24::tx_delay)
//! - [`RF24::is_plus_variant()`](fn@crate::radio::RF24::is_plus_variant)
//!
#![no_std]

mod types;
pub use types::{CrcLength, DataRate, FifoState, PaLevel, StatusFlags};
pub mod radio;

#[cfg(test)]
mod test {
    use crate::radio::RF24;
    use embedded_hal_mock::eh1::{
        delay::NoopDelay,
        digital::{Mock as PinMock, Transaction as PinTransaction},
        spi::{Mock as SpiMock, Transaction as SpiTransaction},
    };

    /// Takes an indefinite repetition of a tuple of 2 vectors: `(expected_data, response_data)`
    /// and generates an array of `SpiTransaction`s.
    ///
    /// NOTE: This macro is only used to generate code in unit tests (for this crate only).
    #[macro_export]
    macro_rules! spi_test_expects {
        ($( ($expected:expr , $response:expr $(,)? ) , ) + ) => {
            [
                $(
                    SpiTransaction::transaction_start(),
                    SpiTransaction::transfer_in_place($expected, $response),
                    SpiTransaction::transaction_end(),
                )*
            ]
        }
    }

    /// A tuple struct to encapsulate objects used to mock [`RF24`],
    pub struct MockRadio(
        pub RF24<SpiMock<u8>, PinMock, NoopDelay>,
        pub SpiMock<u8>,
        pub PinMock,
    );

    /// Create a mock objects using the given expectations.
    ///
    /// The `spi_expectations` parameter
    pub fn mk_radio(
        ce_expectations: &[PinTransaction],
        spi_expectations: &[SpiTransaction<u8>],
    ) -> MockRadio {
        let spi = SpiMock::new(spi_expectations);
        let ce_pin = PinMock::new(ce_expectations);
        let delay_impl = NoopDelay;
        let radio = RF24::new(ce_pin.clone(), spi.clone(), delay_impl);
        MockRadio(radio, spi, ce_pin)
    }
}
