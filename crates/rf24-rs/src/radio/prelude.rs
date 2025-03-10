//! This module defines the generic traits that may
//! need to imported to use radio implementations.
//!
//! Since rustc only compiles objects that are used,
//! it is convenient to import these traits with the `*` syntax.
//!
//! ```
//! use rf24::radio::prelude::*;
//! ```

use crate::types::{CrcLength, DataRate, FifoState, PaLevel, StatusFlags};

use super::RadioConfig;

/// A trait to represent manipulation of data pipes
/// for an ESB capable transceiver.
pub trait EsbPipe {
    type PipeErrorType;

    /// Open a specified `pipe` for receiving data when radio is in RX mode.
    ///
    /// If the specified `pipe` is not in range [0, 5], then this function does nothing.
    ///
    /// Up to 6 pipes can be open for reading at once.  Open all the required
    /// reading pipes, and then call [`EsbRadio::as_rx()`].
    ///
    /// ### About pipe addresses
    /// Pipes 0 and 1 will store a full 5-byte address. Pipes 2-5 will technically
    /// only store a single byte, borrowing up to 4 additional bytes from pipe 1 per
    /// [`EsbPipe::set_address_length()`].
    ///
    /// Pipes 1-5 should share the same address, except the first byte.
    /// Only the first byte in the array should be unique, e.g.
    /// ```ignore
    /// let a = ["Prime", "2Node", "3xxxx", "4xxxx"];
    /// radio.open_rx_pipe(0, a[0].as_bytes()).unwrap(); // address used is "Prime"
    /// radio.open_rx_pipe(1, a[1].as_bytes()).unwrap(); // address used is "2Node"
    /// radio.open_rx_pipe(2, a[2].as_bytes()).unwrap(); // address used is "3Node"
    /// radio.open_rx_pipe(3, a[3].as_bytes()).unwrap(); // address used is "4Node"
    /// ```
    ///
    /// <div class="warning">
    ///
    /// If the pipe 0 is opened for receiving by this function, the `address`
    /// passed to this function (for pipe 0) will be restored at every call to
    /// [`EsbRadio::as_rx()`].
    /// This address restoration is implemented because of the underlying necessary
    /// functionality of [`EsbPipe::open_tx_pipe()`].
    ///
    /// It is important that the `address` length for pipe 0
    /// is equal to the length configured by [`EsbPipe::set_address_length()`].
    ///
    /// </div>
    ///
    /// Read [maniacBug's blog post](http://maniacalbits.blogspot.com/2013/04/rf24-addressing-nrf24l01-radios-require.html)
    /// to understand how to avoid using malformed addresses.
    fn open_rx_pipe(&mut self, pipe: u8, address: &[u8]) -> Result<(), Self::PipeErrorType>;

    /// Set an address to pipe 0 for transmitting when radio is in TX mode.
    ///
    fn open_tx_pipe(&mut self, address: &[u8]) -> Result<(), Self::PipeErrorType>;

    /// Close a specified pipe from receiving data when radio is in RX mode.
    fn close_rx_pipe(&mut self, pipe: u8) -> Result<(), Self::PipeErrorType>;

    /// Set the address length (applies to all pipes).
    ///
    /// If the specified length is clamped to the range [2, 5].
    /// Any value outside that range defaults to 5.
    fn set_address_length(&mut self, length: u8) -> Result<(), Self::PipeErrorType>;

    /// Get the currently configured address length (applied to all pipes).
    fn get_address_length(&mut self) -> Result<u8, Self::PipeErrorType>;
}

/// A trait to represent manipulation of a channel (aka frequency)
/// for an ESB capable transceiver.
pub trait EsbChannel {
    type ChannelErrorType;

    /// Set the radio's currently selected channel.
    ///
    /// These channels translate to the RF frequency as an offset of Hz from 2400 MHz.
    /// The default channel is 76 (2400 + 76 = 2.476 GHz).
    fn set_channel(&mut self, channel: u8) -> Result<(), Self::ChannelErrorType>;

