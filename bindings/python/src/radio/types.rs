use pyo3::prelude::*;

/// The radio's status flags that correspond to interrupt events.
///
/// See:
///     - [`RF24.get_status_flags()`][rf24_py.RF24.get_status_flags]
///     - [`RF24.set_status_flags()`][rf24_py.RF24.set_status_flags]
///     - [`RF24.clear_status_flags()`][rf24_py.RF24.clear_status_flags]
///     - [`RF24.update()`][rf24_py.RF24.update]
#[pyclass(frozen, get_all, module = "rf24_py")]
#[derive(Default, Clone)]
pub struct StatusFlags {
    /// A flag to describe if RX Data Ready to read.
    pub rx_dr: bool,
    /// A flag to describe if TX Data Sent.
    pub tx_ds: bool,
    /// A flag to describe if TX Data Failed.
    pub tx_df: bool,
}

#[pymethods]
impl StatusFlags {
    #[new]
    #[pyo3(
        signature = (rx_dr = 0i32, tx_ds = 0i32, tx_df = 0i32),
        text_signature = "(rx_dr: bool = False, tx_ds: bool = False, tx_df: bool = False) -> StatusFlags",
    )]
    fn new(rx_dr: i32, tx_ds: i32, tx_df: i32) -> Self {
        Self {
            rx_dr: rx_dr != 0,
            tx_ds: tx_ds != 0,
            tx_df: tx_df != 0,
        }
    }

    pub fn __repr__(&self) -> String {
        format!(
            "<StatusFlags rx_dr: {}, tx_ds: {}, tx_df: {}>",
            self.rx_dr, self.tx_ds, self.tx_df
        )
    }
}

impl StatusFlags {
    pub fn into_inner(self) -> rf24::StatusFlags {
        rf24::StatusFlags::new()
            .with_rx_dr(self.rx_dr)
            .with_tx_ds(self.tx_ds)
            .with_tx_df(self.tx_df)
    }

    pub fn from_inner(other: rf24::StatusFlags) -> Self {
        Self {
            rx_dr: other.rx_dr(),
            tx_ds: other.tx_ds(),
            tx_df: other.tx_df(),
        }
    }
}

/// Power Amplifier level. The units dBm (decibel-milliwatts or dB<sub>mW</sub>)
/// represents a logarithmic signal loss.
///
/// Attributes:
///     Min:
///         | nRF24L01 | Si24R1 with<br>LNA Enabled | Si24R1 with<br>LNA Disabled |
///         | :-------:|:--------------------------:|:---------------------------:|
///         | -18 dBm | -6 dBm | -12 dBm |
///     Low:
///         | nRF24L01 | Si24R1 with<br>LNA Enabled | Si24R1 with<br>LNA Disabled |
///         | :-------:|:--------------------------:|:---------------------------:|
///         | -12 dBm | 0 dBm | -4 dBm |
///     High:
///         | nRF24L01 | Si24R1 with<br>LNA Enabled | Si24R1 with<br>LNA Disabled |
///         | :-------:|:--------------------------:|:---------------------------:|
///         | -6 dBm | 3 dBm | 1 dBm |
///     Max:
///         | nRF24L01 | Si24R1 with<br>LNA Enabled | Si24R1 with<br>LNA Disabled |
///         | :-------:|:--------------------------:|:---------------------------:|
///         | 0 dBm | 7 dBm | 4 dBm |
#[pyclass(eq, eq_int, module = "rf24_py")]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PaLevel {
    Min,
    Low,
    High,
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
///
/// Attributes:
///     Mbps1: Represents 1 Mbps
///     Mbps2: Represents 2 Mbps
///     Kbps250: Represents 250 Kbps
#[pyclass(eq, eq_int, module = "rf24_py")]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DataRate {
    Mbps1,
    Mbps2,
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
///
/// Attributes:
///     Disabled: Represents no CRC checksum is used.
///     Bit8: Represents CRC 8 bit checksum is used.
///     Bit16: Represents CRC 16 bit checksum is used.
#[pyclass(name = "CrcLength", eq, eq_int, module = "rf24_py")]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CrcLength {
    Disabled,
    Bit8,
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

/// Enumerations to describe the possible states of a FIFO.
///
/// See also:
///     - [`RF24.get_fifo_state()`][rf24_py.RF24.get_fifo_state]
///
/// Attributes:
///     Full: Represent the state of a FIFO when it is full.
///     Empty: Represent the state of a FIFO when it is empty.
///     Occupied: Represent the state of a FIFO when it is not full but not empty either.
#[pyclass(eq, eq_int, module = "rf24_py")]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FifoState {
    Full,
    Empty,
    Occupied,
}

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
