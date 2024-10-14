/// Power Amplifier level. The units dBm (decibel-milliwatts or dB<sub>mW</sub>)
/// represents a logarithmic signal loss.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PaLevel {
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

/// The length of a CRC checksum that is used (if any).
///
/// Cyclical Redundancy Checking (CRC) is commonly used to ensure data integrity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CrcLength {
    /// represents no CRC checksum is used
    DISABLED,
    /// represents CRC 8 bit checksum is used
    BIT8,
    /// represents CRC 16 bit checksum is used
    BIT16,
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

#[derive(Clone, Copy, Debug, Default)]
/// A struct used to describe the different interruptable events.
pub struct StatusFlags {
    /// A flag to describe if RX Data Ready to read.
    pub rx_dr: bool,
    /// A flag to describe if TX Data Sent.
    pub tx_ds: bool,
    /// A flag to describe if TX Data Failed.
    pub tx_df: bool,
}