    /// Get the radio's currently selected channel.
    fn get_channel(&mut self) -> Result<u8, Self::ChannelErrorType>;
}

/// A trait to represent manipulation of [`StatusFlags`]
/// for an ESB capable transceiver.
pub trait EsbStatus {
    type StatusErrorType;

    /// Get the [`StatusFlags`] state that was cached from the latest SPI transaction.
    fn get_status_flags(&self, flags: &mut StatusFlags);

    /// Configure which status flags trigger the radio's IRQ pin.
    ///
    /// Set any member of [`StatusFlags`] to `false` to have the
    /// IRQ pin ignore the corresponding event.
    /// By default, all events are enabled and will trigger the IRQ pin,
    /// a behavior equivalent to `set_status_flags(None)`.
    fn set_status_flags(&mut self, flags: StatusFlags) -> Result<(), Self::StatusErrorType>;

    /// Clear the radio's IRQ status flags
    ///
    /// This needs to be done after the event has been handled.
    ///
    /// Set any member of [`StatusFlags`] to `true` to clear the corresponding
    /// interrupt event. Setting any member of [`StatusFlags`] to `false` will leave
    /// the corresponding status flag untouched. This means that the IRQ pin can remain
    /// active (LOW) when multiple events occurred but only flag was cleared.
    fn clear_status_flags(&mut self, flags: StatusFlags) -> Result<(), Self::StatusErrorType>;

    /// Refresh the internal cache of status byte
    /// (which is also saved from every SPI transaction).
    ///
    /// Use [`EsbStatus::get_status_flags()`] to get the updated status flags.
    fn update(&mut self) -> Result<(), Self::StatusErrorType>;
}

/// A trait to represent manipulation of RX and TX FIFOs
/// for an ESB capable transceiver.
pub trait EsbFifo {
    type FifoErrorType;

    /// Flush the radio's RX FIFO.
    fn flush_rx(&mut self) -> Result<(), Self::FifoErrorType>;

    /// Flush the radio's TX FIFO.
    ///
    /// This function is automatically called by [`EsbRadio::as_tx()`]
    /// if ACK payloads are enabled.
    fn flush_tx(&mut self) -> Result<(), Self::FifoErrorType>;

    /// Get the state of the specified FIFO.
    ///
    /// - Pass `true` to `about_tx` parameter to get the state of the TX FIFO.
    /// - Pass `false` to `about_tx` parameter to get the state of the RX FIFO.
    fn get_fifo_state(&mut self, about_tx: bool) -> Result<FifoState, Self::FifoErrorType>;

    /// Is there a payload available in the radio's RX FIFO?
    ///
    /// This function simply returns true if there is data to [`EsbRadio::read()`] from the RX FIFO.
    /// Use [`EsbFifo::available_pipe()`] to get information about the pipe that received the data.
    fn available(&mut self) -> Result<bool, Self::FifoErrorType>;

    /// This is similar to [`EsbFifo::available()`] except the `pipe` parameter is given
    /// a mutable [`u8`] value, and the pipe number that received the data is stored to it.
    ///
    /// If there is no data ready to [`EsbRadio::read()`] in the RX FIFO, then the `pipe` parameter's
    /// value is untouched.
    ///
    /// ```ignore
    /// let mut pipe = 9; // using an invalid pipe number
    /// if radio.available_pipe(&mut pipe).is_ok_and(|rv| rv) {
    ///     // `pipe` should now be set to a valid pipe number
    ///     print!("A Payload was received on pipe {pipe}");
    /// }
    /// ```
    ///
    /// <div class="warning">
    ///
    /// According to the nRF24L01 datasheet, the data saved to `pipe` is
    /// "unreliable" during a FALLING transition on the IRQ pin.
    ///
    /// During an ISR (Interrupt Service Routine), call
    /// [`EsbStatus::get_status_flags()`] and/or [`EsbStatus::clear_status_flags()`]
    /// before calling this function.
    ///
    /// </div>
    fn available_pipe(&mut self, pipe: &mut u8) -> Result<bool, Self::FifoErrorType>;
}

