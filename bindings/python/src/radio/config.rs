#![allow(clippy::new_without_default)]
use super::types::{CrcLength, DataRate, PaLevel};
use pyo3::prelude::*;

use std::borrow::Cow;

/// Instantiate a [`RadioConfig`][rf24_py.RadioConfig] object with library defaults.
///
/// | feature | default value |
/// |--------:|:--------------|
/// | [`channel`][rf24_py.RadioConfig.channel] | `76` |
/// | [`address_length`][rf24_py.RadioConfig.address_length] | `5` |
/// | [`pa_level`][rf24_py.RadioConfig.pa_level] | [`PaLevel.Max`][rf24_py.PaLevel.Max] |
/// | [`lna_enable`][rf24_py.RadioConfig.lna_enable] | `True` |
/// | [`crc_length`][rf24_py.RadioConfig.crc_length] | [`CrcLength.Bit16`][rf24_py.CrcLength.Bit16] |
/// | [`data_rate`][rf24_py.RadioConfig.data_rate] | [`DataRate.Mbps1`][rf24_py.DataRate.Mbps1] |
/// | [`payload_length`][rf24_py.RadioConfig.payload_length] | `32` |
/// | [`dynamic_payloads`][rf24_py.RadioConfig.dynamic_payloads] | `False` |
/// | [`auto_ack`][rf24_py.RadioConfig.auto_ack] | `0x3F` (enabled for pipes 0 - 5) |
/// | [`ack_payloads`][rf24_py.RadioConfig.ack_payloads] | `False` |
/// | [`ask_no_ack`][rf24_py.RadioConfig.ask_no_ack] | `False` |
/// | [`auto_retry_delay`][rf24_py.RadioConfig.auto_retry_delay] | `5` |
/// | [`auto_retry_count`][rf24_py.RadioConfig.auto_retry_count] | `15` |
/// | [`tx_address`][rf24_py.RadioConfig.tx_address] | `b"\xE7" * 5` |
/// | [`get_rx_address()`][rf24_py.RadioConfig.get_rx_address] | See below table about [Default RX addresses](#default-rx-pipes-configuration) |
/// | [`rx_dr`][rf24_py.RadioConfig.rx_dr] | `True` |
/// | [`tx_ds`][rf24_py.RadioConfig.tx_ds] | `True` |
/// | [`tx_df`][rf24_py.RadioConfig.tx_df] | `True` |
///
/// ## Default RX pipes' configuration
///
/// | pipe number | state  | address       |
/// |-------------|--------|---------------|
/// |      0[^1]  | closed | `b"\xE7" * 5` |
/// |      1      | open   | `b"\xC2" * 5` |
/// |      2[^2]  | closed | `0xC3`        |
/// |      3[^2]  | closed | `0xC4`        |
/// |      4[^2]  | closed | `0xC5`        |
/// |      5[^2]  | closed | `0xC6`        |
///
/// [^1]: The RX address default value is the same as pipe 0 default RX address.
/// [^2]: Remember, pipes 2 - 5 share the same 4 LSBytes as the address on pipe 1.
#[pyclass(module = "rf24_py")]
#[derive(Debug, Clone, Copy)]
pub struct RadioConfig {
    inner: rf24::radio::RadioConfig,
    _addr_buf: [u8; 5],
}

