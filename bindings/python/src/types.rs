use pyo3::prelude::*;

#[cfg(target_os = "linux")]
use rf24::{CrcLength, DataRate, FifoState, PaLevel, StatusFlags};

#[pyclass(name = "StatusFlags", frozen, get_all, module = "rf24_py")]
#[derive(Default, Clone)]
pub struct PyStatusFlags {
    /// A flag to describe if RX Data Ready to read.
    pub rx_dr: bool,
    /// A flag to describe if TX Data Sent.
    pub tx_ds: bool,
    /// A flag to describe if TX Data Failed.
    pub tx_df: bool,
}

#[pymethods]
impl PyStatusFlags {
    #[new]
    #[pyo3(signature = (rx_dr = false, tx_ds = false, tx_df = false))]
    fn new(rx_dr: bool, tx_ds: bool, tx_df: bool) -> Self {
        Self {
            rx_dr,
            tx_ds,
            tx_df,
        }
    }

    pub fn __repr__(&self) -> String {
        format!(
            "<StatusFlags rx_dr: {}, tx_ds: {}, tx_df: {}>",
            self.rx_dr, self.tx_ds, self.tx_df
        )
    }
}

#[cfg(target_os = "linux")]
impl PyStatusFlags {
    pub fn into_inner(self) -> StatusFlags {
        StatusFlags {
            rx_dr: self.rx_dr,
            tx_ds: self.tx_ds,
            tx_df: self.tx_df,
        }
    }

    pub fn from_inner(other: StatusFlags) -> Self {
        Self {
            rx_dr: other.rx_dr,
            tx_ds: other.tx_ds,
            tx_df: other.tx_df,
        }
    }
}

/// Power Amplifier level. The units dBm (decibel-milliwatts or dB<sub>mW</sub>)
/// represents a logarithmic signal loss.
#[pyclass(name = "PaLevel", eq, eq_int, module = "rf24_py")]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PyPaLevel {
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

#[cfg(target_os = "linux")]
impl PyPaLevel {
    pub fn into_inner(self) -> PaLevel {
        match self {
            PyPaLevel::Min => PaLevel::Min,
            PyPaLevel::Low => PaLevel::Low,
            PyPaLevel::High => PaLevel::High,
            PyPaLevel::Max => PaLevel::Max,
        }
    }
    pub fn from_inner(other: PaLevel) -> PyPaLevel {
        match other {
            PaLevel::Min => PyPaLevel::Min,
            PaLevel::Low => PyPaLevel::Low,
            PaLevel::High => PyPaLevel::High,
            PaLevel::Max => PyPaLevel::Max,
        }
    }
}

/// How fast data moves through the air. Units are in bits per second (bps).
#[pyclass(name = "DataRate", eq, eq_int, module = "rf24_py")]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PyDataRate {
    /// represents 1 Mbps
    Mbps1,
    /// represents 2 Mbps
    Mbps2,
    /// represents 250 Kbps
    Kbps250,
}

#[cfg(target_os = "linux")]
impl PyDataRate {
    pub fn into_inner(self) -> DataRate {
        match self {
            PyDataRate::Mbps1 => DataRate::Mbps1,
            PyDataRate::Mbps2 => DataRate::Mbps2,
            PyDataRate::Kbps250 => DataRate::Kbps250,
        }
    }
    pub fn from_inner(other: DataRate) -> PyDataRate {
        match other {
            DataRate::Mbps1 => PyDataRate::Mbps1,
            DataRate::Mbps2 => PyDataRate::Mbps2,
            DataRate::Kbps250 => PyDataRate::Kbps250,
        }
    }
}

/// The length of a CRC checksum that is used (if any).
///
/// Cyclical Redundancy Checking (CRC) is commonly used to ensure data integrity.
#[pyclass(name = "CrcLength", eq, eq_int, module = "rf24_py")]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PyCrcLength {
    /// represents no CRC checksum is used
    Disabled,
    /// represents CRC 8 bit checksum is used
    Bit8,
    /// represents CRC 16 bit checksum is used
    Bit16,
}

#[cfg(target_os = "linux")]
impl PyCrcLength {
    pub fn into_inner(self) -> CrcLength {
        match self {
            PyCrcLength::Disabled => CrcLength::Disabled,
            PyCrcLength::Bit8 => CrcLength::Bit8,
            PyCrcLength::Bit16 => CrcLength::Bit16,
        }
    }
    pub fn from_inner(other: CrcLength) -> PyCrcLength {
        match other {
            CrcLength::Disabled => PyCrcLength::Disabled,
            CrcLength::Bit8 => PyCrcLength::Bit8,
            CrcLength::Bit16 => PyCrcLength::Bit16,
        }
    }
}

/// The possible states of a FIFO.
#[pyclass(name = "FifoState", eq, eq_int, module = "rf24_py")]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PyFifoState {
    /// Represent the state of a FIFO when it is full.
    Full,
    /// Represent the state of a FIFO when it is empty.
    Empty,
    /// Represent the state of a FIFO when it is not full but not empty either.
    Occupied,
}

#[cfg(target_os = "linux")]
impl PyFifoState {
    pub fn into_inner(self) -> FifoState {
        match self {
            PyFifoState::Full => FifoState::Full,
            PyFifoState::Empty => FifoState::Empty,
            PyFifoState::Occupied => FifoState::Occupied,
        }
    }
    pub fn from_inner(other: FifoState) -> PyFifoState {
        match other {
            FifoState::Full => PyFifoState::Full,
            FifoState::Empty => PyFifoState::Empty,
            FifoState::Occupied => PyFifoState::Occupied,
        }
    }
}
