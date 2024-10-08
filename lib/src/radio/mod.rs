mod rf24;
pub use rf24::{Nrf24Error, RF24};
pub mod prelude {
    use crate::enums::{CrcLength, DataRate, FifoState, PaLevel};

    pub trait EsbPipe {
        type PipeErrorType;

        /// Open a specified `pipe` for receiving data when radio is in RX role.
        ///
        /// If the specified `pipe` is not in range [0, 5], then this function does nothing.
        ///
        /// Up to 6 pipes can be open for reading at once.  Open all the required
        /// reading pipes, and then call [`EsbRadio::start_listening()`].
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
        /// [`EsbRadio::start_listening()`].
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

        /// Set an address to pipe 0 for transmitting when radio is in TX role.
        ///
        fn open_tx_pipe(&mut self, address: &[u8]) -> Result<(), Self::PipeErrorType>;

        /// Close a specified pipe from receiving data when radio is in RX role.
        fn close_rx_pipe(&mut self, pipe: u8) -> Result<(), Self::PipeErrorType>;
        fn set_address_length(&mut self, length: u8) -> Result<(), Self::PipeErrorType>;
        fn get_address_length(&mut self) -> Result<u8, Self::PipeErrorType>;
    }
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

    pub trait EsbStatus {
        type StatusErrorType;
        fn get_status_flags(
            &mut self,
            rx_dr: &mut Option<bool>,
            tx_ds: &mut Option<bool>,
            tx_df: &mut Option<bool>,
        ) -> Result<(), Self::StatusErrorType>;
        fn set_status_flags(
            &mut self,
            rx_dr: bool,
            tx_ds: bool,
            tx_df: bool,
        ) -> Result<(), Self::StatusErrorType>;
        fn clear_status_flags(
            &mut self,
            rx_dr: bool,
            tx_ds: bool,
            tx_df: bool,
        ) -> Result<(), Self::StatusErrorType>;
        fn update(&mut self) -> Result<(), Self::StatusErrorType>;
    }

    pub trait EsbFifo {
        type FifoErrorType;

        /// Flush the radio's RX FIFO.
        fn flush_rx(&mut self) -> Result<(), Self::FifoErrorType>;

        /// Flush the radio's TX FIFO.
        ///
        /// This function is automatically called by [`EsbRadio::stop_listening()`]
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

        /// This is similar to [`EsbFifo::available()`] except that if the `pipe` parameter is given
        /// a mutable [`Some`] value, then the pipe number that received the data is stored to it.
        ///
        /// If there is no data ready to [`EsbRadio::read()`] in the RX FIFO, then the `pipe` parameter's
        /// value is untouched.
        ///
        /// ```ignore
        /// let mut pipe = Some(9 as u8); // using an invalid pipe number
        /// if radio.available_pipe(&pipe).is_ok_and(|rv| rv) {
        ///     // `pipe` should now be set to a valid pipe number
        ///     print!("A Payload was received on pipe {}", pipe.unwrap());
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
        fn available_pipe(&mut self, pipe: &mut Option<u8>) -> Result<bool, Self::FifoErrorType>;
    }

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
        /// // ... then after or during RX role:
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
        fn set_dynamic_payloads(
            &mut self,
            enable: bool,
        ) -> Result<(), Self::PayloadLengthErrorType>;

