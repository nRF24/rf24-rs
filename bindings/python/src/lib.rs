use pyo3::prelude::*;
mod fake_ble;
mod radio;

#[cfg(target_os = "linux")]
fn bind_radio_impl(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<radio::interface::RF24>()?;
    m.add_class::<fake_ble::radio::FakeBle>()?;
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn bind_radio_impl(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}

/// A Python module implemented in Rust.
#[pymodule]
fn rf24_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    bind_radio_impl(m)?;
    m.add_class::<radio::types::CrcLength>()?;
    m.add_class::<radio::types::DataRate>()?;
    m.add_class::<radio::types::FifoState>()?;
    m.add_class::<radio::types::PaLevel>()?;
    m.add_class::<radio::types::StatusFlags>()?;
    m.add_class::<radio::config::RadioConfig>()?;
    m.add_class::<fake_ble::services::BatteryService>()?;
    m.add_class::<fake_ble::services::TemperatureService>()?;
    m.add_class::<fake_ble::services::UrlService>()?;
    m.add_class::<fake_ble::services::BlePayload>()?;
    m.add_function(wrap_pyfunction!(fake_ble::ble_config, m)?)?;
    Ok(())
}
