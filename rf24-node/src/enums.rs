#[cfg(target_os = "linux")]
use rf24_rs::{CrcLength, DataRate, FifoState, PaLevel};

/// The return type for `RF24.getStatusFlags()`
#[napi(object)]
pub struct StatusFlags {
    pub rx_dr: bool,
    pub tx_ds: bool,
    pub tx_df: bool,
}

/// The return type for `RF24.availablePipe()`
#[napi(object)]
pub struct AvailablePipe {
    pub available: bool,
    pub pipe: u8,
}

/// Power Amplifier level. The units dBm (decibel-milliwatts or dB<sub>mW</sub>)
/// represents a logarithmic signal loss.
#[napi(js_name = "PaLevel")]
#[derive(Debug, PartialEq)]
pub enum NodePaLevel {
    /// | nRF24L01 | Si24R1 with<br>LNA Enabled | Si24R1 with<br>LNA Disabled |
    /// | :-------:|:--------------------------:|:---------------------------:|
    /// | -18 dBm | -6 dBm | -12 dBm |
    MIN,
    /// | nRF24L01 | Si24R1 with<br>LNA Enabled | Si24R1 with<br>LNA Disabled |
    /// | :-------:|:--------------------------:|:---------------------------:|
    /// | -12 dBm | 0 dBm | -4 dBm |
    LOW,
    /// | nRF24L01 | Si24R1 with<br>LNA Enabled | Si24R1 with<br>LNA Disabled |
    /// | :-------:|:--------------------------:|:---------------------------:|
    /// | -6 dBm | 3 dBm | 1 dBm |
    HIGH,
    /// | nRF24L01 | Si24R1 with<br>LNA Enabled | Si24R1 with<br>LNA Disabled |
    /// | :-------:|:--------------------------:|:---------------------------:|
    /// | 0 dBm | 7 dBm | 4 dBm |
    MAX,
}

#[cfg(target_os = "linux")]
impl NodePaLevel {
    pub fn into_inner(self) -> PaLevel {
        match self {
            NodePaLevel::MIN => PaLevel::MIN,
            NodePaLevel::LOW => PaLevel::LOW,
            NodePaLevel::HIGH => PaLevel::HIGH,
            NodePaLevel::MAX => PaLevel::MAX,
        }
    }
    pub fn from_inner(other: PaLevel) -> NodePaLevel {
        match other {
            PaLevel::MIN => NodePaLevel::MIN,
            PaLevel::LOW => NodePaLevel::LOW,
            PaLevel::HIGH => NodePaLevel::HIGH,
            PaLevel::MAX => NodePaLevel::MAX,
        }
    }
}

/// How fast data moves through the air. Units are in bits per second (bps).
#[napi(js_name = "DataRate")]
#[derive(Debug, PartialEq)]
pub enum NodeDataRate {
    /// represents 1 Mbps
    Mbps1,
    /// represents 2 Mbps
    Mbps2,
    /// represents 250 Kbps
    Kbps250,
}

#[cfg(target_os = "linux")]
impl NodeDataRate {
    pub fn into_inner(self) -> DataRate {
        match self {
            NodeDataRate::Mbps1 => DataRate::Mbps1,
            NodeDataRate::Mbps2 => DataRate::Mbps2,
            NodeDataRate::Kbps250 => DataRate::Kbps250,
        }
    }
    pub fn from_inner(other: DataRate) -> NodeDataRate {
        match other {
            DataRate::Mbps1 => NodeDataRate::Mbps1,
            DataRate::Mbps2 => NodeDataRate::Mbps2,
            DataRate::Kbps250 => NodeDataRate::Kbps250,
        }
    }
}

/// The length of a CRC checksum that is used (if any).
///
/// Cyclical Redundancy Checking (CRC) is commonly used to ensure data integrity.
#[napi(js_name = "CrcLength")]
#[derive(Debug, PartialEq)]
pub enum NodeCrcLength {
    /// represents no CRC checksum is used
    DISABLED,
    /// represents CRC 8 bit checksum is used
    BIT8,
    /// represents CRC 16 bit checksum is used
    BIT16,
}

#[cfg(target_os = "linux")]
impl NodeCrcLength {
    pub fn into_inner(self) -> CrcLength {
        match self {
            NodeCrcLength::DISABLED => CrcLength::DISABLED,
            NodeCrcLength::BIT8 => CrcLength::BIT8,
            NodeCrcLength::BIT16 => CrcLength::BIT16,
        }
    }
    pub fn from_inner(other: CrcLength) -> NodeCrcLength {
        match other {
            CrcLength::DISABLED => NodeCrcLength::DISABLED,
            CrcLength::BIT8 => NodeCrcLength::BIT8,
            CrcLength::BIT16 => NodeCrcLength::BIT16,
        }
    }
}

/// The possible states of a FIFO.
#[napi(js_name = "FifoState")]
#[derive(Debug, PartialEq)]
pub enum NodeFifoState {
    /// Represent the state of a FIFO when it is full.
    Full,
    /// Represent the state of a FIFO when it is empty.
    Empty,
    /// Represent the state of a FIFO when it is not full but not empty either.
    Occupied,
}

#[cfg(target_os = "linux")]
impl NodeFifoState {
    pub fn into_inner(self) -> FifoState {
        match self {
            NodeFifoState::Full => FifoState::Full,
            NodeFifoState::Empty => FifoState::Empty,
            NodeFifoState::Occupied => FifoState::Occupied,
        }
    }
    pub fn from_inner(other: FifoState) -> NodeFifoState {
        match other {
            FifoState::Full => NodeFifoState::Full,
            FifoState::Empty => NodeFifoState::Empty,
            FifoState::Occupied => NodeFifoState::Occupied,
        }
    }
}
