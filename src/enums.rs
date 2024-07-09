/// Power Amplifier level. The units dBm (decibel-milliwatts or dB<sub>mW</sub>)
/// represents a logarithmic signal loss.
pub enum PaLevel {
    /// | nRF24L01 | Si24R1 with<br>lnaEnabled = 1 | Si24R1 with<br>lnaEnabled = 0 |
    /// | :-------:|:-----------------------------:|:-----------------------------:|
    /// | -18 dBm | -6 dBm | -12 dBm |
    MIN,
    /// | nRF24L01 | Si24R1 with<br>lnaEnabled = 1 | Si24R1 with<br>lnaEnabled = 0 |
    /// | :-------:|:-----------------------------:|:-----------------------------:|
    /// | -12 dBm | 0 dBm | -4 dBm |
    LOW,
    /// | nRF24L01 | Si24R1 with<br>lnaEnabled = 1 | Si24R1 with<br>lnaEnabled = 0 |
    /// | :-------:|:-----------------------------:|:-----------------------------:|
    /// | -6 dBm | 3 dBm | 1 dBm |
    HIGH,
    /// | nRF24L01 | Si24R1 with<br>lnaEnabled = 1 | Si24R1 with<br>lnaEnabled = 0 |
    /// | :-------:|:-----------------------------:|:-----------------------------:|
    /// | 0 dBm | 7 dBm | 4 dBm |
    MAX,
}

/// How fast data moves through the air. Units are in bits per second (bps).
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
pub enum CrcLength {
    /// represents no CRC checksum is used
    DISABLED,
    /// represents CRC 8 bit checksum is used
    BIT8,
    /// represents CRC 16 bit checksum is used
    BIT16,
}

/// The possible states of a FIFO.
pub enum FifoState {
    /// Represent the state of a FIFO when it is full.
    Full,
    /// Represent the state of a FIFO when it is empty.
    Empty,
    /// Represent the state of a FIFO when it is not full but not empty either.
    Occupied,
}