/// A trait to represent manipulation of payload lengths (static or dynamic)
/// for an ESB capable transceiver.
pub trait EsbPayloadLength {
    type PayloadLengthErrorType;

    /// Set the radio's static payload length.
    ///
    /// Note, this has no effect when dynamic payloads are enabled.
    fn set_payload_length(&mut self, length: u8) -> Result<(), Self::PayloadLengthErrorType>;

    /// Get the currently configured static payload length used on pipe 0
    ///
    /// Use [`EsbPayloadLength::get_dynamic_payload_length()`] instead when dynamic payloads are enabled.
    fn get_payload_length(&mut self) -> Result<u8, Self::PayloadLengthErrorType>;

    /// Set the dynamic payloads feature for all pipes.
    ///
    /// Dynamic payloads are required to use ACK packets with payloads appended.
    ///
    /// Enabling dynamic payloads nullifies the effect of
    /// [`EsbPayloadLength::set_payload_length()`] and
    /// [`EsbPayloadLength::get_payload_length()`].
    /// Use [`EsbPayloadLength::get_dynamic_payload_length()`] to
    /// fetch the length of the next [`EsbFifo::available()`] payload in the RX FIFO.
    ///
    /// ```ignore
    /// radio.set_dynamic_payloads(true).unwrap();
    /// // ... then after or during RX mode:
    /// if radio.available().unwrap() {
    ///     let length = radio.get_dynamic_payload_length().unwrap();
    ///     let mut payload = [0; 32];
    ///     radio.read(&mut payload, length).unwrap();
    ///     // do something with the new payload data:
    ///     for byte in payload[..length as usize] {
    ///         print!("{:#02x} ", byte);
    ///     }
    /// }
    /// ```
    fn set_dynamic_payloads(&mut self, enable: bool) -> Result<(), Self::PayloadLengthErrorType>;

    /// Get the current setting of the dynamic payloads feature.
    ///
    /// Controlled by [`EsbPayloadLength::set_dynamic_payloads()`].
    fn get_dynamic_payloads(&self) -> bool;

    /// Get the dynamic length of the next available payload in the RX FIFO.
    ///
    /// When dynamic payloads are disabled (via [`EsbPayloadLength::set_dynamic_payloads()`])
    /// or there is no [`EsbFifo::available()`] payload in the RX FIFO, this function's
    /// returned value shall be considered invalid.
    fn get_dynamic_payload_length(&mut self) -> Result<u8, Self::PayloadLengthErrorType>;
}

/// A trait to represent manipulation of the automatic acknowledgement feature
/// for an ESB capable transceiver.
pub trait EsbAutoAck: EsbPayloadLength {
    type AutoAckErrorType;

    /// Enable or disable the custom ACK (acknowledgement) payloads attached to auto-ack packets.
    ///
    /// By default this feature is disabled.
    /// Using payloads in the auto-ack packets requires enabling dynamic payloads feature
    /// This function will only ensure dynamic payloads are enabled on pipes 0 and 1.
    /// Use [`EsbPayloadLength::set_dynamic_payloads()`] to enable dynamic payloads on all pipes.
    ///
    /// Use [`EsbFifo::available()`] to see if there any ACK payloads in the RX FIFO.
    /// Use [`EsbRadio::read()`] to fetch the payloads from the RX FIFO.
    ///
    /// To append a payload to an auto ack packet, use [`EsbAutoAck::write_ack_payload()`].
    fn set_ack_payloads(&mut self, enable: bool) -> Result<(), Self::AutoAckErrorType>;

    /// Get the current setting of the ACK payloads feature.
    ///
    /// Controlled with [`EsbAutoAck::set_ack_payloads()`].
    fn get_ack_payloads(&self) -> bool;

