use pyo3::prelude::*;
// #[cfg(target_os = "linux")]
mod enums;
mod radio;

#[cfg(target_os = "linux")]
fn bind_radio_impl(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<radio::PyRF24>()
}

#[cfg(not(target_os = "linux"))]
fn bind_radio_impl(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}

/// A Python module implemented in Rust.
#[pymodule]
fn rf24_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    bind_radio_impl(m)?;
    m.add_class::<enums::PyCrcLength>()?;
    m.add_class::<enums::PyDataRate>()?;
    m.add_class::<enums::PyFifoState>()?;
    m.add_class::<enums::PyPaLevel>()?;
    m.add_class::<enums::PyStatusFlags>()?;
    Ok(())
}
