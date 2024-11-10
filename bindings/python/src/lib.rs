use pyo3::prelude::*;
mod config;
mod radio;
mod types;

#[cfg(target_os = "linux")]
fn bind_radio_impl(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<radio::RF24>()?;
    m.add_class::<config::RadioConfig>()
}

#[cfg(not(target_os = "linux"))]
fn bind_radio_impl(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}

/// A Python module implemented in Rust.
#[pymodule]
fn rf24_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    bind_radio_impl(m)?;
    m.add_class::<types::CrcLength>()?;
    m.add_class::<types::DataRate>()?;
    m.add_class::<types::FifoState>()?;
    m.add_class::<types::PaLevel>()?;
    m.add_class::<types::StatusFlags>()?;
    Ok(())
}
