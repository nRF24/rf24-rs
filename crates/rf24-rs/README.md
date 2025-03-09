# rf24-rs

This crate is a rust driver library for the nRF24L01 wireless transceivers.

## Examples

Examples are located in the [rf24-rs repository](https://github.com/nRF24/rf24-rs/tree/main/examples/rust).

[rf24-struct]: struct@crate::radio::RF24

## Basic API

- [`RF24::new()`](fn@crate::radio::RF24::new)
- [`RF24`][rf24-struct]`::`[`init()`](fn@crate::radio::prelude::EsbInit::init)
- [`RF24`][rf24-struct]`::`[`is_rx()`](fn@crate::radio::prelude::EsbRadio::is_rx)
- [`RF24`][rf24-struct]`::`[`as_rx()`](fn@crate::radio::prelude::EsbRadio::as_rx)
- [`RF24`][rf24-struct]`::`[`as_tx()`](fn@crate::radio::prelude::EsbRadio::as_tx)
- [`RF24`][rf24-struct]`::`[`open_tx_pipe()`](fn@crate::radio::prelude::EsbPipe::open_tx_pipe)
- [`RF24`][rf24-struct]`::`[`open_rx_pipe()`](fn@crate::radio::prelude::EsbPipe::open_rx_pipe)
- [`RF24`][rf24-struct]`::`[`close_rx_pipe()`](fn@crate::radio::prelude::EsbPipe::close_rx_pipe)
- [`RF24`][rf24-struct]`::`[`available()`](fn@crate::radio::prelude::EsbFifo::available)
- [`RF24`][rf24-struct]`::`[`available_pipe()`](fn@crate::radio::prelude::EsbFifo::available_pipe)
- [`RF24`][rf24-struct]`::`[`read()`](fn@crate::radio::prelude::EsbRadio::read)
- [`RF24`][rf24-struct]`::`[`send()`](fn@crate::radio::prelude::EsbRadio::send)
- [`RF24`][rf24-struct]`::`[`resend()`](fn@crate::radio::prelude::EsbRadio::resend)
- [`RF24`][rf24-struct]`::`[`set_channel()`](fn@crate::radio::prelude::EsbChannel::set_channel)
- [`RF24`][rf24-struct]`::`[`get_channel()`](fn@crate::radio::prelude::EsbChannel::get_channel)

## Advanced API

- [`RF24`][rf24-struct]`::`[`write_ack_payload()`](fn@crate::radio::prelude::EsbAutoAck::write_ack_payload)
- [`RF24`][rf24-struct]`::`[`write()`](fn@crate::radio::prelude::EsbRadio::write)
- [`RF24`][rf24-struct]`::`[`rewrite()`](fn@crate::radio::prelude::EsbRadio::rewrite)
- [`RF24`][rf24-struct]`::`[`get_fifo_state()`](fn@crate::radio::prelude::EsbFifo::get_fifo_state)
- [`RF24`][rf24-struct]`::`[`clear_status_flags()`](fn@crate::radio::prelude::EsbStatus::clear_status_flags)
- [`RF24`][rf24-struct]`::`[`update()`](fn@crate::radio::prelude::EsbStatus::update)
- [`RF24`][rf24-struct]`::`[`get_status_flags()`](fn@crate::radio::prelude::EsbStatus::get_status_flags)
- [`RF24`][rf24-struct]`::`[`flush_rx()`](fn@crate::radio::prelude::EsbFifo::flush_rx)
- [`RF24`][rf24-struct]`::`[`flush_tx()`](fn@crate::radio::prelude::EsbFifo::flush_tx)
- [`RF24::start_carrier_wave()`](fn@crate::radio::RF24::start_carrier_wave)
- [`RF24::stop_carrier_wave()`](fn@crate::radio::RF24::stop_carrier_wave)
- [`RF24::rpd()`](fn@crate::radio::RF24::rpd)
- [`RF24`][rf24-struct]`::`[`get_last_arc()`](fn@crate::radio::prelude::EsbRadio::get_last_arc)
- [`RF24`][rf24-struct]`::`[`get_dynamic_payload_length()`](fn@crate::radio::prelude::EsbPayloadLength::get_dynamic_payload_length)

## Configuration API

- [`RF24`][rf24-struct]`::`[`set_status_flags()`](fn@crate::radio::prelude::EsbStatus::set_status_flags)
- [`RF24`][rf24-struct]`::`[`set_auto_ack()`](fn@crate::radio::prelude::EsbAutoAck::set_auto_ack)
- [`RF24`][rf24-struct]`::`[`set_auto_ack_pipe()`](fn@crate::radio::prelude::EsbAutoAck::set_auto_ack_pipe)
- [`RF24`][rf24-struct]`::`[`set_auto_retries()`](fn@crate::radio::prelude::EsbAutoAck::set_auto_retries)
- [`RF24`][rf24-struct]`::`[`set_dynamic_payloads()`](fn@crate::radio::prelude::EsbPayloadLength::set_dynamic_payloads)
- [`RF24`][rf24-struct]`::`[`allow_ask_no_ack()`](fn@crate::radio::prelude::EsbAutoAck::allow_ask_no_ack)
- [`RF24`][rf24-struct]`::`[`allow_ack_payloads()`](fn@crate::radio::prelude::EsbAutoAck::set_ack_payloads)
- [`RF24`][rf24-struct]`::`[`set_address_length()`](fn@crate::radio::prelude::EsbPipe::set_address_length)
- [`RF24`][rf24-struct]`::`[`get_address_length()`](fn@crate::radio::prelude::EsbPipe::get_address_length)
- [`RF24`][rf24-struct]`::`[`set_payload_length()`](fn@crate::radio::prelude::EsbPayloadLength::set_payload_length)
- [`RF24`][rf24-struct]`::`[`get_payload_length()`](fn@crate::radio::prelude::EsbPayloadLength::get_payload_length)
- [`RF24`][rf24-struct]`::`[`set_data_rate()`](fn@crate::radio::prelude::EsbDataRate::set_data_rate)
- [`RF24`][rf24-struct]`::`[`get_data_rate()`](fn@crate::radio::prelude::EsbDataRate::get_data_rate)
- [`RF24`][rf24-struct]`::`[`set_pa_level()`](fn@crate::radio::prelude::EsbPaLevel::set_pa_level)
- [`RF24`][rf24-struct]`::`[`get_pa_level()`](fn@crate::radio::prelude::EsbPaLevel::get_pa_level)
- [`RF24::set_lna()`](fn@crate::radio::RF24::set_lna)
- [`RF24`][rf24-struct]`::`[`set_crc_length()`](fn@crate::radio::prelude::EsbCrcLength::set_crc_length)
- [`RF24`][rf24-struct]`::`[`get_crc_length()`](fn@crate::radio::prelude::EsbCrcLength::get_crc_length)
- [`RF24`][rf24-struct]`::`[`is_powered()`](fn@crate::radio::prelude::EsbPower::is_powered)
- [`RF24`][rf24-struct]`::`[`power_up()`](fn@crate::radio::prelude::EsbPower::power_up)
- [`RF24`][rf24-struct]`::`[`power_down()`](fn@crate::radio::prelude::EsbPower::power_down)
- [`RF24::tx_delay`](value@crate::radio::RF24::tx_delay)
- [`RF24::is_plus_variant()`](fn@crate::radio::RF24::is_plus_variant)