    /// Write a `buf` to the radio's TX FIFO for use with automatic ACK packets.
    ///
    /// The given `buf` will be the outgoing payload added to an automatic ACK packet
    /// when acknowledging an incoming payload that was received with the specified
    /// `pipe`. This function returns `true` if the payload was written to the TX FIFO.
    /// It returns `false` under the following conditions:
    ///
    /// - the radio's TX FIFO is full.
    /// - the specified `pipe` number is invalid (not in range [0, 5]).
    /// - the ACK payload feature is not enabled (see [`EsbAutoAck::set_ack_payloads()`])
    ///
    /// It is important to discard any non-ACK payloads in the TX FIFO (using
    /// [`EsbFifo::flush_tx()`]) before writing the first ACK payload into the TX FIFO.
    /// This function can be used before and/or after calling [`EsbRadio::as_rx()`].
    ///
    /// <div class="warning">
    ///
    /// The payload must be loaded into the radio's TX FIFO _before_ the incoming
    /// payload is received.
    ///
    /// Remember, the TX FIFO can only store a maximum of 3 payloads,
    /// and there are typically more pipes than TX FIFO occupancy.
    /// Expected behavior is better assured when the ACK payloads are only used
    /// for 1 pipe.
    ///
    /// </div>
    ///
    /// Since ACK payloads require the dynamic payloads feature enabled, the given
    /// `buf`'s length will determine the length of the payload in the ACK packet.
    ///
    /// See also [`EsbAutoAck::set_ack_payloads()`],
    /// [`EsbPayloadLength::set_dynamic_payloads`], and [`EsbAutoAck::set_auto_ack()`].
    fn write_ack_payload(&mut self, pipe: u8, buf: &[u8]) -> Result<bool, Self::AutoAckErrorType>;

    /// Enable or disable the auto-ack (automatic acknowledgement) feature for all
    /// pipes.
    ///
    /// This feature is enabled by default. The auto-ack feature responds to every
    /// received payload with an ACK packet. These ACK packets get sent
    /// from the receiving radio back to the transmitting radio. To attach an
    /// ACK payload to a ACK packet, use [`EsbAutoAck::write_ack_payload()`]`.
    ///
    /// If this feature is disabled on a transmitting radio, then the
    /// transmitting radio will always report that the payload was received
    /// (even if it was not). Please remember that this feature's configuration
    /// needs to match for transmitting and receiving radios.
    ///
    /// When using the `ask_no_ack` parameter to [`EsbRadio::send()`] and [`EsbRadio::write()`],
    /// this feature can be disabled for an individual payload. However, if this feature
    /// is disabled, then the `ask_no_ack` parameter will have no effect.
    ///
    /// If disabling auto-acknowledgment packets, the ACK payloads
    /// feature is also disabled as this feature is required to send ACK
    /// payloads.
    fn set_auto_ack(&mut self, enable: bool) -> Result<(), Self::AutoAckErrorType>;

    /// Set the auto-ack feature for an individual `pipe`.
    ///
    /// Pipe 0 is used for TX operations, which include sending ACK packets. If
    /// using this feature on both TX & RX nodes, then pipe 0 must have this
    /// feature enabled for the RX & TX operations. If this feature is disabled
    /// on a transmitting radio's pipe 0, then the transmitting radio will
    /// always report that the payload was received (even if it was not).
    /// Remember to also enable this feature for any pipe that is openly
    /// listening to a transmitting radio with this feature enabled.
    ///
    /// If this feature is enabled for pipe 0, then the `ask_no_ack` parameter to
    /// [`EsbRadio::send()`] and [`EsbRadio::write()`] can be used to disable this feature for
    /// an individual payload. However, if this feature is disabled for pipe 0,
    /// then the `ask_no_ack` parameter will have no effect.
    ///
    /// If disabling auto-acknowledgment packets on pipe 0, the ACK
    /// payloads feature is also disabled as this feature is required on pipe 0
    /// to send ACK payloads.
    fn set_auto_ack_pipe(&mut self, enable: bool, pipe: u8) -> Result<(), Self::AutoAckErrorType>;

