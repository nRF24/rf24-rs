#![cfg(target_os = "linux")]
use std::borrow::Cow;
use std::time::Duration;

use super::config::RadioConfig;
use super::types::{CrcLength, DataRate, FifoState, PaLevel, StatusFlags};
use embedded_hal::{delay::DelayNs, digital::OutputPin};
use linux_embedded_hal::{
    gpio_cdev::{chips, LineRequestFlags},
    spidev::{SpiModeFlags, SpidevOptions},
    CdevPin, SpidevDevice,
};
use nix::sys::time::TimeSpec;
use nix::time::{clock_nanosleep, ClockId, ClockNanosleepFlags};

use pyo3::{
    exceptions::{PyOSError, PyRuntimeError, PyValueError},
    prelude::*,
};
use rf24::radio::prelude::*;

struct Delay;

impl DelayNs for Delay {
    fn delay_ns(&mut self, ns: u32) {
        clock_nanosleep(
            ClockId::CLOCK_REALTIME,
            ClockNanosleepFlags::empty(),
            &TimeSpec::from_duration(Duration::from_nanos(ns as u64)),
        )
        .unwrap_or_else(|e| panic!("delay_ns({ns}) failed. {e:?}"));
    }
}

/// Construct an object to control the radio.
///
/// Parameters:
///     ce_pin: The GPIO pin number connected to the radio's CE pin.
///     cs_pin: The identifying number for the SPI bus' CS pin;
///         also labeled as "CEx" (where "x" is this parameter's value) on many
///         Raspberry Pi pin diagrams.
///
/// Other parameters:
///     dev_gpio_chip: The GPIO chip's identifying number.
///         Consider the path `/dev/gpiochipN` where `N` is this parameter's value.
///     dev_spi_bus: The SPI bus number.
///         Consider the path `/dev/spidevX.Y` where `X` is this parameter's value
///         and `Y` is the `cs_pin` parameter's value.
///     spi_speed: The SPI bus speed in Hz. Defaults to the radio's maximum supported
///         speed (10 MHz).
#[pyclass(module = "rf24_py")]
pub struct RF24 {
    inner: rf24::radio::RF24<SpidevDevice, CdevPin, Delay>,
    read_buf: [u8; 32],
}

