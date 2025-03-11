use crate::radio::rf24::bit_fields::{Config, Feature, SetupRetry, SetupRfAw};
use crate::{CrcLength, DataRate, PaLevel};

/// A struct to contain configuration about pipe addresses.
#[derive(Debug, Clone, Copy)]
pub struct EsbPipeConfig {
    pub(super) tx_address: [u8; 5],
    pipe0: [u8; 5],
    pipe1: [u8; 5],
    pipe2: u8,
    pipe3: u8,
    pipe4: u8,
    pipe5: u8,
    pipe6: u8,
    pipe7: u8,
    pub(super) rx_pipes_enabled: u8,
}

impl Default for EsbPipeConfig {
    fn default() -> Self {
        Self {
            tx_address: [0xE7; 5],
            pipe0: [0xE7; 5],
            pipe1: [0xC2; 5],
            pipe2: 0xC3,
            pipe3: 0xC4,
            pipe4: 0xC5,
            pipe5: 0xC6,
            pipe6: 0xC7,
            pipe7: 0xC8,
            rx_pipes_enabled: 2,
        }
    }
}

impl EsbPipeConfig {
    pub fn set_tx_address(&mut self, address: &[u8]) {
        let len = address.len().min(5);
        self.tx_address[..len].copy_from_slice(&address[..len]);
    }

    pub fn set_rx_address(&mut self, pipe: u8, address: &[u8]) {
        let len = address.len().min(5);
        if len == 0 {
            return;
        }
        if pipe < 8 {
            self.rx_pipes_enabled |= 1 << pipe;
        }
        match pipe {
            0 => self.pipe0[..len].copy_from_slice(&address[..len]),
            1 => self.pipe1[..len].copy_from_slice(&address[..len]),
            2 => self.pipe2 = address[0],
            3 => self.pipe3 = address[0],
            4 => self.pipe4 = address[0],
            5 => self.pipe5 = address[0],
            6 => self.pipe6 = address[0],
            7 => self.pipe7 = address[0],
            _ => (),
        }
    }

    pub fn close_rx_pipe(&mut self, pipe: u8) {
        if pipe < 8 {
            self.rx_pipes_enabled &= !(1 << pipe);
        }
    }

    pub(super) fn get_rx_address(&self, pipe: u8, address: &mut [u8]) {
        let len = address.len().min(5);
        match pipe {
            0 => address[..len].copy_from_slice(&self.pipe0[..len]),
            1 => address[..len].copy_from_slice(&self.pipe1[..len]),
            2 => address[0] = self.pipe2,
            3 => address[0] = self.pipe3,
            4 => address[0] = self.pipe4,
            5 => address[0] = self.pipe5,
            6 => address[0] = self.pipe6,
            7 => address[0] = self.pipe7,
            _ => (),
        }
        if pipe > 1 && len > 1 {
            address[1..(len - 1)].copy_from_slice(&self.pipe1[1..(len - 1)]);
        }
    }
}

/// An object to configure the radio.
///
/// This struct follows a builder pattern. Since all fields are private, users should
/// start with the [`RadioConfig::default`] constructor, then mutate the object accordingly.
/// ```
/// let mut config = Config::default();
/// config = config.with_channel(42);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct RadioConfig {
    pub(crate) config_reg: Config,
    pub(crate) auto_retries: SetupRetry,
    pub(crate) setup_rf_aw: SetupRfAw,
    pub(crate) feature: Feature,
    channel: u8,
    payload_length: u8,
    auto_ack: u8,
    pipes: EsbPipeConfig,
}