        /// Get the dynamic length of the next available payload in the RX FIFO.
        ///
        /// This returns `0` when dynamic payloads are disabled.
        fn get_dynamic_payload_length(&mut self) -> Result<u8, Self::PayloadLengthErrorType>;
    }

    pub trait EsbAutoAck: EsbPayloadLength {
        type AutoAckErrorType;

        /// Allows appending payloads to automatic ACK (acknowledgement) packets.
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
        fn allow_ack_payloads(&mut self, enable: bool) -> Result<(), Self::AutoAckErrorType>;

        /// Write a `buf` to the radio's TX FIFO for use with automatic ACK packets.
        ///
        /// The given `buf` will be the outgoing payload added to an automatic ACK packet
        /// when acknowledging an incoming payload that was received with the specified
        /// `pipe`. This function returns `true` if the payload was written to the TX FIFO.
        /// It returns `false` under the following conditions:
        ///
        /// - the radio's TX FIFO is full.
        /// - the specified `pipe` number is invalid (not in range [0, 5]).
        /// - the ACK payload feature is not enabled (see [`EsbAutoAck::allow_ack_payloads()`])
        ///
        /// It is important to discard any non-ACK payloads in the TX FIFO (using
        /// [`EsbFifo::flush_tx()`]) before writing the first ACK payload into the TX FIFO.
        /// This function can be used before and after calling [`EsbRadio::start_listening()`].
        ///
        /// <div class="warning">
        ///
        /// The payload must be loaded into the radio's TX FIFO _before_ the incoming
        /// payload is received.
        ///
        /// Remember, the TX FIFO stack can store only 3 payloads,
        /// and there are typically more pipes than TX FIFO occupancy.
        /// Expected behavior is better assured when the ACK payloads are only used
        /// for 1 pipe
        ///
        /// </div>
        ///
        /// Since ACK payloads require the dynamic payloads feature enabled, the given
        /// `buf`'s length will determine the length of the payload in the ACK packet.
        ///
        /// See also [`EsbAutoAck::allow_ack_payloads()`],
        /// [`EsbPayloadLength::set_dynamic_payloads`], and [`EsbAutoAck::set_auto_ack()`].
        fn write_ack_payload(
            &mut self,
            pipe: u8,
            buf: &[u8],
        ) -> Result<bool, Self::AutoAckErrorType>;

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
        fn set_auto_ack_pipe(
            &mut self,
            enable: bool,
            pipe: u8,
        ) -> Result<(), Self::AutoAckErrorType>;

        /// Set the number of retry attempts and delay between retry attempts when
        /// transmitting a payload.
        ///
        /// The radio is waiting for an acknowledgement (ACK) packet during the delay between retry attempts.
        ///
        /// Both parameters are clamped to range [0, 15].
        /// - `delay`: How long to wait between each retry, in multiples of
        ///   250 us. The minimum of 0 means 250 us, and the maximum of 15 means
        ///   4000 us. The default value of 5 means 1500us (5 * 250 + 250).
        /// - `count`: How many retries before giving up. The default/maximum is 15. Use
        ///   0 to disable the auto-retry feature all together.
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
        /// allow disabling the auto-ack feature on a per-payload basis.
        fn allow_ask_no_ack(&mut self, enable: bool) -> Result<(), Self::AutoAckErrorType>;
    }

    pub trait EsbPaLevel {
        type PaLevelErrorType;

        /// Get the currently configured Power Amplitude Level (PA Level)
        fn get_pa_level(&mut self) -> Result<PaLevel, Self::PaLevelErrorType>;

        /// Set the radio's Power Amplitude Level (PA Level)
        fn set_pa_level(&mut self, pa_level: PaLevel) -> Result<(), Self::PaLevelErrorType>;
    }

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
        /// To ensure proper operation, this function will `delay` after the radio is awaken.
        /// If the `delay` parameter is given a [`Some`] value, then the this function
        /// will wait for the specified number of microseconds. If `delay` is a [`None`]
        /// value, this function will wait for 5 milliseconds.
        ///
        /// To perform other tasks will the radio is powering up:
        /// ```ignore
        /// radio.power_up(Some(0)).unwrap();
        /// // ... do something else for 5 milliseconds
        /// radio.start_listening().unwrap();
        /// ```
        fn power_up(&mut self, delay: Option<u32>) -> Result<(), Self::PowerErrorType>;
    }

    pub trait EsbCrcLength {
        type CrcLengthErrorType;

        /// Get the currently configured CRC (Cyclical Redundancy Checksum) length
        fn get_crc_length(&mut self) -> Result<CrcLength, Self::CrcLengthErrorType>;

        /// Set the radio's CRC (Cyclical Redundancy Checksum) length
        fn set_crc_length(&mut self, crc_length: CrcLength) -> Result<(), Self::CrcLengthErrorType>;
    }

    pub trait EsbDataRate {
        type DataRateErrorType;

        /// Get the currently configured Data Rate
        fn get_data_rate(&mut self) -> Result<DataRate, Self::DataRateErrorType>;

        /// Set the radio's Data Rate
        fn set_data_rate(&mut self, data_rate: DataRate) -> Result<(), Self::DataRateErrorType>;
    }

    pub trait EsbRadio:
        EsbChannel
        + EsbPipe
        + EsbStatus
        + EsbFifo
        + EsbPayloadLength
        + EsbAutoAck
        + EsbPaLevel
        + EsbPower
        + EsbCrcLength
        + EsbDataRate
    {
        type RadioErrorType;

        /// Initialize the radio's hardware
        fn init(&mut self) -> Result<(), Self::RadioErrorType>;

        /// Put the radio into RX role
        fn start_listening(&mut self) -> Result<(), Self::RadioErrorType>;

        /// Put the radio into TX role
        fn stop_listening(&mut self) -> Result<(), Self::RadioErrorType>;

        /// Blocking write.
        ///
        /// This transmits a payload (given by `buf`) and returns a bool describing if
        /// the transmission was successful or not.
        ///
        /// See [`EsbRadio::write()`] for description of `ask_no_ack` parameter and more
        /// detail about how the radio processes data in the TX FIFO.
        fn send(&mut self, buf: &[u8], ask_no_ack: bool) -> Result<bool, Self::RadioErrorType>;

        /// Non-blocking write.
        ///
        /// This function does not wait for the radio to complete the transmission.
        /// Instead it simply writes the given `buf` into the radio's TX FIFO.
        /// If the TX FIFO is already full, this function just calls
        /// [`EsbStatus::clear_status_flags()`] and returns `false`.
        ///
        /// If `ask_no_ack` is true, then the transmitted payload will not use the auto-ack
        /// feature. This parameter is different from auto-ack because it controls the
        /// auto-ack feature for only this payload, whereas the auto-ack feature controls
        /// ACK packets for all payloads. If [`EsbAutoAck::allow_ask_no_ack()`] is not called
        /// at least once prior to asserting this parameter, then it has no effect.
        ///
        /// This function's `start_tx` parameter determines if the radio should enter active
        /// TX mode. This function does not deactivate TX mode.
        ///
        /// If the radio's remains in TX mode after successfully transmitting a payload,
        /// then any subsequent payloads in the TX FIFO will automatically be processed.
        /// Set the `start_tx` parameter `false` to prevent entering TX mode.
        fn write(
            &mut self,
            buf: &[u8],
            ask_no_ack: bool,
            start_tx: bool,
        ) -> Result<bool, Self::RadioErrorType>;

        fn resend(&mut self) -> Result<bool, Self::RadioErrorType>;

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
        /// length of the given `buf`.
        fn read(&mut self, buf: &mut [u8], len: u8) -> Result<(), Self::RadioErrorType>;
    }
}
