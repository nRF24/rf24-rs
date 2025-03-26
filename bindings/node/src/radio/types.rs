//! This module defines thin wrappers around rust native types to be exposed in node.js

use napi::{JsNumber, Result};

/// A private helper to implicitly convert JS numbers to boolean values (falling back to a `default` value)
pub fn coerce_to_bool(napi_instance: Option<JsNumber>, default: bool) -> Result<bool> {
    if let Some(napi_value) = napi_instance {
        return napi_value.coerce_to_bool()?.get_value();
    }
    Ok(default)
}

/// Optional configuration parameters to fine tune instantiating the {@link RF24} object.
/// Pass this object as third parameter to {@link RF24} constructor.
#[napi(object)]
pub struct HardwareConfig {
    /// The GPIO chip number: `/dev/gpiochipN` where `N` is this value.
    ///
    /// @defaultValue `0`, but needs to be `4` on RPi5 (or newer).
    /// This may also need to be specified for nVidia's hardware offerings.
    pub dev_gpio_chip: Option<u8>,

    /// The SPI bus number: `/dev/spidevX.Y` where `X` is this value
    /// and `Y` is the `csPin` required parameter to {@link RF24} constructor
    ///
    /// @defaultValue `0`, but can be as high as `3` depending on the number of
    /// SPI buses available/exposed on the board.
    pub dev_spi_bus: Option<u8>,

    /// The SPI speed in Hz used to communicate with the nRF24L01 over SPI.
    ///
    /// @defaultValue `10000000` (10 MHz) which is the radio's maximum
    /// supported speed. Lower this to 6 or 4 MHz when using long wires or
    /// if builtin pull-up resistors are weak.
    pub spi_speed: Option<u32>,
}

impl Default for HardwareConfig {
    fn default() -> Self {
        Self {
            dev_gpio_chip: Some(0),
            dev_spi_bus: Some(0),
            spi_speed: Some(10_000_000),
        }
    }
}

/// The return type for {@link RF24.getStatusFlags}
/// and optional parameters for {@link RF24.setStatusFlags}
/// and {@link RF24.clearStatusFlags}.
#[napi(object)]
#[derive(Default)]
pub struct StatusFlags {
    /// A flag to describe if RX Data Ready to read.
    ///
    /// @defaultValue `false`
    pub rx_dr: Option<bool>,
    /// A flag to describe if TX Data Sent.
    ///
    /// @defaultValue `false`
    pub tx_ds: Option<bool>,
    /// A flag to describe if TX Data Failed.
    ///
    /// @defaultValue `false`
    pub tx_df: Option<bool>,
}

#[cfg_attr(
    not(target_os = "linux"),
    allow(dead_code, reason = "only used on linux")
)]
impl StatusFlags {
    pub fn into_inner(self) -> rf24::StatusFlags {
        rf24::StatusFlags::default()
            .with_rx_dr(self.rx_dr.unwrap_or_default())
            .with_tx_ds(self.tx_ds.unwrap_or_default())
            .with_tx_df(self.tx_df.unwrap_or_default())
    }

    pub fn from_inner(other: rf24::StatusFlags) -> Self {
        Self {
            rx_dr: Some(other.rx_dr()),
            tx_ds: Some(other.tx_ds()),
            tx_df: Some(other.tx_df()),
        }
    }
}

/// An optional configuration for {@link RF24.write}
#[napi(object)]
pub struct WriteConfig {
    /// Set to `true` if you want to disable auto-ACK feature for the individual
    /// payload (required `buf` parameter to {@link RF24.write}).
    ///
    /// @defaultValue `false`. Be sure to set {@link RF24.allowAskNoAck} to `true`
    /// at least once beforehand, otherwise this option will have no affect at all.
    pub ask_no_ack: Option<bool>,

    /// Set to `true` to assert the radio's CE pin (and begin active TX mode) after the payload is
    /// uploaded to the TX FIFO.
    ///
    /// Only set this to false if filling the TX FIFO (maximum 3 level stack) before entering
    /// active TX mode. Setting this option to false does not deactivate the radio's CE pin.
    ///
    /// @defaultValue `true`
    pub start_tx: Option<bool>,
}

impl Default for WriteConfig {
    fn default() -> Self {
        Self {
            ask_no_ack: Some(false),
            start_tx: Some(true),
        }
    }
}

/// The return type for {@link RF24.availablePipe}
#[napi(object)]
pub struct AvailablePipe {
    /// Is RX data available in the RX FIFO?
    pub available: bool,
    /// The pipe number that received the next available payload in the RX FIFO.
    ///
    /// This shall be considered an invalid value if `available` is false.
    pub pipe: u8,
}