    /// Set the number of retry attempts and delay between retry attempts when
    /// transmitting a payload.
    ///
    /// When the auto-ack feature is enabled (via [`EsbAutoAck::set_auto_ack()`]),
    /// the radio waits for an acknowledgement (ACK) packet during the `delay` between retry
    /// attempts (`count`).
    ///
    /// Both parameters are clamped to range [0, 15].
    /// - `delay`: How long to wait between each retry, in multiples of
    ///   250 us (microseconds). The minimum value of 0 means 250 us, and
    ///   the maximum valueof 15 means 4000 us.
    ///   The default value of 5 means 1500us (`5 * 250 + 250`).
    /// - `count`: How many retries before giving up. The default/maximum is 15. Use
    ///   0 to disable the auto-retry feature.
    ///
    /// Disabling the auto-retry feature on a transmitter still uses the
    /// auto-ack feature (if enabled), except it will not retry to transmit if
    /// the payload was not acknowledged on the first attempt.
    fn set_auto_retries(&mut self, delay: u8, count: u8) -> Result<(), Self::AutoAckErrorType>;

    /// Allow the functionality of the `ask_no_ack` parameter in [`EsbRadio::send()`] and
    /// [`EsbRadio::write()`].
    ///
    /// This only needs to called once before using the `ask_no_ack` parameter in
    /// [`EsbRadio::send()`] and [`EsbRadio::write()`]. Enabling this feature will basically
    /// allow disabling the auto-ack feature on a per-payload basis. Such behavior would be
    /// desirable when transmitting to multiple radios that are setup to receive data from the
    /// same address.
    fn allow_ask_no_ack(&mut self, enable: bool) -> Result<(), Self::AutoAckErrorType>;
}

/// A trait to represent manipulation of the power amplitude level
/// for an ESB capable transceiver.
pub trait EsbPaLevel {
    type PaLevelErrorType;

    /// Get the currently configured Power Amplitude Level (PA Level)
    fn get_pa_level(&mut self) -> Result<PaLevel, Self::PaLevelErrorType>;

    /// Set the radio's Power Amplitude Level (PA Level)
    fn set_pa_level(&mut self, pa_level: PaLevel) -> Result<(), Self::PaLevelErrorType>;
}

/// A trait to represent manipulation of the state of power
/// for an ESB capable transceiver.
pub trait EsbPower {
    type PowerErrorType;

    /// Power down the radio.
    ///
    /// <div class="warning">
    ///
    /// The nRF24L01 cannot receive nor transmit data when powered down.
    ///
    /// </div>
    fn power_down(&mut self) -> Result<(), Self::PowerErrorType>;

    /// Power up the radio.
    ///
    /// This wakes the radio from a sleep state, resulting in a
    /// power standby mode that allows the radio to receive or transmit data.
    ///
    /// To ensure proper operation, this function will `delay` after the radio is powered up.
    /// If the `delay` parameter is given a [`Some`] value, then the this function
    /// will wait for the specified number of microseconds. If `delay` is a [`None`]
    /// value, this function will wait for 5 milliseconds.
    ///
    /// To perform other tasks while the radio is powering up:
    /// ```ignore
    /// radio.power_up(Some(0)).unwrap();
    /// // ... do something else for 5 milliseconds
    /// radio.as_rx().unwrap();
    /// ```
    fn power_up(&mut self, delay: Option<u32>) -> Result<(), Self::PowerErrorType>;

    /// Get the current (cached) state of the radio's power.
    ///
    /// Returns `true` if powered up or `false` if powered down.
    fn is_powered(&self) -> bool;
}

/// A trait to represent manipulation of Cyclical Redundancy Checksums
/// for an ESB capable transceiver.
pub trait EsbCrcLength {
    type CrcLengthErrorType;

    /// Get the currently configured CRC (Cyclical Redundancy Checksum) length
    fn get_crc_length(&mut self) -> Result<CrcLength, Self::CrcLengthErrorType>;

