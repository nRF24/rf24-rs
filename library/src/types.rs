//! This module defines types used by various traits.
//! These types are meant to be agnostic of the trait implementation.

use core::{
    fmt::{Display, Formatter, Result},
    write,
};

/// Power Amplifier level. The units dBm (decibel-milliwatts or dB<sub>mW</sub>)
/// represents a logarithmic signal loss.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PaLevel {
    /// | nRF24L01 | Si24R1 with<br>LNA Enabled | Si24R1 with<br>LNA Disabled |
    /// | :-------:|:--------------------------:|:---------------------------:|
    /// | -18 dBm | -6 dBm | -12 dBm |
    Min,
    /// | nRF24L01 | Si24R1 with<br>LNA Enabled | Si24R1 with<br>LNA Disabled |
    /// | :-------:|:--------------------------:|:---------------------------:|
    /// | -12 dBm | 0 dBm | -4 dBm |
    Low,
    /// | nRF24L01 | Si24R1 with<br>LNA Enabled | Si24R1 with<br>LNA Disabled |
    /// | :-------:|:--------------------------:|:---------------------------:|
    /// | -6 dBm | 3 dBm | 1 dBm |
    High,
    /// | nRF24L01 | Si24R1 with<br>LNA Enabled | Si24R1 with<br>LNA Disabled |
    /// | :-------:|:--------------------------:|:---------------------------:|
    /// | 0 dBm | 7 dBm | 4 dBm |
    Max,
}

#[cfg(feature = "defmt")]
#[cfg(target_os = "none")]
impl defmt::Format for PaLevel {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            PaLevel::Min => defmt::write!(fmt, "Min"),
            PaLevel::Low => defmt::write!(fmt, "Low"),
            PaLevel::High => defmt::write!(fmt, "High"),
            PaLevel::Max => defmt::write!(fmt, "Max"),
        }
    }
}

impl Display for PaLevel {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            PaLevel::Min => write!(f, "Min"),
            PaLevel::Low => write!(f, "Low"),
            PaLevel::High => write!(f, "High"),
            PaLevel::Max => write!(f, "Max"),
        }
    }
}

/// How fast data moves through the air. Units are in bits per second (bps).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DataRate {
    /// represents 1 Mbps
    Mbps1,
    /// represents 2 Mbps
    Mbps2,
    /// represents 250 Kbps
    Kbps250,
}

#[cfg(feature = "defmt")]
#[cfg(target_os = "none")]
impl defmt::Format for DataRate {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            DataRate::Mbps1 => defmt::write!(fmt, "1 Mbps"),
            DataRate::Mbps2 => defmt::write!(fmt, "2 Mbps"),
            DataRate::Kbps250 => defmt::write!(fmt, "250 Kbps"),
        }
    }
}

impl Display for DataRate {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            DataRate::Mbps1 => write!(f, "1 Mbps"),
            DataRate::Mbps2 => write!(f, "2 Mbps"),
            DataRate::Kbps250 => write!(f, "250 Kbps"),
        }
    }
}

/// The length of a CRC checksum that is used (if any).
///
/// Cyclical Redundancy Checking (CRC) is commonly used to ensure data integrity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CrcLength {
    /// represents no CRC checksum is used
    Disabled,
    /// represents CRC 8 bit checksum is used
    Bit8,
    /// represents CRC 16 bit checksum is used
    Bit16,
}

#[cfg(feature = "defmt")]
#[cfg(target_os = "none")]
impl defmt::Format for CrcLength {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            CrcLength::Disabled => defmt::write!(fmt, "disabled"),
            CrcLength::Bit8 => defmt::write!(fmt, "8 bit"),
            CrcLength::Bit16 => defmt::write!(fmt, "16 bit"),
        }
    }
}

impl Display for CrcLength {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            CrcLength::Disabled => write!(f, "disabled"),
            CrcLength::Bit8 => write!(f, "8 bit"),
            CrcLength::Bit16 => write!(f, "16 bit"),
        }
    }
}

/// The possible states of a FIFO.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FifoState {
    /// Represent the state of a FIFO when it is full.
    Full,
    /// Represent the state of a FIFO when it is empty.
    Empty,
    /// Represent the state of a FIFO when it is not full but not empty either.
    Occupied,
}

#[cfg(feature = "defmt")]
#[cfg(target_os = "none")]
impl defmt::Format for FifoState {
    #[cfg(feature = "defmt")]
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            FifoState::Empty => defmt::write!(fmt, "Empty"),
            FifoState::Full => defmt::write!(fmt, "Full"),
            FifoState::Occupied => defmt::write!(fmt, "Occupied"),
        }
    }
}

impl Display for FifoState {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            FifoState::Empty => write!(f, "Empty"),
            FifoState::Full => write!(f, "Full"),
            FifoState::Occupied => write!(f, "Occupied"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
/// A struct used to describe the different interrupt events.
pub struct StatusFlags {
    /// A flag to describe if RX Data Ready to read.
    pub rx_dr: bool,
    /// A flag to describe if TX Data Sent.
    pub tx_ds: bool,
    /// A flag to describe if TX Data Failed.
    pub tx_df: bool,
}

#[cfg(feature = "defmt")]
impl defmt::Format for StatusFlags {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(
            fmt,
            "StatusFlags rx_dr: {}, tx_ds: {}, tx_df: {}",
            self.rx_dr,
            self.tx_ds,
            self.tx_df
        )
    }
}

impl Display for StatusFlags {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "StatusFlags rx_dr: {}, tx_ds: {}, tx_df: {}",
            self.rx_dr, self.tx_ds, self.tx_df
        )
    }
}