/// Power Amplifier level. The units dBm (decibel-milliwatts or dB<sub>mW</sub>)
/// represents a logarithmic signal loss.
#[napi]
#[derive(Debug, PartialEq)]
pub enum PaLevel {
    /// | nRF24L01 | Si24R1 with<br>LNA Enabled | Si24R1 with<br>LNA Disabled |
    /// |:--------:|:--------------------------:|:---------------------------:|
    /// | -18 dBm | -6 dBm | -12 dBm |
    Min,
    /// | nRF24L01 | Si24R1 with<br>LNA Enabled | Si24R1 with<br>LNA Disabled |
    /// |:--------:|:--------------------------:|:---------------------------:|
    /// | -12 dBm | 0 dBm | -4 dBm |
    Low,
    /// | nRF24L01 | Si24R1 with<br>LNA Enabled | Si24R1 with<br>LNA Disabled |
    /// |:--------:|:--------------------------:|:---------------------------:|
    /// | -6 dBm | 3 dBm | 1 dBm |
    High,
    /// | nRF24L01 | Si24R1 with<br>LNA Enabled | Si24R1 with<br>LNA Disabled |
    /// |:--------:|:--------------------------:|:---------------------------:|
    /// | 0 dBm | 7 dBm | 4 dBm |
    Max,
}

impl PaLevel {
    pub fn into_inner(self) -> rf24::PaLevel {
        match self {
            PaLevel::Min => rf24::PaLevel::Min,
            PaLevel::Low => rf24::PaLevel::Low,
            PaLevel::High => rf24::PaLevel::High,
            PaLevel::Max => rf24::PaLevel::Max,
        }
    }
    pub fn from_inner(other: rf24::PaLevel) -> PaLevel {
        match other {
            rf24::PaLevel::Min => PaLevel::Min,
            rf24::PaLevel::Low => PaLevel::Low,
            rf24::PaLevel::High => PaLevel::High,
            rf24::PaLevel::Max => PaLevel::Max,
        }
    }
}

/// How fast data moves through the air. Units are in bits per second (bps).
#[napi]
#[derive(Debug, PartialEq)]
pub enum DataRate {
    /// Represents 1 Mbps
    Mbps1,
    /// Represents 2 Mbps
    Mbps2,
    /// Represents 250 Kbps
    Kbps250,
}

impl DataRate {
    pub fn into_inner(self) -> rf24::DataRate {
        match self {
            DataRate::Mbps1 => rf24::DataRate::Mbps1,
            DataRate::Mbps2 => rf24::DataRate::Mbps2,
            DataRate::Kbps250 => rf24::DataRate::Kbps250,
        }
    }
    pub fn from_inner(other: rf24::DataRate) -> DataRate {
        match other {
            rf24::DataRate::Mbps1 => DataRate::Mbps1,
            rf24::DataRate::Mbps2 => DataRate::Mbps2,
            rf24::DataRate::Kbps250 => DataRate::Kbps250,
        }
    }
}

/// The length of a CRC checksum that is used (if any).
///
/// Cyclical Redundancy Checking (CRC) is commonly used to ensure data integrity.
#[napi]
#[derive(Debug, PartialEq)]
pub enum CrcLength {
    /// Represents no CRC checksum is used
    Disabled,
    /// Represents CRC 8 bit checksum is used
    Bit8,
    /// Represents CRC 16 bit checksum is used
    Bit16,
}

impl CrcLength {
    pub fn into_inner(self) -> rf24::CrcLength {
        match self {
            CrcLength::Disabled => rf24::CrcLength::Disabled,
            CrcLength::Bit8 => rf24::CrcLength::Bit8,
            CrcLength::Bit16 => rf24::CrcLength::Bit16,
        }
    }
    pub fn from_inner(other: rf24::CrcLength) -> CrcLength {
        match other {
            rf24::CrcLength::Disabled => CrcLength::Disabled,
            rf24::CrcLength::Bit8 => CrcLength::Bit8,
            rf24::CrcLength::Bit16 => CrcLength::Bit16,
        }
    }
}

/// The possible states of a FIFO.
#[napi]
#[derive(Debug, PartialEq)]
pub enum FifoState {
    /// Represent the state of a FIFO when it is full.
    Full,
    /// Represent the state of a FIFO when it is empty.
    Empty,
    /// Represent the state of a FIFO when it is not full but not empty either.
    Occupied,
}

#[cfg_attr(
    not(target_os = "linux"),
    allow(dead_code, reason = "only used on linux")
)]
impl FifoState {
    pub fn into_inner(self) -> rf24::FifoState {
        match self {
            FifoState::Full => rf24::FifoState::Full,
            FifoState::Empty => rf24::FifoState::Empty,
            FifoState::Occupied => rf24::FifoState::Occupied,
        }
    }
    pub fn from_inner(other: rf24::FifoState) -> FifoState {
        match other {
            rf24::FifoState::Full => FifoState::Full,
            rf24::FifoState::Empty => FifoState::Empty,
            rf24::FifoState::Occupied => FifoState::Occupied,
        }
    }
}