    /// Set the radio's CRC (Cyclical Redundancy Checksum) length
    fn set_crc_length(&mut self, crc_length: CrcLength) -> Result<(), Self::CrcLengthErrorType>;
}

/// A trait to represent manipulation of the Data Rate
/// for an ESB capable transceiver.
pub trait EsbDataRate {
    type DataRateErrorType;

    /// Get the currently configured Data Rate
    fn get_data_rate(&mut self) -> Result<DataRate, Self::DataRateErrorType>;

    /// Set the radio's Data Rate
    fn set_data_rate(&mut self, data_rate: DataRate) -> Result<(), Self::DataRateErrorType>;
}

/// A trait to represent debug output
/// for an ESB capable transceiver.
pub trait EsbDetails {
    type DetailsErrorType;

    /// Print details about radio's current configuration.
    ///
    /// This should only be used for debugging development.
    /// Using this in production should be limited due to a significant increase in
    /// compile size.
    fn print_details(&mut self) -> Result<(), Self::DetailsErrorType>;
}

pub trait EsbInit {
    type ConfigErrorType;

    /// Initialize the radio's hardware.
    ///
    /// This is similar to [`EsbInit::with_config()`] (with [`RadioConfig::default()`]),
    /// but this function also
    ///
    /// - waits 5 milliseconds for radio to finish powering up
    /// - tests if radio module is compatible with generic nRF24L01+ variant
    /// - checks if radio has responding correctly after configuration
    ///
    /// This function should only be called once after instantiating the radio object.
    /// Afterward, it is quicker to use [`EsbInit::with_config()`] to reconfigure the
    /// radio for different network requirements.
    fn init(&mut self) -> Result<(), Self::ConfigErrorType>;

    /// Reconfigure the radio using the given `config` object.
    ///
    /// See [`RadioConfig`] for more detail.
    /// This function is a convenience where calling multiple configuration functions may
    /// be cumbersome.
    fn with_config(&mut self, config: &RadioConfig) -> Result<(), Self::ConfigErrorType>;
}

/// A trait to represent manipulation of an ESB capable transceiver.
///
/// Although the name is rather generic, this trait describes the
/// behavior of a radio's rudimentary modes (RX and TX).
pub trait EsbRadio {
    type RadioErrorType;

    /// Put the radio into active RX mode.
    ///
    /// Conventionally, this should be called after setting the RX addresses via
    /// [`EsbPipe::open_rx_pipe()`]
    fn as_rx(&mut self) -> Result<(), Self::RadioErrorType>;

    /// Put the radio into inactive TX mode.
    ///
    /// This must be called at least once before calling [`EsbRadio::send()`] or
    /// [`EsbRadio::write()`].
    /// Conventionally, this should be called after setting the TX address via
    /// [`EsbPipe::open_tx_pipe()`].
    fn as_tx(&mut self) -> Result<(), Self::RadioErrorType>;

    /// Is the radio in RX mode?
    fn is_rx(&self) -> bool;

    /// Blocking function to transmit a given payload.
    ///
    /// This transmits a payload (given by `buf`) and returns a bool describing if
    /// the transmission was successful or not.
    ///
    /// See [`EsbRadio::write()`] for description of `ask_no_ack` parameter and more
    /// detail about how the radio processes data in the TX FIFO.
    fn send(&mut self, buf: &[u8], ask_no_ack: bool) -> Result<bool, Self::RadioErrorType>;