impl Default for RadioConfig {
    /// Instantiate a [`RadioConfig`] object with library defaults.
    ///
    /// | feature | default value |
    /// |--------:|:--------------|
    /// | [`RadioConfig::channel()`] | `76` |
    /// | [`RadioConfig::address_length()`] | `5` |
    /// | [`RadioConfig::pa_level()`] | [`PaLevel::Max`] |
    /// | [`RadioConfig::lna_enable()`] | `true` |
    /// | [`RadioConfig::crc_length()`] | [`CrcLength::Bit16`] |
    /// | [`RadioConfig::data_rate()`] | [`DataRate::Mbps1`] |
    /// | [`RadioConfig::payload_length()`] | `32` |
    /// | [`RadioConfig::dynamic_payloads()`] | `false` |
    /// | [`RadioConfig::auto_ack()`] | `0x3F` (enabled for pipes 0 - 5) |
    /// | [`RadioConfig::ack_payloads()`] | `false` |
    /// | [`RadioConfig::ask_no_ack()`] | `false` |
    /// | [`RadioConfig::auto_retry_delay()`] | `5` |
    /// | [`RadioConfig::auto_retry_count()`] | `15` |
    /// | [`RadioConfig::tx_address()`] | `[0xE7; 5]` |
    /// | [`RadioConfig::rx_address()`] | See below table about [Default RX addresses](#default-rx-pipes-configuration) |
    /// | [`RadioConfig::rx_dr()`] | `true` |
    /// | [`RadioConfig::tx_ds()`] | `true` |
    /// | [`RadioConfig::tx_df()`] | `true` |
    ///
    /// ## Default RX pipes' configuration
    ///
    /// | pipe number | state  | address     |
    /// |-------------|--------|-------------|
    /// |      0[^2]  | closed | `[0xE7; 5]` |
    /// |      1      | open   | `[0xC2; 5]` |
    /// |      2[^1]  | closed | `0xC3`      |
    /// |      3[^1]  | closed | `0xC4`      |
    /// |      4[^1]  | closed | `0xC5`      |
    /// |      5[^1]  | closed | `0xC6`      |
    ///
    /// [^1]: Remember, pipes 2 - 5 share the same 4 LSBytes as the address on pipe 1.
    /// [^2]: The RX address default value is the same as pipe 0 default RX address.
    fn default() -> Self {
        Self {
            /*
               - all events enabled for IRQ pin
               - 8 bit CRC
               - powered down
               - inactive TX (StandBy-I) mode
            */
            config_reg: Config::default(),
            /*
               - 5 * 250 + 250 = 1500 us delay between attempts
               - 15 max attempts
            */
            auto_retries: SetupRetry::default(),
            /*
                - 5 byte address length
                - 1 Mbps data rate
                - Max PA level
                - LNA enabled
            */
            setup_rf_aw: SetupRfAw::default(),
            /*
               - disabled dynamic payloads
               - disabled ACK payloads
               - disabled ask_no_ack param
            */
            feature: Feature::default(),
            channel: 76,
            payload_length: 32,
            // enable auto-ACK for pipes 0 - 5
            auto_ack: 0x3F,
            pipes: EsbPipeConfig::default(),
        }
    }
}

impl RadioConfig {
    /// Returns the value set by [`RadioConfig::with_crc_length()`].
    pub const fn crc_length(&self) -> CrcLength {
        self.config_reg.crc_length()
    }

    /// The Cyclical Redundancy Checksum (CRC) length.
    ///
    /// See [`EsbCrcLength::set_crc_length()`](fn@crate::radio::prelude::EsbCrcLength::set_crc_length).
    pub fn with_crc_length(self, length: CrcLength) -> Self {
        let new_config = self.config_reg.with_crc_length(length);
        Self {
            config_reg: new_config,
            ..self
        }
    }

    /// Returns the value set by [`RadioConfig::with_data_rate()`].
    pub const fn data_rate(&self) -> DataRate {
        self.setup_rf_aw.data_rate()
    }

    /// The Data Rate (over the air).
    ///
    /// See [`EsbDataRate::set_data_rate()`](fn@crate::radio::prelude::EsbDataRate::set_data_rate).
    pub fn with_data_rate(self, data_rate: DataRate) -> Self {
        let new_config = self.setup_rf_aw.with_data_rate(data_rate);
        Self {
            setup_rf_aw: new_config,
            ..self
        }
    }

