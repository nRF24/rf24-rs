#[cfg(target_os = "linux")]
use rf24::{CrcLength, DataRate, FifoState, PaLevel, StatusFlags};

/// Optional configuration parameters to fine tune instantiating the RF24 object.
/// Pass this object as third parameter to RF24 constructor.
#[napi(object)]
pub struct HardwareConfig {
    /// The GPIO chip number: `/dev/gpiochipN` where `N` is this value.
    ///
    /// Defaults to `0`, but needs to be `4` on RPi5 (or newer).
    /// This may also need to be specified for nVidia's hardware offerings.
    pub dev_gpio_chip: Option<u8>,

    /// The SPI bus number: `/dev/spidevX.Y` where `X` is this value
    /// and `Y` is the `csPin` required parameter to RF24 constructor
    ///
    /// Defaults to 0, but can be as high as 3 depending on the number of
    /// SPI buses available/exposed on the board.
    pub dev_spi_bus: Option<u8>,

    /// The SPI speed in Hz used to communicate with the nRF24L01 over SPI.
    ///
    /// Defaults to 10 MHz (`10000000`) which is the radio's maximum
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

/// The return type for `RF24.getStatusFlags()` and optional parameters for
/// `RF24.setStatusFlags()` and `RF24.clearStatusFlags()`.
///
/// These flags default to `true` if not specified for `RF24.setStatusFlags()`
/// or `RF24.clearStatusFlags()`.
#[napi(object, js_name = "StatusFlags")]
#[derive(Default)]
pub struct NodeStatusFlags {
    /// A flag to describe if RX Data Ready to read.
    pub rx_dr: Option<bool>,
    /// A flag to describe if TX Data Sent.
    pub tx_ds: Option<bool>,
    /// A flag to describe if TX Data Failed.
    pub tx_df: Option<bool>,
}

#[cfg(target_os = "linux")]
impl NodeStatusFlags {
    pub fn into_inner(self) -> StatusFlags {
        StatusFlags {
            rx_dr: self.rx_dr.unwrap_or_default(),
            tx_ds: self.tx_ds.unwrap_or_default(),
            tx_df: self.tx_df.unwrap_or_default(),
        }
    }

    pub fn from_inner(other: StatusFlags) -> Self {
        Self {
            rx_dr: Some(other.rx_dr),
            tx_ds: Some(other.tx_ds),
            tx_df: Some(other.tx_df),
        }
    }
}

/// An optional configuration for `RF24.write()`
#[napi(object)]
pub struct WriteConfig {
    /// Set to true if you want to disable auto-ACK feature for the individual
    /// payload (required `buf` parameter to `RF24.write()`).
    ///
    /// Defaults to false. Be sure to invoke `RF24.allowAskNoAck(true)` at least once beforehand,
    /// otherwise this option will have no affect at all.
    pub ask_no_ack: Option<bool>,

    /// Set to true to assert the radio's CE pin (and begin active TX mode) after the payload is
    /// uploaded to the TX FIFO.
    ///
    /// Only set this to false if filling the TX FIFO (maximum 3 level stack) before entering
    /// active TX mode. Setting this option to false does not deactivate the radio's CE pin.
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

/// The return type for `RF24.availablePipe()`
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
#[napi(js_name = "PaLevel")]
#[derive(Debug, PartialEq)]
pub enum NodePaLevel {
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

#[cfg(target_os = "linux")]
impl NodePaLevel {
    pub fn into_inner(self) -> PaLevel {
        match self {
            NodePaLevel::Min => PaLevel::Min,
            NodePaLevel::Low => PaLevel::Low,
            NodePaLevel::High => PaLevel::High,
            NodePaLevel::Max => PaLevel::Max,
        }
    }
    pub fn from_inner(other: PaLevel) -> NodePaLevel {
        match other {
            PaLevel::Min => NodePaLevel::Min,
            PaLevel::Low => NodePaLevel::Low,
            PaLevel::High => NodePaLevel::High,
            PaLevel::Max => NodePaLevel::Max,
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
    Disabled,
    /// represents CRC 8 bit checksum is used
    Bit8,
    /// represents CRC 16 bit checksum is used
    Bit16,
}

#[cfg(target_os = "linux")]
impl NodeCrcLength {
    pub fn into_inner(self) -> CrcLength {
        match self {
            NodeCrcLength::Disabled => CrcLength::Disabled,
            NodeCrcLength::Bit8 => CrcLength::Bit8,
            NodeCrcLength::Bit16 => CrcLength::Bit16,
        }
    }
    pub fn from_inner(other: CrcLength) -> NodeCrcLength {
        match other {
            CrcLength::Disabled => NodeCrcLength::Disabled,
            CrcLength::Bit8 => NodeCrcLength::Bit8,
            CrcLength::Bit16 => NodeCrcLength::Bit16,
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