    /// Non-blocking function to prepare radio for transmitting payload(s).
    ///
    /// This is a helper function to [`EsbRadio::send()`].
    ///
    /// Unlike [`EsbRadio::send()`], this function does not wait for the radio to complete
    /// the transmission. Instead it simply writes the given `buf` into the radio's TX FIFO.
    /// If the TX FIFO is already full, this function just calls
    /// [`EsbStatus::clear_status_flags()`] (only for `tx_df` and `tx_ds` flags) and returns
    /// `false`.
    ///
    /// If `ask_no_ack` is true, then the transmitted payload will not use the auto-ack
    /// feature. This parameter is different from [`EsbAutoAck::set_auto_ack()`] because it
    /// controls the auto-ack feature for only the given payload (`buf`), whereas
    /// [`EsbAutoAck::set_auto_ack()`] controls ACK packets for _all_ payloads.
    /// If [`EsbAutoAck::allow_ask_no_ack()`] is not passed `true` at least once before passing
    /// `true` to this parameter, then this parameter has no effect.
    ///
    /// This function's `start_tx` parameter determines if the radio should enter active
    /// TX mode. This function does not exit active TX mode.
    ///
    /// The radio remains in active TX mode while there are payloads available in the TX FIFO.
    /// Set the `start_tx` parameter `false` to prevent entering active TX mode. If the radio
    /// is already in active TX mode (because it is processing payloads in the TX FIFO), then
    /// this parameter has no effect.
    fn write(
        &mut self,
        buf: &[u8],
        ask_no_ack: bool,
        start_tx: bool,
    ) -> Result<bool, Self::RadioErrorType>;

    /// Similar to [`EsbRadio::send()`] but specifically for failed transmissions.
    ///
    /// Remember, any failed transmission's payload will remain in the TX FIFO.
    ///
    /// This will reuse the payload in the top level of the radio's TX FIFO.
    /// If successfully transmitted, this returns `true`, otherwise it returns `false`.
    ///
    /// Unlike [`EsbRadio::rewrite()`], this function will only make one attempt to
    /// resend the failed payload.
    fn resend(&mut self) -> Result<bool, Self::RadioErrorType>;

    /// Similar to [`EsbRadio::write()`] but specifically for failed transmissions.
    ///
    /// Remember, any failed transmission's payload will remain in the TX FIFO.
    ///
    /// This is a non-blocking helper to [`EsbRadio::resend()`].
    /// This will put the radio in an active TX mode and reuse the payload in the top level
    /// of the radio's TX FIFO.
    ///
    /// The reused payload will be continuously retransmitted until one of the following
    /// conditions occurs:
    ///
    /// - The retransmission fails.
    /// - A new payload is written to the radio's TX FIFO (via [`EsbRadio::write()`] or
    ///   [`EsbRadio::send()`])
    /// - The radio's TX FIFO is flushed (via [`EsbFifo::flush_tx()`]).
    /// - The radio's CE pin is set to inactive LOW. This can be done directly on the pin or by calling
    ///   [`EsbRadio::as_tx()`].
    fn rewrite(&mut self) -> Result<(), Self::RadioErrorType>;

    /// Get the Auto-Retry Count (ARC) about the previous transmission.
    ///
    /// This data is reset for every payload attempted to transmit.
    /// It cannot exceed 15 per the `count` parameter in [`EsbAutoAck::set_auto_retries()`].
    /// If auto-ack feature is disabled, then this function provides no useful data.
    fn get_last_arc(&mut self) -> Result<u8, Self::RadioErrorType>;

    /// Read data from the radio's RX FIFO into the specified `buf`.
    ///
    /// All payloads received by the radio are stored in the RX FIFO (a 3 layer stack).
    /// Use [`EsbFifo::available()`] to determine if there is data ready to read.
    ///
    /// The `len` parameter determines how much data is stored to `buf`. Ultimately,
    /// the value of `len` is restricted by the radio's maximum 32 byte limit and the
    /// length of the given `buf`. Pass [`None`] to automatically use static payload length
    /// (set by [`EsbPayloadLength::set_payload_length()`]) or the dynamic payload length
    /// (fetched internally using [`EsbPayloadLength::get_dynamic_payload_length()`]) if
    /// dynamic payload lengths are enable (see [`EsbPayloadLength::set_dynamic_payloads()`]).
    fn read(&mut self, buf: &mut [u8], len: Option<u8>) -> Result<u8, Self::RadioErrorType>;
}