    /// Returns the value set by [`RadioConfig::with_pa_level()`].
    pub const fn pa_level(&self) -> PaLevel {
        self.setup_rf_aw.pa_level()
    }

    /// The Power Amplitude (PA) level.
    ///
    /// See [`EsbPaLevel::set_pa_level()`](fn@crate::radio::prelude::EsbPaLevel::set_pa_level).
    pub fn with_pa_level(self, level: PaLevel) -> Self {
        let new_config = self.setup_rf_aw.with_pa_level(level);
        Self {
            setup_rf_aw: new_config,
            ..self
        }
    }

    /// Returns the value set by [`RadioConfig::with_lna_enable()`].
    pub const fn lna_enable(&self) -> bool {
        self.setup_rf_aw.lna_enable()
    }

    /// Enable or disable the chip's Low Noise Amplifier (LNA) feature.
    ///
    /// This value may not be respected depending on the radio module used.
    /// Consult the radio's manufacturer for accurate details.
    pub fn with_lna_enable(self, enable: bool) -> Self {
        let new_config = self.setup_rf_aw.with_lna_enable(enable);
        Self {
            setup_rf_aw: new_config,
            ..self
        }
    }

    /// Returns the value set by [`RadioConfig::with_address_length()`].
    pub const fn address_length(&self) -> u8 {
        self.setup_rf_aw.address_length()
    }

    /// The address length.
    ///
    /// This value is clamped to range [2, 5].
    pub fn with_address_length(self, value: u8) -> Self {
        let new_config = self.setup_rf_aw.with_address_length(value);
        Self {
            setup_rf_aw: new_config,
            ..self
        }
    }

    /// Returns the value set by [`RadioConfig::with_channel()`].
    pub const fn channel(&self) -> u8 {
        self.channel
    }

    /// Set the channel (over the air frequency).
    ///
    /// This value is clamped to range [0, 125].
    /// The radio's frequency can be determined by the following equation:
    /// ```text
    /// frequency (in Hz) = channel + 2400
    /// ```
    pub fn with_channel(self, value: u8) -> Self {
        Self {
            channel: value.min(125),
            ..self
        }
    }

    /// The auto-retry feature's `delay` (set via [`RadioConfig::with_auto_retries()`])
    pub const fn auto_retry_delay(&self) -> u8 {
        self.auto_retries.ard()
    }

    /// The auto-retry feature's `count` (set via [`RadioConfig::with_auto_retries()`])
    pub const fn auto_retry_count(&self) -> u8 {
        self.auto_retries.arc()
    }

    /// Set the auto-retry feature's `delay` and `count` parameters.
    ///
    /// See [`EsbAutoAck::set_auto_retries()`](fn@crate::radio::prelude::EsbAutoAck::set_auto_retries).
    pub fn with_auto_retries(self, delay: u8, count: u8) -> Self {
        let new_config = self
            .auto_retries
            .with_ard(delay.min(15))
            .with_arc(count.min(15));
        Self {
            auto_retries: new_config,
            ..self
        }
    }

    /// Get the value set by [`RadioConfig::rx_dr()`].
    pub const fn rx_dr(&self) -> bool {
        self.config_reg.rx_dr()
    }

    /// Enable or disable the "RX Data Ready" event triggering the radio's IRQ.
    ///
    /// See [`StatusFlags::rx_dr()`](fn@crate::StatusFlags::rx_dr).
    pub fn with_rx_dr(self, enable: bool) -> Self {
        let new_config = self.config_reg.with_rx_dr(enable);
        Self {
            config_reg: new_config,
            ..self
        }
    }

    /// Get the value set by [`RadioConfig::tx_ds()`].
    pub const fn tx_ds(&self) -> bool {
        self.config_reg.tx_ds()
    }