#[pymethods]
impl RF24 {
    #[new]
    #[pyo3(
        text_signature = "(ce_pin: int, cs_pin: int, dev_gpio_chip: int = 0, dev_spi_bus: int = 0, spi_speed: int = 10000000) -> RF24",
        signature = (ce_pin, cs_pin, dev_gpio_chip = 0u8, dev_spi_bus = 0u8, spi_speed = 10_000_000),
    )]
    pub fn new(
        ce_pin: u32,
        cs_pin: u8,
        dev_gpio_chip: u8,
        dev_spi_bus: u8,
        spi_speed: u32,
    ) -> PyResult<Self> {
        // get the desired "/dev/gpiochip{dev_gpio_chip}"
        let mut dev_gpio = chips()
            .map_err(|_| PyOSError::new_err("Failed to get list of GPIO chips for the system"))?
            .find(|chip| {
                if let Ok(chip) = chip {
                    if chip
                        .path()
                        .to_string_lossy()
                        .ends_with(&dev_gpio_chip.to_string())
                    {
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
        let spi_config = SpidevOptions::new()
            .max_speed_hz(spi_speed)
            .mode(SpiModeFlags::SPI_MODE_0)
            .bits_per_word(8)
            .build();
        spi.configure(&spi_config)
            .map_err(|e| PyOSError::new_err(format!("{e:?}")))?;

        Ok(Self {
            inner: rf24::radio::RF24::new(ce_pin, spi, Delay),
            read_buf: [0u8; 32],
        })
    }

    /// Initialize the radio on the configured hardware (as specified to
    /// [`RF24`][rf24_py.RF24] constructor).
    ///
    /// This is the same as [`with_config(RadioConfig())`][rf24_py.RF24.with_config],
    /// but this function also
    /// - detects if the radio is a plus variant ([`is_plus_variant`][rf24_py.RF24.is_plus_variant])
    /// - checks for data corruption across the SPI lines (MOSI, MISO, SCLK)
    ///
    /// Raises:
    ///     RuntimeError: If a hardware failure caused problems (includes a
    ///         message to describe what problem was detected).
    pub fn begin(&mut self) -> PyResult<()> {
        self.inner
            .init()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// Reconfigure the radio with a specified [`RadioConfig`][rf24_py.RadioConfig].
    ///
    /// Warning:
    ///     It is strongly encouraged to call [`RF24.begin()`][rf24_py.RF24.begin]
    ///     after constructing the RF24 object.
    ///
    ///     Only use this function subsequently to quickly switch between different
    ///     network settings.
    pub fn with_config(&mut self, config: &RadioConfig) -> PyResult<()> {
        self.inner
            .with_config(config.get_inner())
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// Set the radio's CE pin HIGH (`True`) or LOW (`False`).
    ///
    /// This is only exposed for advanced use of TX FIFO during
    /// asynchronous TX operations. It is highly encouraged to use
    /// [`as_rx()`][rf24_py.RF24.as_rx] or [`as_tx()`][rf24_py.RF24.as_tx]
    /// to ensure proper radio behavior when entering RX or TX mode.
    pub fn ce_pin(&mut self, value: i32) -> PyResult<()> {
        if value != 0 {
            self.inner.ce_pin.set_high()
        } else {
            self.inner.ce_pin.set_low()
        }
        .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    #[getter]
    pub fn is_rx(&self) -> bool {
        self.inner.is_rx()
    }

    /// Put the radio into active RX mode.
    pub fn as_rx(&mut self) -> PyResult<()> {
        self.inner
            .as_rx()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// Deactivates active RX mode and puts the radio into an inactive TX mode.
    ///
    /// The datasheet recommends idling the radio in an inactive TX mode.
    ///
    /// Note:
    ///     This function will also flush the TX FIFO when ACK payloads are enabled
    ///     (via [`RF24.ack_payloads`][rf24_py.RF24.ack_payloads]).
    pub fn as_tx(&mut self) -> PyResult<()> {
        self.inner
            .as_tx()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// Blocking function that loads a given `buf` into the TX FIFO, waits for a response
    /// (if auto-ack is enabled), then returns a Boolean describing success.
    ///
    /// Parameters:
    ///     buf: The buffer of bytes to transmit.
    ///
    /// Other parameters:
    ///     ask_no_ack: A flag to disable the auto-ack feature for the given payload in `buf`.
    ///         This has no effect if auto-ack is disabled or
    ///         [`RF24.allow_ask_no_ack`][rf24_py.RF24.allow_ask_no_ack] is not enabled.
    #[pyo3(
        signature = (buf, ask_no_ack = 0i32),
        text_signature = "(buf: bytes | bytearray, ask_no_ack: bool | int = False) -> bool",
    )]
    pub fn send(&mut self, buf: &[u8], ask_no_ack: i32) -> PyResult<bool> {
        self.inner
            .send(buf, ask_no_ack != 0)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// A non-blocking function that uploads a given `buf` to the radio's TX FIFO.
    ///
    /// This is a helper function to [`RF24.send()`][rf24_py.RF24.send].
    /// Use this in combination with [`RF24.update()`][rf24_py.RF24.update] and
    /// [`RF24.get_status_flags()`][rf24_py.RF24.get_status_flags]
    /// to determine if transmission was successful.
    ///
    /// Parameters:
    ///     buf: The buffer of bytes to load into the TX FIFO.
    ///
    /// Other parameters:
    ///     ask_no_ack: A flag to disable the auto-ack feature for the given payload in `buf`.
    ///
    ///         This has no effect if auto-ack is disabled or [RF24.allow_ask_no_ack] is not
    ///         enabled.
    ///     start_tx: A flag to assert the radio's CE pin after the given `buf` is uploaded to
    ///         the RX FIFO. Setting this to false does not un-assert the radio's CE pin to LOW.
    ///
    /// Returns:
    ///     A Boolean that describes if the given `buf` was successfully loaded into the TX FIFO.
    #[pyo3(
        signature = (buf, ask_no_ack = 0i32, start_tx = 1i32),
        text_signature = "(buf: bytes | bytearray, ask_no_ack: bool | int = False, start_tx: bool | int = True) -> bool",
    )]
    pub fn write(&mut self, buf: &[u8], ask_no_ack: i32, start_tx: i32) -> PyResult<bool> {
        self.inner
            .write(buf, ask_no_ack != 0, start_tx != 0)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// Read data from the radio's RX FIFO.
    ///
    /// Use [`RF24.available()`][rf24_py.RF24.available] to determine if there is data ready
    /// to read from the RX FIFO.
    ///
    /// Other parameters:
    ///     len: An optional number of bytes to read from the FIFO. This is capped at `32`.
    ///         If not specified, then the length of the next available payload is used (which
    ///         automatically respects if dynamic payloads are enabled).
    ///
    /// See also:
    ///     [`RF24.dynamic_payloads`][rf24_py.RF24.dynamic_payloads] for dynamically
    ///     sized payload or [`RF24.payload_length`][rf24_py.RF24.payload_length] for
    ///     statically sized payloads.
    #[pyo3(signature = (len = None))]
    pub fn read(&mut self, len: Option<u8>) -> PyResult<Cow<[u8]>> {
        let len = self
            .inner
            .read(&mut self.read_buf, len)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;
        Ok(Cow::from(&self.read_buf[0..len as usize]))
    }

    /// A blocking function to resend a failed payload in the TX FIFO.
    ///
    /// This is similar to [`RF24.send()`][rf24_py.RF24.send] but specifically for
    /// failed transmissions.
    pub fn resend(&mut self) -> PyResult<bool> {
        self.inner
            .resend()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// A non-blocking function to restart a failed transmission.
    ///
    /// This is a helper function to [`RF24.resend()`][rf24_py.RF24.resend].
    /// Use [`RF24.update()`][rf24_py.RF24.update] and
    /// [`RF24.get_status_flags()`][rf24_py.RF24.get_status_flags] to determine if
    /// retransmission was successful.
    pub fn rewrite(&mut self) -> PyResult<()> {
        self.inner
            .rewrite()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// Get the Automatic Retry Count (ARC) of attempts made during the last transmission.
    ///
    /// This resets with every new transmission. The returned value is meaningless if the
    /// auto-ack feature is disabled.
    ///
    /// See also:
    ///     Use [`RF24.set_auto_retries`][rf24_py.RF24.set_auto_retries] to configure the
    ///     automatic retries feature.
    pub fn get_last_arc(&mut self) -> PyResult<u8> {
        self.inner
            .get_last_arc()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// A property that describes if the radio is a nRF24L01+ or not.
    #[getter]
    pub fn is_plus_variant(&self) -> bool {
        self.inner.is_plus_variant()
    }

    /// A property that describes the radio's Received Power Detection (RPD).
    ///
    /// This is reset upon entering RX mode and is only set if the radio detects a
    /// signal if strength -64 dBm or greater (actual threshold may vary depending
    /// on radio model).
    #[getter]
    pub fn get_rpd(&mut self) -> PyResult<bool> {
        self.inner
            .rpd()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// Start a constant carrier wave on the given `channel` using the specified
    /// power amplitude `level`.
    ///
    /// This functionality is only useful for testing the radio hardware works as a
    /// transmitter.
    ///
    /// Parameters:
    ///     level: The Power Amplitude level to use when transmitting.
    ///     channel: The channel (radio's frequency) used to transmit.
    ///         The channel should not be changed while transmitting because it can
    ///         cause undefined behavior.
    pub fn start_carrier_wave(&mut self, level: PaLevel, channel: u8) -> PyResult<()> {
        self.inner
            .start_carrier_wave(level.into_inner(), channel)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// Stop transmitting the constant carrier wave.
    ///
    /// [`RF24.start_carrier_wave()`][rf24_py.RF24.start_carrier_wave] should be called
    /// before this function.
    pub fn stop_carrier_wave(&mut self) -> PyResult<()> {
        self.inner
            .stop_carrier_wave()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// Enable or disable the LNA feature.
    ///
    /// On nRF24L01+ modules with a builtin antenna, this feature is always enabled.
    /// For clone's and module's with a separate PA/LNA circuit (external antenna),
    /// this function may not behave exactly as expected. Consult the radio module's
    /// manufacturer.
    pub fn set_lna(&mut self, enable: i32) -> PyResult<()> {
        self.inner
            .set_lna(enable != 0)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// Enable or disable the custom ACK payloads attached to auto-ack packets.
    ///
    /// > [!IMPORTANT]
    /// > This feature requires dynamically sized payloads.
    /// > This attribute will enable [`dynamic_payloads`][rf24_py.RF24.dynamic_payloads]
    /// > automatically when needed. This attribute will not disable
    /// > [`dynamic_payloads`][rf24_py.RF24.dynamic_payloads].
    #[setter]
    pub fn set_ack_payloads(&mut self, enable: i32) -> PyResult<()> {
        self.inner
            .set_ack_payloads(enable != 0)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    #[getter]
    pub fn get_ack_payloads(&self) -> bool {
        self.inner.get_ack_payloads()
    }

    /// Enable or disable the auto-ack feature for all pipes.
    ///
    /// Note:
    ///     This feature requires CRC to be enabled.
    ///     See [`RF24.crc_length`][rf24_py.RF24.crc_length] for more detail.
    ///
    /// Parameters:
    ///     enable: Pass true to enable the auto-ack feature for all pipes.
    pub fn set_auto_ack(&mut self, enable: i32) -> PyResult<()> {
        self.inner
            .set_auto_ack(enable != 0)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// Enable or disable the auto-ack feature for a specified `pipe`.
    ///
    /// Note:
    ///     This feature requires CRC to be enabled.
    ///     See [`RF24.crc_length`][rf24_py.RF24.crc_length] for more detail.
    ///
    /// Parameters:
    ///     enable: Pass true to enable the auto-ack feature for the specified `pipe`.
    ///     pipe: The pipe about which to control the auto-ack feature.
    pub fn set_auto_ack_pipe(&mut self, enable: i32, pipe: u8) -> PyResult<()> {
        self.inner
            .set_auto_ack_pipe(enable != 0, pipe)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// Allow disabling the auto-ack feature for individual payloads.
    ///
    /// Parameters:
    ///     enable: Setting this to `true` will allow the `ask_no_ack` parameter to
    ///         take effect. See [`RF24.send()`][rf24_py.RF24.send] and
    ///         [`RF24.write()`][rf24_py.RF24.write] for more detail.
    pub fn allow_ask_no_ack(&mut self, enable: i32) -> PyResult<()> {
        self.inner
            .allow_ask_no_ack(enable != 0)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// Upload a given ACK packet's payload (`buf`) into the radio's TX FIFO.
    ///
    /// This feature requires
    /// [`RF24.ack_payloads`][rf24_py.RF24.ack_payloads] to be enabled.
    ///
    /// Parameters:
    ///     pipe: The pipe number that (when data is received) will be responded
    ///         with the given payload (`buf`).
    ///     buf: The payload to attach to the auto-ack packet when responding to
    ///         data received on specified `pipe`.
    ///
    /// Returns:
    ///     A boolean value that describes if the payload was successfully uploaded
    ///         to the TX FIFO. Remember, the TX FIFO only has 3 levels ("slots").
    pub fn write_ack_payload(&mut self, pipe: u8, buf: &[u8]) -> PyResult<bool> {
        self.inner
            .write_ack_payload(pipe, buf)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// Configure the automatic retry feature.
    ///
    /// This feature is part of the auto-ack feature, thus the auto-ack feature is
    /// required for this function to have any effect.
    ///
    /// Parameters:
    ///     delay: This value is clamped to the range [0, 15]. This value is
    ///         translated to microseconds with the formula
    ///
    ///             250 + (delay * 250) = microseconds
    ///
    ///         Meaning, the effective range of `delay` is [250, 4000].
    ///     count: The number of attempt to retransmit when no ACK packet was
    ///         received (after transmitting). This value is clamped to the range [0, 15].
    pub fn set_auto_retries(&mut self, delay: u8, count: u8) -> PyResult<()> {
        self.inner
            .set_auto_retries(delay, count)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// Set/get the channel (frequency) that the radio uses to transmit and receive.
    ///
    /// This value is clamped to the range [0, 125].
    ///
    /// The channel can be roughly translated into a frequency with the formula:
    /// ```text
    /// frequency (in Hz) = channel + 2400
    /// ```
    #[setter]
    pub fn set_channel(&mut self, channel: u8) -> PyResult<()> {
        self.inner
            .set_channel(channel)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    #[getter]
    pub fn get_channel(&mut self) -> PyResult<u8> {
        self.inner
            .get_channel()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// Set/get the [`CrcLength`][rf24_py.CrcLength] used for all outgoing and incoming
    /// transmissions.
    ///
    /// > [!IMPORTANT]
    /// > Because CRC is required for the auto-ack feature, the radio's firmware will
    /// > forcefully enable CRC even if the user explicitly disables it.
    #[setter]
    pub fn set_crc_length(&mut self, crc_length: CrcLength) -> PyResult<()> {
        self.inner
            .set_crc_length(crc_length.into_inner())
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    #[getter]
    pub fn get_crc_length(&mut self) -> PyResult<CrcLength> {
        self.inner
            .get_crc_length()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
            .map(CrcLength::from_inner)
    }

    /// Set the [`DataRate`][rf24_py.DataRate] used for all incoming and outgoing
    /// transmissions.
    #[setter]
    pub fn set_data_rate(&mut self, data_rate: DataRate) -> PyResult<()> {
        self.inner
            .set_data_rate(data_rate.into_inner())
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    #[getter]
    pub fn get_data_rate(&mut self) -> PyResult<DataRate> {
        self.inner
            .get_data_rate()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
            .map(DataRate::from_inner)
    }

    /// Is there a payload available in the RX FIFO?
    ///
    /// Use [`RF24.read()`][rf24_py.RF24.read] to get the payload data.
    pub fn available(&mut self) -> PyResult<bool> {
        self.inner
            .available()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// Similar to [`RF24.available()`][rf24_py.RF24.available] but also returns the
    /// pipe that received the next available payload.
    pub fn available_pipe(&mut self) -> PyResult<(bool, u8)> {
        let mut pipe = 15;
        let result = self
            .inner
            .available_pipe(&mut pipe)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))?;
        Ok((result, pipe))
    }

    /// Discard all 3 layers in the radio's RX FIFO.
    pub fn flush_rx(&mut self) -> PyResult<()> {
        self.inner
            .flush_rx()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// Discard all 3 layers in the radio's TX FIFO.
    pub fn flush_tx(&mut self) -> PyResult<()> {
        self.inner
            .flush_tx()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// Get the state of the specified FIFO.
    ///
    /// Parameters:
    ///     about_tx: True returns data about the TX FIFO.
    ///         False returns data about the RX FIFO.
    pub fn get_fifo_state(&mut self, about_tx: i32) -> PyResult<FifoState> {
        self.inner
            .get_fifo_state(about_tx != 0)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
            .map(FifoState::from_inner)
    }

    /// Set/get the Power Amplitude (PA) level used for all transmissions (including
    /// auto ack packet).
    #[setter]
    pub fn set_pa_level(&mut self, pa_level: PaLevel) -> PyResult<()> {
        self.inner
            .set_pa_level(pa_level.into_inner())
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    #[getter]
    pub fn get_pa_level(&mut self) -> PyResult<PaLevel> {
        self.inner
            .get_pa_level()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
            .map(PaLevel::from_inner)
    }

    /// Set/get the statically sized payload length.
    ///
    /// This configuration is not used if dynamic payloads are enabled.
    /// Use [`RF24.get_dynamic_payload_length()`][rf24_py.RF24.get_dynamic_payload_length]
    /// instead if dynamically sized payloads are enabled (via
    /// [`RF24.dynamic_payloads`][rf24_py.RF24.dynamic_payloads]).
    #[setter]
    pub fn set_payload_length(&mut self, length: u8) -> PyResult<()> {
        self.inner
            .set_payload_length(length)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    #[getter]
    pub fn get_payload_length(&mut self) -> PyResult<u8> {
        self.inner
            .get_payload_length()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// Enable or disable the dynamically sized payloads feature.
    ///
    /// If set to `true`, the statically sized payload length (set via
    /// [`RF24.payload_length`][rf24_py.RF24.payload_length]) are not used.
    #[setter]
    pub fn set_dynamic_payloads(&mut self, enable: i32) -> PyResult<()> {
        self.inner
            .set_dynamic_payloads(enable != 0)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    #[getter]
    pub fn get_dynamic_payloads(&self) -> bool {
        self.inner.get_dynamic_payloads()
    }

    /// Get the length of the next available payload in the RX FIFO.
    ///
    /// If dynamically sized payloads are not enabled (via
    /// [`RF24.dynamic_payloads`][rf24_py.RF24.dynamic_payloads]),
    /// then use [`RF24.payload_length`][rf24_py.RF24.payload_length].
    pub fn get_dynamic_payload_length(&mut self) -> PyResult<u8> {
        self.inner
            .get_dynamic_payload_length()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// Open a specific pipe for receiving from the given address.
    ///
    /// It is highly recommended to avoid using pipe 0 to receive because it is also
    /// used to transmit automatic acknowledgements.
    ///
    /// Note:
    ///     Only pipes 0 and 1 actually use up to 5 bytes of the given address.
    ///     Pipes 2 - 5 only use the first byte of the given address and last 4
    ///     bytes of the address set to pipe 1.
    ///
    /// Parameters:
    ///     pipe: The pipe number to receive data. This must be in range [0, 5],
    ///         otherwise this function does nothing.
    ///     address: The address to receive data from.
    pub fn open_rx_pipe(&mut self, pipe: u8, address: &[u8]) -> PyResult<()> {
        self.inner
            .open_rx_pipe(pipe, address)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// Set the address used for transmitting on pipe 0.
    ///
    /// Only pipe 0 can be used for transmitting. It is highly recommended to
    /// avoid using pipe 0 to receive because of this.
    ///
    /// Parameters:
    ///     address: The address to receive data from.
    pub fn open_tx_pipe(&mut self, address: &[u8]) -> PyResult<()> {
        self.inner
            .open_tx_pipe(address)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// Close the specified pipe from receiving transmissions.
    ///
    /// Use [`RF24.open_rx_pipe()`][rf24_py.RF24.open_rx_pipe] to set the address for a
    /// specific pipe.
    ///
    /// Parameters:
    ///     pipe: The pipe to close. This must be in range [0, 5], otherwise this function
    ///         does nothing.
    pub fn close_rx_pipe(&mut self, pipe: u8) -> PyResult<()> {
        self.inner
            .close_rx_pipe(pipe)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// Set/get the address length (applied to all pipes).
    ///
    /// The address length is only allowed to be in range [2, 5].
    #[setter]
    pub fn set_address_length(&mut self, length: u8) -> PyResult<()> {
        self.inner
            .set_address_length(length)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    #[getter]
    pub fn get_address_length(&mut self) -> PyResult<u8> {
        self.inner
            .get_address_length()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// Power Up/Down the radio.
    ///
    /// No transmissions can be received when the radio is powered down.
    ///
    /// See also:
    ///     Setting this attribute to `True` is equivalent to calling
    ///     [`power_up()`][rf24_py.RF24.power_up] (using default delay).
    #[setter]
    pub fn set_power(&mut self, enable: i32) -> PyResult<()> {
        if enable != 0 {
            self.power_up(None)
        } else {
            self.power_down()
        }
    }

    #[getter]
    pub fn get_power(&self) -> bool {
        self.inner.is_powered()
    }

    /// Power Down the radio.
    ///
    /// No transmissions can be received when the radio is powered down.
    pub fn power_down(&mut self) -> PyResult<()> {
        self.inner
            .power_down()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// Power up the radio.
    ///
    /// Parameters:
    ///     delay: The number of nanoseconds to wait for the radio to finish
    ///         powering up. If not specified, the default wait time defaults
    ///         to 5 milliseconds.
    #[pyo3(
        text_signature = "(delay: int | None = None) -> None",
        signature = (delay = None),
    )]
    pub fn power_up(&mut self, delay: Option<u32>) -> PyResult<()> {
        self.inner
            .power_up(delay)
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// The driver will delay for this duration (32 bit unsigned int of microseconds)
    /// when [`as_tx()`][rf24_py.RF24.as_tx] is called.
    ///
    /// If the auto-ack feature is disabled, then this can be set as low as 0.
    /// If the auto-ack feature is enabled, then set to 100 microseconds minimum on
    /// generally faster devices (like RPi).
    ///
    /// This value cannot be negative.
    ///
    /// Since this value can be optimized per the radio's data rate, this value is
    /// automatically adjusted when changing
    /// [`data_rate`][rf24_py.RF24.data_rate].
    /// If setting this to a custom value be sure, to set it *after*
    /// changing the radio's data rate.
    ///
    /// Warning:
    ///     If set to 0, ensure 130 microsecond delay
    ///     after calling [`as_tx()`][rf24_py.RF24.as_tx]
    ///     and before transmitting.
    #[setter]
    pub fn set_tx_delay(&mut self, value: u32) {
        self.inner.tx_delay = value;
    }

    #[getter]
    pub fn get_tx_delay(&self) -> u32 {
        self.inner.tx_delay
    }

    /// Configure the IRQ pin to reflect the specified [`StatusFlags`][rf24_py.StatusFlags].
    ///
    /// Other Parameters:
    ///     flags: If this value is `None`, then all flags are reflected by the IRQ pin.
    ///
    #[pyo3(signature = (flags = None))]
    pub fn set_status_flags(&mut self, flags: Option<StatusFlags>) -> PyResult<()> {
        let flags = flags.map(|f| f.into_inner());
        #[allow(clippy::unwrap_or_default)]
        self.inner
            .set_status_flags(flags.unwrap_or(rf24::StatusFlags::new()))
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// Reset the specified [`StatusFlags`][rf24_py.StatusFlags].
    ///
    /// Other Parameters:
    ///     flags: If this value is `None`, then all flags are reset.
    #[pyo3(signature = (flags = None))]
    pub fn clear_status_flags(&mut self, flags: Option<StatusFlags>) -> PyResult<()> {
        let flags = flags.map(|f| f.into_inner());
        #[allow(clippy::unwrap_or_default)]
        self.inner
            .clear_status_flags(flags.unwrap_or(rf24::StatusFlags::new()))
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// Update the cached value of Status flags.
    ///
    /// Use [`RF24.get_status_flags()`][rf24_py.RF24.get_status_flags] to get the updated values.
    pub fn update(&mut self) -> PyResult<()> {
        self.inner
            .update()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }

    /// Get the current state of the [`StatusFlags`][rf24_py.StatusFlags].
    ///
    /// Note:
    ///     This function simply returns the value of the flags that was cached
    ///     from the last SPI transaction. It does not actually update the values
    ///     (from the radio) before returning them.
    ///
    ///     Use [`RF24.update()`][rf24_py.RF24.update] to update them first.
    pub fn get_status_flags(&mut self) -> StatusFlags {
        let mut flags = rf24::StatusFlags::default();
        self.inner.get_status_flags(&mut flags);
        StatusFlags::from_inner(flags)
    }

    /// Print helpful debug information to stdout.
    pub fn print_details(&mut self) -> PyResult<()> {
        self.inner
            .print_details()
            .map_err(|e| PyRuntimeError::new_err(format!("{e:?}")))
    }
}
