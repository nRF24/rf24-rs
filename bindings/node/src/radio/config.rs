#![allow(clippy::new_without_default)]
use super::types::{coerce_to_bool, CrcLength, DataRate, PaLevel};

use napi::{bindgen_prelude::Buffer, JsNumber, Result};

/// An object to configure the radio.
#[napi]
#[derive(Debug, Clone, Copy)]
pub struct RadioConfig {
    inner: rf24::radio::RadioConfig,
    addr_buf: [u8; 5],
}

#[napi]
impl RadioConfig {
    /// Instantiate a {@link RadioConfig} object with library defaults.
    ///
    /// | feature | default value |
    /// |--------:|:--------------|
    /// | {@link RadioConfig.channel} | `76` |
    /// | {@link RadioConfig.addressLength} | `5` |
    /// | {@link RadioConfig.paLevel} | {@link PaLevel.Max} |
    /// | {@link RadioConfig.lnaEnable} | `true` |
    /// | {@link RadioConfig.crcLength} | {@link CrcLength.Bit16} |
    /// | {@link RadioConfig.dataRate} | {@link DataRate.Mbps1} |
    /// | {@link RadioConfig.payloadLength} | `32` |
    /// | {@link RadioConfig.dynamicPayloads} | `false` |
    /// | {@link RadioConfig.autoAck} | `0x3F` (enabled for pipes 0 - 5) |
    /// | {@link RadioConfig.ackPayloads} | `false` |
    /// | {@link RadioConfig.askNoAck} | `false` |
    /// | {@link RadioConfig.autoRetryDelay} | `5` |
    /// | {@link RadioConfig.autoRetryCount} | `15` |
    /// | {@link RadioConfig.txAddress} | `[0xE7, 0xE7, 0xE7, 0xE7, 0xE7]` |
    /// | {@link RadioConfig.getRxAddress} | See below table about [Default RX addresses](#default-rx-pipes-configuration) |
    /// | {@link RadioConfig.rxDr} | `true` |
    /// | {@link RadioConfig.txDs} | `true` |
    /// | {@link RadioConfig.txDf} | `true` |
    ///
    /// #### Default RX pipes' configuration
    ///
    /// | pipe number | state  | address     |
    /// |-------------|--------|-------------|
    /// |    0[^1]    | closed | `[0xE7, 0xE7, 0xE7, 0xE7, 0xE7]` |
    /// |    1        | open   | `[0xC2, 0xC2, 0xC2, 0xC2, 0xC2]` |
    /// |    2[^2]    | closed | `0xC3`      |
    /// |    3[^2]    | closed | `0xC4`      |
    /// |    4[^2]    | closed | `0xC5`      |
    /// |    5[^2]    | closed | `0xC6`      |
    ///
    /// [^1]: The RX address default value is the same as pipe 0 default TX address.
    /// [^2]: Remember, pipes 2 - 5 share the same 4 LSBytes as the address on pipe 1.
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: rf24::radio::RadioConfig::default(),
            addr_buf: [0u8; 5],
        }
    }

    #[napi(getter, js_name = "channel")]
    pub fn get_channel(&self) -> u8 {
        self.inner.channel()
    }

    /// Set the channel (over the air frequency).
    ///
    /// This value is clamped to range [0, 125].
    /// The radio's frequency can be determined by the following equation:
    /// ```text
    /// frequency (in Hz) = channel + 2400
    /// ```
    #[napi(setter, js_name = "channel")]
    pub fn set_channel(&mut self, value: u8) {
        self.inner = self.inner.with_channel(value);
    }

    #[napi(getter, js_name = "payloadLength")]
    pub fn get_payload_length(&self) -> u8 {
        self.inner.payload_length()
    }

    /// The payload length for statically sized payloads.
    ///
    /// This value can not be set larger than 32 bytes.
    /// See {@link RF24.payloadLength | `RF24.payloadLength()`}.
    #[napi(setter, js_name = "payloadLength")]
    pub fn set_payload_length(&mut self, value: u8) {
        self.inner = self.inner.with_payload_length(value);
    }

    #[napi(getter, js_name = "addressLength")]
    pub fn get_address_length(&self) -> u8 {
        self.inner.address_length()
    }

    /// The address length.
    ///
    /// This value is clamped to range [2, 5].
    #[napi(setter, js_name = "addressLength")]
    pub fn set_address_length(&mut self, value: u8) {
        self.inner = self.inner.with_address_length(value);
    }

    #[napi(getter, js_name = "crcLength")]
    pub fn get_crc_length(&self) -> CrcLength {
        CrcLength::from_inner(self.inner.crc_length())
    }

    /// The Cyclical Redundancy Checksum (CRC) length.
    ///
    /// See {@link RF24.crcLength}.
    #[napi(setter, js_name = "crcLength")]
    pub fn set_crc_length(&mut self, value: CrcLength) {
        self.inner = self.inner.with_crc_length(value.into_inner());
    }

    #[napi(getter, js_name = "paLevel")]
    pub fn get_pa_level(&self) -> PaLevel {
        PaLevel::from_inner(self.inner.pa_level())
    }

    /// The Power Amplitude (PA) level.
    ///
    /// See {@link RF24.paLevel | `RF24.paLevel()`}.
    #[napi(setter, js_name = "paLevel")]
    pub fn set_pa_level(&mut self, value: PaLevel) {
        self.inner = self.inner.with_pa_level(value.into_inner());
    }

    #[napi(getter, js_name = "lnaEnable")]
    pub fn get_lna_enable(&self) -> bool {
        self.inner.lna_enable()
    }

    /// Enable or disable the chip's Low Noise Amplifier (LNA) feature.
    ///
    /// This value may not be respected depending on the radio module used.
    /// Consult the radio's manufacturer for accurate details.
    #[napi(setter, js_name = "lnaEnable")]
    pub fn set_lna_enable(
        &mut self,
        #[napi(ts_arg_type = "boolean | number")] value: JsNumber,
    ) -> Result<()> {
        let value = coerce_to_bool(Some(value), true)?;
        self.inner = self.inner.with_lna_enable(value);
        Ok(())
    }

    #[napi(getter, js_name = "dataRate")]
    pub fn get_data_rate(&self) -> DataRate {
        DataRate::from_inner(self.inner.data_rate())
    }

    /// The Data Rate (over the air).
    ///
    /// See {@link RF24.dataRate}.
    #[napi(setter, js_name = "dataRate")]
    pub fn set_data_rate(&mut self, value: DataRate) {
        self.inner = self.inner.with_data_rate(value.into_inner());
    }

    #[napi(getter, js_name = "autoAck")]
    pub fn get_auto_ack(&self) -> u8 {
        self.inner.auto_ack()
    }

    /// Enable or disable auto-ACK feature.
    ///
    /// The given value (in binary form) is used to control the auto-ack feature for each pipe.
    /// Bit 0 controls the feature for pipe 0. Bit 1 controls the feature for pipe 1. And so on.
    ///
    /// To enable the feature for pipes 0, 1 and 4:
    /// ```js
    /// let config = new RadioConfig();
    /// config.auto_ack = 0b010011;
    /// ```
    /// If enabling the feature for any pipe other than 0, then the pipe 0 should also have the
    /// feature enabled because pipe 0 is used to transmit automatic ACK packets in RX mode.
    #[napi(setter, js_name = "autoAck")]
    pub fn set_auto_ack(&mut self, value: u8) {
        self.inner = self.inner.with_auto_ack(value);
    }

    /// The auto-retry feature's `delay` set by
    /// {@link RadioConfig.setAutoRetries}.
    #[napi(getter, js_name = "autoRetryDelay")]
    pub fn get_auto_retry_delay(&self) -> u8 {
        self.inner.auto_retry_delay()
    }

    /// The auto-retry feature's `count` set by
    /// {@link RadioConfig.setAutoRetries}.
    #[napi(getter, js_name = "autoRetryCount")]
    pub fn get_auto_retry_count(&self) -> u8 {
        self.inner.auto_retry_count()
    }

    /// Set the auto-retry feature's `delay` and `count` parameters.
    ///
    /// See {@link RF24.setAutoRetries}.
    #[napi]
    pub fn set_auto_retries(&mut self, delay: u8, count: u8) {
        self.inner = self.inner.with_auto_retries(delay, count);
    }

    #[napi(getter, js_name = "dynamicPayloads")]
    pub fn get_dynamic_payloads(&self) -> bool {
        self.inner.dynamic_payloads()
    }

    /// Enable or disable dynamically sized payloads.
    ///
    /// Enabling this feature nullifies the utility of {@link RadioConfig.payloadLength}.
    ///
    /// This feature is enabled automatically when enabling ACK payloads
    /// via {@link RadioConfig.ackPayloads}.
    #[napi(setter, js_name = "dynamicPayloads")]
    pub fn set_dynamic_payloads(
        &mut self,
        #[napi(ts_arg_type = "boolean | number")] value: JsNumber,
    ) -> Result<()> {
        let value = coerce_to_bool(Some(value), false)?;
        self.inner = self.inner.with_dynamic_payloads(value);
        Ok(())
    }

    #[napi(getter, js_name = "ackPayloads")]
    pub fn get_ack_payloads(&self) -> bool {
        self.inner.ack_payloads()
    }

    /// Enable or disable custom ACK payloads for auto-ACK packets.
    ///
    /// ACK payloads require the {@link RadioConfig.autoAck}
    /// and {@link RadioConfig.dynamicPayloads}
    /// to be enabled. If ACK payloads are enabled, then this function also enables those
    /// features (for all pipes).
    #[napi(setter, js_name = "ackPayloads")]
    pub fn set_ack_payloads(
        &mut self,
        #[napi(ts_arg_type = "boolean | number")] value: JsNumber,
    ) -> Result<()> {
        let value = coerce_to_bool(Some(value), false)?;
        self.inner = self.inner.with_ack_payloads(value);
        Ok(())
    }

    #[napi(getter, js_name = "askNoAck")]
    pub fn get_ask_no_ack(&self) -> bool {
        self.inner.ask_no_ack()
    }

    /// Allow disabling auto-ack per payload.
    ///
    /// See `askNoAck` parameter for
    /// {@link RF24.send} and {@link RF24.write} ({@link WriteConfig.askNoAck}).
    #[napi(setter, js_name = "askNoAck")]
    pub fn set_ask_no_ack(
        &mut self,
        #[napi(ts_arg_type = "boolean | number")] value: JsNumber,
    ) -> Result<()> {
        let value = coerce_to_bool(Some(value), false)?;
        self.inner = self.inner.with_ask_no_ack(value);
        Ok(())
    }

    #[napi(getter, js_name = "rxDr")]
    pub fn get_rx_dr(&self) -> bool {
        self.inner.rx_dr()
    }

    /// Enable or disable the "RX Data Ready" event triggering the radio's IRQ.
    ///
    /// See {@link StatusFlags.rxDr}.
    #[napi(setter, js_name = "rxDr")]
    pub fn set_rx_dr(
        &mut self,
        #[napi(ts_arg_type = "boolean | number")] value: JsNumber,
    ) -> Result<()> {
        let value = coerce_to_bool(Some(value), false)?;
        self.inner = self.inner.with_rx_dr(value);
        Ok(())
    }

    #[napi(getter, js_name = "txDs")]
    pub fn get_tx_ds(&self) -> bool {
        self.inner.tx_ds()
    }

    /// Enable or disable the "TX Data Sent" event triggering the radio's IRQ.
    ///
    /// See {@link StatusFlags.txDs}.
    #[napi(setter, js_name = "txDs")]
    pub fn set_tx_ds(
        &mut self,
        #[napi(ts_arg_type = "boolean | number")] value: JsNumber,
    ) -> Result<()> {
        let value = coerce_to_bool(Some(value), false)?;
        self.inner = self.inner.with_tx_ds(value);
        Ok(())
    }

    #[napi(getter, js_name = "txDf")]
    pub fn get_tx_df(&self) -> bool {
        self.inner.tx_df()
    }

    /// Enable or disable the "TX Data Failed" event triggering the radio's IRQ.
    ///
    /// See {@link StatusFlags.txDf}.
    #[napi(setter, js_name = "txDf")]
    pub fn set_tx_df(
        &mut self,
        #[napi(ts_arg_type = "boolean | number")] value: JsNumber,
    ) -> Result<()> {
        let value = coerce_to_bool(Some(value), false)?;
        self.inner = self.inner.with_tx_df(value);
        Ok(())
    }

    /// Is a specified RX pipe open (`true`) or closed (`false`)?
    ///
    /// The value returned here is controlled by
    /// {@link RadioConfig.setRxAddress} (to open a pipe)
    /// and {@link RadioConfig.closeRxPipe}.
    #[napi]
    pub fn is_rx_pipe_enabled(&self, pipe: u8) -> bool {
        self.inner.is_rx_pipe_enabled(pipe)
    }

    /// Set the address of a specified RX `pipe` for receiving data.
    ///
    /// This does nothing if the given `pipe` is greater than `8`.
    /// For pipes 2 - 5, the 4 LSBytes are used from address set to pipe 1 with the
    /// MSByte from the given `address`.
    ///
    /// See also {@link RadioConfig.txAddress}.
    #[napi]
    pub fn set_rx_address(&mut self, pipe: u8, address: Buffer) {
        let address = address.to_vec();
        self.inner = self.inner.with_rx_address(pipe, &address)
    }

    /// Get the address for a specified `pipe` set by {@link RadioConfig.setRxAddress}.
    #[napi]
    pub fn get_rx_address(&mut self, pipe: u8) -> Buffer {
        self.inner.rx_address(pipe, &mut self.addr_buf);
        Buffer::from(self.addr_buf.to_vec())
    }

    /// Set the TX address.
    ///
    /// Only pipe 0 can be used for TX operations (including auto-ACK packets during RX operations).
    #[napi(setter, js_name = "txAddress")]
    pub fn set_tx_address(&mut self, value: Buffer) {
        let value = value.to_vec();
        self.inner = self.inner.with_tx_address(&value);
    }

    #[napi(getter, js_name = "txAddress")]
    pub fn get_tx_address(&mut self) -> Buffer {
        self.inner.tx_address(&mut self.addr_buf);
        Buffer::from(self.addr_buf.to_vec())
    }

    /// Close a RX pipe from receiving data.
    ///
    /// This is only useful if pipe 1 should be closed instead of open
    /// (after constructing {@link RadioConfig}).
    #[napi]
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
            addr_buf: [0u8; 5],
        }
    }
}