    /// Enable or disable the "TX Data Sent" event triggering the radio's IRQ.
    ///
    /// See [`StatusFlags::tx_ds()`](fn@crate::StatusFlags::tx_ds).
    pub fn with_tx_ds(self, enable: bool) -> Self {
        let new_config = self.config_reg.with_tx_ds(enable);
        Self {
            config_reg: new_config,
            ..self
        }
    }

    /// Get the value set by [`RadioConfig::tx_df()`].
    pub const fn tx_df(&self) -> bool {
        self.config_reg.tx_df()
    }

    /// Enable or disable the "TX Data Failed" event triggering the radio's IRQ.
    ///
    /// See [`StatusFlags::tx_df()`](fn@crate::StatusFlags::tx_df).
    pub fn with_tx_df(self, enable: bool) -> Self {
        let new_config = self.config_reg.with_tx_df(enable);
        Self {
            config_reg: new_config,
            ..self
        }
    }

    /// Return the value set by [`RadioConfig::with_ask_no_ack()`].
    pub const fn ask_no_ack(&self) -> bool {
        self.feature.ask_no_ack()
    }

    /// Allow disabling auto-ack per payload.
    ///
    /// See `ask_no_ack` parameter for
    /// [`EsbRadio::send()`](fn@crate::radio::prelude::EsbRadio::send) and
    /// [`EsbRadio::write()`](fn@crate::radio::prelude::EsbRadio::write).
    pub fn with_ask_no_ack(self, enable: bool) -> Self {
        let new_config = self.feature.with_ask_no_ack(enable);
        Self {
            feature: new_config,
            ..self
        }
    }

    /// Return the value set by [`RadioConfig::with_dynamic_payloads()`].
    ///
    /// This feature is enabled automatically when enabling ACK payloads
    /// via [`RadioConfig::with_ack_payloads()`].
    pub const fn dynamic_payloads(&self) -> bool {
        self.feature.dynamic_payloads()
    }

    /// Enable or disable dynamically sized payloads.
    ///
    /// Enabling this feature nullifies the utility of [`RadioConfig::payload_length()`].
    pub fn with_dynamic_payloads(self, enable: bool) -> Self {
        let new_config = self.feature.with_dynamic_payloads(enable);
        Self {
            feature: new_config,
            ..self
        }
    }

    /// Return the value set by [`RadioConfig::with_auto_ack()`].
    pub const fn auto_ack(&self) -> u8 {
        self.auto_ack
    }

    /// Enable or disable auto-ACK feature.
    ///
    /// The given value (in binary form) is used to control the auto-ack feature for each pipe.
    /// Bit 0 controls the feature for pipe 0. Bit 1 controls the feature for pipe 1. And so on.
    ///
    /// To enable the feature for pipes 0, 1 and 4:
    /// ```
    /// let config = RadioConfig::default().with_auto_ack(0b010011);
    /// ```
    /// If enabling the feature for any pipe other than 0, then the pipe 0 should also have the
    /// feature enabled because pipe 0 is used to transmit automatic ACK packets in RX mode.
    pub fn with_auto_ack(self, enable: u8) -> Self {
        Self {
            auto_ack: enable,
            ..self
        }
    }

    /// Return the value set by [`RadioConfig::with_ack_payloads()`].
    pub const fn ack_payloads(&self) -> bool {
        self.feature.ack_payloads()
    }

    /// Enable or disable custom ACK payloads for auto-ACK packets.
    ///
    /// ACK payloads require the [`RadioConfig::auto_ack`] and [`RadioConfig::dynamic_payloads`]
    /// to be enabled. If ACK payloads are enabled, then this function also enables those
    /// features (for all pipes).
    pub fn with_ack_payloads(self, enable: bool) -> Self {
        let auto_ack = if enable { 0xFF } else { self.auto_ack };
        let new_config = self.feature.with_ack_payloads(enable);
        Self {
            auto_ack,
            feature: new_config,
            ..self
        }
    }

