#![cfg(target_os = "linux")]
use crate::enums::{PyCrcLength, PyDataRate, PyFifoState, PyPaLevel};
use linux_embedded_hal::{
    gpio_cdev::{chips, LineRequestFlags},
    spidev::{SpiModeFlags, SpidevOptions},
    CdevPin, Delay, SpidevDevice,
};
use pyo3::{
    exceptions::{PyOSError, PyRuntimeError, PyValueError},
    prelude::*,
};
use rf24_rs::radio::{prelude::*, RF24};

#[pyclass(name = "RF24", module = "rf24_py")]
pub struct PyRF24 {
    inner: RF24<SpidevDevice, CdevPin, Delay>,
}

#[pymethods]
impl PyRF24 {
    #[new]
    #[pyo3(
        text_signature = "(ce_pin: int, cs_pin: int, dev_gpio_chip: int = 0, dev_spi_bus: int = 0) -> RF24",
        signature = (ce_pin, cs_pin, dev_gpio_chip = 0u8, dev_spi_bus = 0u8),
    )]
    pub fn new(ce_pin: u32, cs_pin: u8, dev_gpio_chip: u8, dev_spi_bus: u8) -> PyResult<Self> {
        // get the desired "dev/gpiochip{dev_gpio_chip}"
        let mut dev_gpio = chips()
            .map_err(|_| PyOSError::new_err("Failed to get list of GPIO chips for the system"))?
            .find(|chip| {
                if let Ok(chip) = chip {
                    if chip.path().ends_with(dev_gpio_chip.to_string()) {
                        return true;
                    }
                }
                false
            })
            .ok_or(PyOSError::new_err(format!(
                "Could not find specified dev/gpiochip{dev_gpio_chip} for this system."
            )))?
            .map_err(|e| {
                PyOSError::new_err(format!(
                    "Could not open GPIO chip dev/gpiochip{dev_gpio_chip}: {e:?}"
                ))
            })?;
        let ce_line = dev_gpio
            .get_line(ce_pin)
            .map_err(|e| PyValueError::new_err(format!("GPIO{ce_pin} is unavailable: {e:?}")))?;
        let ce_line_handle = ce_line
            .request(LineRequestFlags::OUTPUT, 0, "rf24-rs")
            .map_err(|e| PyOSError::new_err(format!("GPIO{ce_pin} is already in use: {e:?}")))?;
        let ce_pin =
            CdevPin::new(ce_line_handle).map_err(|e| PyOSError::new_err(format!("{e:?}")))?;

        let mut spi =
            SpidevDevice::open(format!("/dev/spidev{dev_spi_bus}.{cs_pin}")).map_err(|_| {
                PyOSError::new_err(format!(
                    "SPI bus {dev_spi_bus} with CS pin option {cs_pin} is not available in this system"
                )
            )
            })?;
        let config = SpidevOptions::new()
            .max_speed_hz(10000000)
            .mode(SpiModeFlags::SPI_MODE_0)
            .bits_per_word(8)
            .build();
        spi.configure(&config)
            .map_err(|e| PyOSError::new_err(format!("{e:?}")))?;

        Ok(Self {
            inner: RF24::new(ce_pin, spi, Delay),
        })
    }

    pub fn begin(&mut self) -> PyResult<()> {
        self.inner
            .init()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    pub fn start_listening(&mut self) -> PyResult<()> {
        self.inner
            .start_listening()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    pub fn stop_listening(&mut self) -> PyResult<()> {
        self.inner
            .stop_listening()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    pub fn send(&mut self, buf: &[u8], ask_no_ack: bool) -> PyResult<bool> {
        self.inner
            .send(buf, ask_no_ack)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    pub fn write(&mut self, buf: &[u8], ask_no_ack: bool, start_tx: bool) -> PyResult<bool> {
        self.inner
            .write(buf, ask_no_ack, start_tx)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    pub fn read(&mut self, len: u8) -> PyResult<Vec<u8>> {
        let mut buf = Vec::with_capacity(len as usize);
        self.inner
            .read(&mut buf, len)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;
        Ok(buf)
    }

    pub fn resend(&mut self) -> PyResult<bool> {
        self.inner
            .resend()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    pub fn rewrite(&mut self) -> PyResult<()> {
        self.inner
            .rewrite()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    pub fn get_last_arc(&mut self) -> PyResult<u8> {
        self.inner
            .get_last_arc()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    pub fn is_plus_variant(&self) -> bool {
        self.inner.is_plus_variant()
    }

    pub fn test_rpd(&mut self) -> PyResult<bool> {
        self.inner
            .test_rpd()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    pub fn start_carrier_wave(&mut self, level: PyPaLevel, channel: u8) -> PyResult<()> {
        self.inner
            .start_carrier_wave(level.into_inner(), channel)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    pub fn stop_carrier_wave(&mut self) -> PyResult<()> {
        self.inner
            .stop_carrier_wave()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    pub fn set_lna(&mut self, enable: bool) -> PyResult<()> {
        self.inner
            .set_lna(enable)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    pub fn allow_ack_payloads(&mut self, enable: bool) -> PyResult<()> {
        self.inner
            .allow_ack_payloads(enable)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }
    pub fn set_auto_ack(&mut self, enable: bool) -> PyResult<()> {
        self.inner
            .set_auto_ack(enable)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }
    pub fn set_auto_ack_pipe(&mut self, enable: bool, pipe: u8) -> PyResult<()> {
        self.inner
            .set_auto_ack_pipe(enable, pipe)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }
    pub fn allow_ask_no_ack(&mut self, enable: bool) -> PyResult<()> {
        self.inner
            .allow_ask_no_ack(enable)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }
    pub fn write_ack_payload(&mut self, pipe: u8, buf: &[u8]) -> PyResult<bool> {
        self.inner
            .write_ack_payload(pipe, buf)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }
    pub fn set_auto_retries(&mut self, delay: u8, count: u8) -> PyResult<()> {
        self.inner
            .set_auto_retries(delay, count)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    pub fn set_channel(&mut self, channel: u8) -> PyResult<()> {
        self.inner
            .set_channel(channel)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    pub fn get_channel(&mut self) -> PyResult<u8> {
        self.inner
            .get_channel()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    pub fn get_crc_length(&mut self) -> PyResult<PyCrcLength> {
        self.inner
            .get_crc_length()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
            .map(|e| PyCrcLength::from_inner(e))
    }

    pub fn set_crc_length(&mut self, crc_length: PyCrcLength) -> PyResult<()> {
        self.inner
            .set_crc_length(crc_length.into_inner())
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }
    pub fn get_data_rate(&mut self) -> PyResult<PyDataRate> {
        self.inner
            .get_data_rate()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
            .map(|e| PyDataRate::from_inner(e))
    }
    pub fn set_data_rate(&mut self, data_rate: PyDataRate) -> PyResult<()> {
        self.inner
            .set_data_rate(data_rate.into_inner())
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    pub fn available(&mut self) -> PyResult<bool> {
        self.inner
            .available()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    pub fn available_pipe(&mut self) -> PyResult<(bool, u8)> {
        let mut pipe = Some(0u8);
        let result = self
            .inner
            .available_pipe(&mut pipe)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;
        Ok((result, pipe.expect("`pipe` should be a number")))
    }

    /// Use this to discard all 3 layers in the radio's RX FIFO.
    pub fn flush_rx(&mut self) -> PyResult<()> {
        self.inner
            .flush_rx()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// Use this to discard all 3 layers in the radio's TX FIFO.
    pub fn flush_tx(&mut self) -> PyResult<()> {
        self.inner
            .flush_tx()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    pub fn get_fifo_state(&mut self, about_tx: bool) -> PyResult<PyFifoState> {
        self.inner
            .get_fifo_state(about_tx)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
            .map(|e| PyFifoState::from_inner(e))
    }

    pub fn get_pa_level(&mut self) -> PyResult<PyPaLevel> {
        self.inner
            .get_pa_level()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
            .map(|e| PyPaLevel::from_inner(e))
    }

    pub fn set_pa_level(&mut self, pa_level: PyPaLevel) -> PyResult<()> {
        self.inner
            .set_pa_level(pa_level.into_inner())
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    pub fn set_payload_length(&mut self, length: u8) -> PyResult<()> {
        self.inner
            .set_payload_length(length)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    pub fn get_payload_length(&mut self) -> PyResult<u8> {
        self.inner
            .get_payload_length()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    pub fn set_dynamic_payloads(&mut self, enable: bool) -> PyResult<()> {
        self.inner
            .set_dynamic_payloads(enable)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    pub fn get_dynamic_payload_length(&mut self) -> PyResult<u8> {
        self.inner
            .get_dynamic_payload_length()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    pub fn open_rx_pipe(&mut self, pipe: u8, address: &[u8]) -> PyResult<()> {
        self.inner
            .open_rx_pipe(pipe, address)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    pub fn open_tx_pipe(&mut self, address: &[u8]) -> PyResult<()> {
        self.inner
            .open_tx_pipe(address)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// If the given `pipe` number is  not in range [0, 5], then this function does nothing.
    pub fn close_rx_pipe(&mut self, pipe: u8) -> PyResult<()> {
        self.inner
            .close_rx_pipe(pipe)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    pub fn set_address_length(&mut self, length: u8) -> PyResult<()> {
        self.inner
            .set_address_length(length)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    pub fn get_address_length(&mut self) -> PyResult<u8> {
        self.inner
            .get_address_length()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    pub fn power_down(&mut self) -> PyResult<()> {
        self.inner
            .power_down()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    #[pyo3(
        text_signature = "(delay: int | None = None) -> None",
        signature = (delay = None),
    )]
    pub fn power_up(&mut self, delay: Option<u32>) -> PyResult<()> {
        self.inner
            .power_up(delay)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    pub fn set_status_flags(&mut self, rx_dr: bool, tx_ds: bool, tx_df: bool) -> PyResult<()> {
        self.inner
            .set_status_flags(rx_dr, tx_ds, tx_df)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    pub fn clear_status_flags(&mut self, rx_dr: bool, tx_ds: bool, tx_df: bool) -> PyResult<()> {
        self.inner
            .clear_status_flags(rx_dr, tx_ds, tx_df)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    pub fn update(&mut self) -> PyResult<()> {
        self.inner
            .update()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    pub fn get_status_flags(&mut self) -> PyResult<(bool, bool, bool)> {
        let mut rx_dr = Some(false);
        let mut tx_ds = Some(false);
        let mut tx_df = Some(false);
        self.inner
            .get_status_flags(&mut rx_dr, &mut tx_ds, &mut tx_df)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;
        Ok((rx_dr.unwrap(), tx_ds.unwrap(), tx_df.unwrap()))
    }
}