#[pymethods]
impl RadioConfig {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: rf24::radio::RadioConfig::default(),
            _addr_buf: [0u8; 5],
        }
    }

    /// Set the channel (over the air frequency).
    ///
    /// This value is clamped to range [0, 125].
    /// The radio's frequency can be determined by the following equation:
    /// ```text
    /// frequency (in Hz) = channel + 2400
    /// ```
    #[getter]
    pub fn get_channel(&self) -> u8 {
        self.inner.channel()
    }

    #[setter]
    pub fn set_channel(&mut self, value: u8) {
        self.inner = self.inner.with_channel(value);
    }

    /// The payload length for statically sized payloads.
    ///
    /// This value can not be set larger than 32 bytes.
    /// See [`RF24.payload_length`][rf24_py.RF24.payload_length].
    #[getter]
    pub fn get_payload_length(&self) -> u8 {
        self.inner.payload_length()
    }

    #[setter]
    pub fn set_payload_length(&mut self, value: u8) {
        self.inner = self.inner.with_payload_length(value);
    }

    /// The address length.
    ///
    /// This value is clamped to range [2, 5].
    #[getter]
    pub fn get_address_length(&self) -> u8 {
        self.inner.address_length()
    }

    #[setter]
    pub fn set_address_length(&mut self, value: u8) {
        self.inner = self.inner.with_address_length(value);
    }

    /// The Cyclical Redundancy Checksum (CRC) length.
    ///
    /// See [`RF24.crc_length`][rf24_py.RF24.crc_length].
    #[getter]
    pub fn get_crc_length(&self) -> CrcLength {
        CrcLength::from_inner(self.inner.crc_length())
    }

    #[setter]
    pub fn set_crc_length(&mut self, value: CrcLength) {
        self.inner = self.inner.with_crc_length(value.into_inner());
    }

    /// The Power Amplitude (PA) level.
    ///
    /// See [`RF24.pa_level`][rf24_py.RF24.pa_level].
    #[getter]
    pub fn get_pa_level(&self) -> PaLevel {
        PaLevel::from_inner(self.inner.pa_level())
    }

    #[setter]
    pub fn set_pa_level(&mut self, value: PaLevel) {
        self.inner = self.inner.with_pa_level(value.into_inner());
    }

    /// Enable or disable the chip's Low Noise Amplifier (LNA) feature.
    ///
    /// This value may not be respected depending on the radio module used.
    /// Consult the radio's manufacturer for accurate details.
    #[getter]
    pub fn get_lna_enable(&self) -> bool {
        self.inner.lna_enable()
    }

    #[setter]
    pub fn set_lna_enable(&mut self, value: i32) {
        self.inner = self.inner.with_lna_enable(value != 0);
    }

    /// The Data Rate (over the air).
    ///
    /// See [`RF24.data_rate`][rf24_py.RF24.data_rate].
    #[getter]
    pub fn get_data_rate(&self) -> DataRate {
        DataRate::from_inner(self.inner.data_rate())
    }

    #[setter]
    pub fn set_data_rate(&mut self, value: DataRate) {
        self.inner = self.inner.with_data_rate(value.into_inner());
    }

    /// Enable or disable auto-ACK feature.
    ///
    /// The given value (in binary form) is used to control the auto-ack feature for each pipe.
    /// Bit 0 controls the feature for pipe 0. Bit 1 controls the feature for pipe 1. And so on.
    ///
    /// To enable the feature for pipes 0, 1 and 4:
    /// ```python
    /// config = RadioConfig()
    /// config.auto_ack = 0b010011
    /// ```
    /// If enabling the feature for any pipe other than 0, then the pipe 0 should also have the
    /// feature enabled because pipe 0 is used to transmit automatic ACK packets in RX mode.
    #[getter]
    pub fn get_auto_ack(&self) -> u8 {
        self.inner.auto_ack()
    }

    #[setter]
    pub fn set_auto_ack(&mut self, value: u8) {
        self.inner = self.inner.with_auto_ack(value);
    }

    /// The auto-retry feature's `delay` set by
    /// [`RadioConfig.set_auto_retries()`][rf24_py.RadioConfig.set_auto_retries].
    #[getter]
    pub fn get_auto_retry_delay(&self) -> u8 {
        self.inner.auto_retry_delay()
    }

    /// The auto-retry feature's `count` set by
    /// [`RadioConfig.set_auto_retries()`][rf24_py.RadioConfig.set_auto_retries].
    #[getter]
    pub fn get_auto_retry_count(&self) -> u8 {
        self.inner.auto_retry_count()
    }

    /// Set the auto-retry feature's `delay` and `count` parameters.
    ///
    /// See [`RF24.set_auto_retries()`][rf24_py.RF24.set_auto_retries].
    pub fn set_auto_retries(&mut self, delay: u8, count: u8) {
        self.inner = self.inner.with_auto_retries(delay, count);
    }

    /// Enable or disable dynamically sized payloads.
    ///
    /// Enabling this feature nullifies the utility of [`RadioConfig.payload_length`][rf24_py.RadioConfig.payload_length].
    ///
    /// This feature is enabled automatically when enabling ACK payloads
    /// via [`RadioConfig.ack_payloads`][rf24_py.RadioConfig.ack_payloads].
    #[getter]
    pub fn get_dynamic_payloads(&self) -> bool {
        self.inner.dynamic_payloads()
    }

    #[setter]
    pub fn set_dynamic_payloads(&mut self, value: i32) {
        self.inner = self.inner.with_dynamic_payloads(value != 0);
    }

    /// Enable or disable custom ACK payloads for auto-ACK packets.
    ///
    /// ACK payloads require the [`RadioConfig.auto_ack`][rf24_py.RadioConfig.auto_ack]
    /// and [`RadioConfig.dynamic_payloads`][rf24_py.RadioConfig.dynamic_payloads]
    /// to be enabled. If ACK payloads are enabled, then this function also enables those
    /// features (for all pipes).
    #[getter]
    pub fn get_ack_payloads(&self) -> bool {
        self.inner.ack_payloads()
    }

    #[setter]
    pub fn set_ack_payloads(&mut self, value: i32) {
        self.inner = self.inner.with_ack_payloads(value != 0);
    }

    /// Allow disabling auto-ack per payload.
    ///
    /// See `ask_no_ack` parameter for
    /// [`RF24.send()`][rf24_py.RF24.send] and
    /// [`RF24.write()`][rf24_py.RF24.write].
    #[getter]
    pub fn get_ask_no_ack(&self) -> bool {
        self.inner.ask_no_ack()
    }

    #[setter]
    pub fn set_ask_no_ack(&mut self, value: i32) {
        self.inner = self.inner.with_ask_no_ack(value != 0);
    }

    /// Enable or disable the "RX Data Ready" event triggering the radio's IRQ.
    ///
    /// See [`StatusFlags.rx_dr`][rf24_py.StatusFlags.rx_dr].
    #[getter]
    pub fn get_rx_dr(&self) -> bool {
        self.inner.rx_dr()
    }

    #[setter]
    pub fn set_rx_dr(&mut self, value: i32) {
        self.inner = self.inner.with_rx_dr(value != 0);
    }

    /// Enable or disable the "TX Data Sent" event triggering the radio's IRQ.
    ///
    /// See [`StatusFlags.tx_ds`][rf24_py.StatusFlags.tx_ds].
    #[getter]
    pub fn get_tx_ds(&self) -> bool {
        self.inner.tx_ds()
    }

    #[setter]
    pub fn set_tx_ds(&mut self, value: i32) {
        self.inner = self.inner.with_tx_ds(value != 0);
    }

    /// Enable or disable the "TX Data Failed" event triggering the radio's IRQ.
    ///
    /// See [`StatusFlags.tx_df`][rf24_py.StatusFlags.tx_df].
    #[getter]
    pub fn get_tx_df(&self) -> bool {
        self.inner.tx_df()
    }

    #[setter]
    pub fn set_tx_df(&mut self, value: i32) {
        self.inner = self.inner.with_tx_df(value != 0);
    }

    /// Is a specified RX pipe open (`true`) or closed (`false`)?
    ///
    /// The value returned here is controlled by
    /// [`RadioConfig.set_rx_address()`][rf24_py.RadioConfig.set_rx_address] (to open a pipe)
    /// and [`RadioConfig.close_rx_pipe()`][rf24_py.RadioConfig.close_rx_pipe].
    pub fn is_rx_pipe_enabled(&self, pipe: u8) -> bool {
        self.inner.is_rx_pipe_enabled(pipe)
    }

    /// Set the address of a specified RX `pipe` for receiving data.
    ///
    /// This does nothing if the given `pipe` is greater than `8`.
    /// For pipes 2 - 5, the 4 LSBytes are used from address set to pipe 1 with the
    /// MSByte from the given `address`.
    ///
    /// See also [`RadioConfig.tx_address()`][rf24_py.RadioConfig.tx_address].
    pub fn set_rx_address(&mut self, pipe: u8, address: &[u8]) {
        self.inner = self.inner.with_rx_address(pipe, address);
    }

    /// Get the address for a specified `pipe` set by [`RadioConfig.set_rx_address()`][rf24_py.RadioConfig.set_rx_address].
    pub fn get_rx_address(&mut self, pipe: u8) -> Cow<[u8]> {
        self.inner.rx_address(pipe, &mut self._addr_buf);
        Cow::from(&self._addr_buf)
    }

    /// Set the TX address.
    ///
    /// Only pipe 0 can be used for TX operations (including auto-ACK packets during RX operations).
    #[getter]
    pub fn get_tx_address(&mut self) -> Cow<[u8]> {
        self.inner.tx_address(&mut self._addr_buf);
        Cow::from(&self._addr_buf)
    }

    #[setter]
    pub fn set_tx_address(&mut self, value: &[u8]) {
        self.inner = self.inner.with_tx_address(value);
    }
    /// Close a RX pipe from receiving data.
    ///
    /// This is only useful if pipe 1 should be closed instead of open
    /// (after constructing [`RadioConfig`][rf24_py.RadioConfig]).
    pub fn close_rx_pipe(&mut self, pipe: u8) {
        self.inner = self.inner.close_rx_pipe(pipe);
    }
}

impl RadioConfig {
    pub fn get_inner(&self) -> &rf24::radio::RadioConfig {
        &self.inner
    }

    pub fn from_inner(config: rf24::radio::RadioConfig) -> Self {
        Self {
            inner: config,
            _addr_buf: [0u8; 5],
        }
    }
}