    /// Return the value set by [`RadioConfig::with_payload_length()`].
    ///
    /// The hardware's maximum payload length is enforced by the hardware specific
    /// implementations of [`EsbPayloadLength::set_payload_length()`](fn@crate::radio::prelude::EsbPayloadLength::set_payload_length).
    pub const fn payload_length(&self) -> u8 {
        self.payload_length
    }

    /// The payload length for statically sized payloads.
    ///
    /// See [`EsbPayloadLength::set_payload_length()`](fn@crate::radio::prelude::EsbPayloadLength::set_payload_length).
    pub fn with_payload_length(self, value: u8) -> Self {
        // NOTE: max payload length is enforced in hardware-specific implementations
        Self {
            payload_length: value,
            ..self
        }
    }

    // Close a RX pipe from receiving data.
    //
    // This is only useful if pipe 1 should be closed instead of open (after [`RadioConfig::default()`]).
    pub fn close_rx_pipe(self, pipe: u8) -> Self {
        let mut pipes = self.pipes;
        pipes.close_rx_pipe(pipe);
        Self { pipes, ..self }
    }

    /// Is a specified RX pipe open (`true`) or closed (`false`)?
    ///
    /// The value returned here is controlled by
    /// [`RadioConfig::with_rx_address()`] (to open a pipe) and [`RadioConfig::close_rx_pipe()`].
    pub fn is_rx_pipe_enabled(&self, pipe: u8) -> bool {
        self.pipes.rx_pipes_enabled & (1u8 << pipe.min(8)) > 0
    }

    /// Get the address for a specified `pipe` set by [`RadioConfig::with_rx_address()`]
    pub fn rx_address(&self, pipe: u8, address: &mut [u8]) {
        self.pipes.get_rx_address(pipe, address);
    }

    /// Set the address of a specified RX `pipe` for receiving data.
    ///
    /// This does nothing if the given `pipe` is greater than `8`.
    /// For pipes 2 - 5, the 4 LSBytes are used from address set to pipe 1 with the
    /// MSByte from the given `address`.
    ///
    /// See also [`RadioConfig::with_tx_address()`].
    pub fn with_rx_address(self, pipe: u8, address: &[u8]) -> Self {
        let mut pipes = self.pipes;
        pipes.set_rx_address(pipe, address);
        Self { pipes, ..self }
    }

    /// Get the address set by [`RadioConfig::with_tx_address()`]
    pub fn tx_address(&self, address: &mut [u8]) {
        let len = address.len().min(5);
        address[..len].copy_from_slice(&self.pipes.tx_address[..len]);
    }

    /// Set the TX address.
    ///
    /// Only pipe 0 can be used for TX operations (including auto-ACK packets during RX operations).
    pub fn with_tx_address(self, address: &[u8]) -> Self {
        let mut pipes = self.pipes;
        pipes.set_tx_address(address);
        Self { pipes, ..self }
    }
}

#[cfg(test)]
mod test {
    use super::RadioConfig;
    use crate::{CrcLength, DataRate, PaLevel};

    #[test]
    fn crc_length() {
        let mut config = RadioConfig::default();
        for len in [CrcLength::Disabled, CrcLength::Bit16, CrcLength::Bit8] {
            config = config.with_crc_length(len);
            assert_eq!(len, config.crc_length());
        }
    }

    #[test]
    fn config_irq_flags() {
        let mut config = RadioConfig::default();
        assert!(config.rx_dr());
        assert!(config.tx_ds());
        assert!(config.tx_df());
        config = config.with_rx_dr(false).with_tx_ds(false).with_tx_df(false);
        assert!(!config.rx_dr());
        assert!(!config.tx_ds());
        assert!(!config.tx_df());
    }

    #[test]
    fn address_length() {
        let mut config = RadioConfig::default();
        for len in 0..10 {
            config = config.with_address_length(len);
            assert_eq!(config.address_length(), len.clamp(2, 5));
        }
    }

    #[test]
    fn pa_level() {
        let mut config = RadioConfig::default();
        for level in [PaLevel::Max, PaLevel::High, PaLevel::Low, PaLevel::Min] {
            config = config.with_pa_level(level);
            assert_eq!(config.pa_level(), level);
        }
        assert!(config.lna_enable());
        config = config.with_lna_enable(false);
        assert!(!config.lna_enable());
    }

    #[test]
    fn data_rate() {
        let mut config = RadioConfig::default();
        for rate in [DataRate::Kbps250, DataRate::Mbps1, DataRate::Mbps2] {
            config = config.with_data_rate(rate);
            assert_eq!(config.data_rate(), rate);
        }
    }

    #[test]
    fn feature_register() {
        let mut config = RadioConfig::default();
        assert_eq!(config.auto_ack(), 0x3F);
        assert!(!config.ack_payloads());
        assert!(!config.dynamic_payloads());
        assert!(!config.ask_no_ack());

        config = config.with_ack_payloads(true);
        assert_eq!(config.auto_ack(), 0xFF);
        assert!(config.ack_payloads());
        assert!(config.dynamic_payloads());
        assert!(!config.ask_no_ack());

        config = config.with_ask_no_ack(true).with_ack_payloads(false);
        assert!(!config.ack_payloads());
        assert!(config.dynamic_payloads());
        assert!(config.ask_no_ack());

        config = config.with_dynamic_payloads(false);
        assert!(!config.dynamic_payloads());
        assert!(!config.ack_payloads());
        assert!(config.ask_no_ack());

        config = config.with_auto_ack(3);
        assert_eq!(config.auto_ack(), 3);
        assert!(!config.dynamic_payloads());
    }

    #[test]
    fn payload_length() {
        let config = RadioConfig::default().with_payload_length(255);
        assert_eq!(config.payload_length(), 255);
    }

    #[test]
    fn channel() {
        let config = RadioConfig::default().with_channel(255);
        assert_eq!(config.channel(), 125);
    }
    #[test]
    fn auto_retries() {
        let mut config = RadioConfig::default();
        assert_eq!(config.auto_retry_count(), 15);
        assert_eq!(config.auto_retry_delay(), 5);
        config = config.with_auto_retries(20, 3);
        assert_eq!(config.auto_retry_count(), 3);
        assert_eq!(config.auto_retry_delay(), 15);
    }

    #[test]
    fn pipe_addresses() {
        let mut config = RadioConfig::default();
        let mut address = [0xB0; 5];
        config = config.with_tx_address(&address);
        let mut result = [0; 3];
        config.tx_address(&mut result);
        assert!(address.starts_with(&result));
        config = config.close_rx_pipe(1).close_rx_pipe(10);
        // just for coverage, pass a empty byte array as RX address
        config = config.with_rx_address(0, &[]);
        assert!(!config.is_rx_pipe_enabled(1));
        for pipe in 0..=8 {
            address.copy_from_slice(&[0xB0 + pipe; 5]);
            config = config.with_rx_address(pipe, &address);
            config.rx_address(pipe, &mut result);
            if pipe < 2 {
                assert!(address.starts_with(&result));
            } else if pipe < 8 {
                assert_eq!(address[0], result[0]);
                // check base from pipe 1 is used for LSBs
                assert!(result[1..].starts_with(&[0xB1, 0xB1]));
            } else {
                // pipe > 8 result in non-op mutations
                assert_ne!(address[0], result[0]);
                // check base from pipe 1 is still used for LSBs
                assert!(result[1..].starts_with(&[0xB1, 0xB1]));
            }

            if pipe < 8 {
                assert!(config.is_rx_pipe_enabled(pipe));
            }
        }
    }
}
