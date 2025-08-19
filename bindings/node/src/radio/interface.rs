use std::time::Duration;

use super::config::RadioConfig;
use super::types::{
    coerce_to_bool, AvailablePipe, CrcLength, DataRate, FifoState, HardwareConfig, PaLevel,
    StatusFlags, WriteConfig,
};

use embedded_hal::{delay::DelayNs, digital::OutputPin};
use linux_embedded_hal::{
    gpio_cdev::{chips, LineRequestFlags},
    spidev::{SpiModeFlags, SpidevOptions},
    CdevPin, SpidevDevice,
};
use nix::sys::time::TimeSpec;
use nix::time::{clock_nanosleep, ClockId, ClockNanosleepFlags};

use napi::{bindgen_prelude::Buffer, Error, JsNumber, Result, Status};

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

/// This class provides the user facing API to interact with a nRF24L01 transceiver.
#[napi(js_name = "RF24")]
pub struct RF24 {
    inner: rf24::radio::RF24<SpidevDevice, CdevPin, Delay>,
    read_buf: [u8; 32],
}

#[napi]
impl RF24 {
    /// Construct an object to control the radio.
    ///
    /// @param cePin - The GPIO pin number connected to the radio's CE pin.
    /// @param csPin - The identifying number for the SPI bus' CS pin;
    /// also labeled as "CEx" (where "x" is this parameter's value) on many
    /// Raspberry Pi pin diagrams. See {@link HardwareConfig.devSpiBus} for more detail.
    /// @param hardwareConfig - Optional parameters to fine tune hardware configuration
    /// (like SPI bus number and GPIO chip number).
    ///
    /// @group Basic
    #[napi(constructor)]
    pub fn new(ce_pin: u32, cs_pin: u8, hardware_config: Option<HardwareConfig>) -> Result<Self> {
        // convert optional arg to default values
        let hw_config = hardware_config.unwrap_or_default();
        let spi_speed = hw_config.spi_speed.unwrap_or(10_000_000);
        let dev_gpio_chip = hw_config.dev_gpio_chip.unwrap_or_default();
        let dev_spi_bus = hw_config.dev_spi_bus.unwrap_or_default();

        // get the desired "/dev/gpiochip{dev_gpio_chip}"
        let mut dev_gpio = chips()
            .map_err(|_| {
                Error::new(
                    Status::GenericFailure,
                    "Failed to get list of GPIO chips for the system",
                )
            })?
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
            .ok_or(Error::new(
                Status::InvalidArg,
                format!("Could not find specified dev/gpiochip{dev_gpio_chip} for this system."),
            ))?
            .map_err(|e| {
                Error::new(
                    Status::InvalidArg,
                    format!("Could not open GPIO chip dev/gpiochip{dev_gpio_chip}: {e:?}"),
                )
            })?;
        let ce_line = dev_gpio.get_line(ce_pin).map_err(|e| {
            Error::new(
                Status::InvalidArg,
                format!("GPIO{ce_pin} is unavailable: {e:?}"),
            )
        })?;
        let ce_line_handle = ce_line
            .request(LineRequestFlags::OUTPUT, 0, "rf24-rs")
            .map_err(|e| {
                Error::new(
                    Status::InvalidArg,
                    format!("GPIO{ce_pin} is already in use: {e:?}"),
                )
            })?;
        let ce_pin = CdevPin::new(ce_line_handle)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))?;

        let mut spi =
            SpidevDevice::open(format!("/dev/spidev{dev_spi_bus}.{cs_pin}")).map_err(|_| {
                Error::new(Status::InvalidArg, format!(
                    "SPI bus {dev_spi_bus} with CS pin option {cs_pin} is not available in this system"
                )
            )
            })?;
        let config = SpidevOptions::new()
            .max_speed_hz(spi_speed)
            .mode(SpiModeFlags::SPI_MODE_0)
            .bits_per_word(8)
            .build();
        spi.configure(&config)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))?;

        Ok(Self {
            inner: rf24::radio::RF24::new(ce_pin, spi, Delay),
            read_buf: [0u8; 32],
        })
    }

    /// Initialize the radio on the configured hardware (as specified to {@link RF24} constructor).
    ///
    /// @throws A Generic Error if a hardware failure caused problems
    /// (includes a message to describe what problem was detected).
    ///
    /// This is the same as {@link RF24.withConfig},
    /// but this function also
    ///
    /// - detects if the radio is a plus variant ({@link RF24.isPlusVariant})
    /// - checks for data corruption across the SPI lines (MOSI, MISO, SCLK)
    ///
    /// @group Basic
    #[napi]
    pub fn begin(&mut self) -> Result<()> {
        self.inner
            .init()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Reconfigure the radio with the specified `config`.
    ///
    /// > [!WARNING]
    /// > It is strongly encouraged to call {@link RF24.begin}
    /// > after constructing the RF24 object.
    /// >
    /// > Only use this function subsequently to quickly switch between different
    /// > network settings.
    ///
    /// @group Configuration
    #[napi]
    pub fn with_config(&mut self, config: &RadioConfig) -> Result<()> {
        self.inner
            .with_config(config.get_inner())
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Set the radio's CE pin HIGH (`true`) or LOW (`false`).
    ///
    /// This is only exposed for advanced use of TX FIFO during
    /// asynchronous TX operations. It is highly encouraged to use
    /// {@link RF24.asRx} or {@link RF24.asTx}
    /// to ensure proper radio behavior when entering RX or TX mode.
    ///
    /// @group Advanced
    #[napi]
    pub fn ce_pin(
        &mut self,
        #[napi(ts_arg_type = "boolean | number")] value: JsNumber,
    ) -> Result<()> {
        if coerce_to_bool(Some(value), false)? {
            self.inner.ce_pin.set_high()
        } else {
            self.inner.ce_pin.set_low()
        }
        .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Is the radio in active RX mode?
    ///
    /// @group Basic
    #[napi(getter)]
    pub fn is_rx(&self) -> bool {
        self.inner.is_rx()
    }

    /// Put the radio into active RX mode.
    ///
    /// Conventionally, this should be called after setting the RX addresses via
    /// {@link RF24.openRxPipe}.
    ///
    /// This function will restore the cached RX address set to pipe 0.
    /// This is done because the {@link RF24.asTx} will appropriate the
    /// RX address on pipe 0 for auto-ack purposes.
    ///
    /// @group Basic
    #[napi]
    pub fn as_rx(&mut self) -> Result<()> {
        self.inner
            .as_rx()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Puts the radio into an inactive TX mode.
    ///
    /// This must be called at least once before calling {@link RF24.send} or
    /// {@link RF24.write}.
    ///
    /// For auto-ack purposes, this function will also restore
    /// the cached `txAddress` to the RX pipe 0.
    ///
    /// The datasheet recommends idling the radio in an inactive TX mode.
    ///
    /// > [!NOTE]
    /// > This function will also flush the TX FIFO when ACK payloads are enabled
    /// > (via {@link RF24.ackPayloads}).
    ///
    /// @param txAddress - If specified, then this buffer will be
    /// cached and set as the new TX address.
    ///
    /// @group Basic
    #[napi]
    pub fn as_tx(&mut self, tx_address: Option<Buffer>) -> Result<()> {
        self.inner
            .as_tx(tx_address.as_deref())
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Blocking function that loads a given `buf` into the TX FIFO, waits for a response
    /// (if auto-ack is enabled), then returns a Boolean describing success.
    ///
    /// @param buf - The buffer of bytes to transmit.
    /// @param askNoAck - A flag to disable the auto-ack feature for the given payload in `buf`.
    /// This has no effect if auto-ack is disabled or
    /// {@link RF24.allowAskNoAck} is not enabled.
    ///
    /// @returns A boolean that describes if transmission is successful or not.
    /// This will always return true if auto-ack is disabled.
    ///
    /// @group Basic
    #[napi]
    pub fn send(
        &mut self,
        buf: Buffer,
        #[napi(ts_arg_type = "boolean | number")] ask_no_ack: Option<JsNumber>,
    ) -> Result<bool> {
        let buf = buf.to_vec();
        let ask_no_ack = coerce_to_bool(ask_no_ack, false)?;
        self.inner
            .send(&buf, ask_no_ack)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// A non-blocking function that uploads a given `buf` to the radio's TX FIFO.
    ///
    /// This is a helper function to {@link RF24.send}.
    /// Use this in combination with {@link RF24.update} and
    /// {@link RF24.getStatusFlags}
    /// to determine if transmission was successful.
    ///
    /// @param buf - The buffer of bytes to load into the TX FIFO.
    ///
    /// @returns A Boolean that describes if the given `buf` was successfully loaded
    /// into the TX FIFO. Remember, the TX FIFO has only 3 levels ("slots").
    ///
    /// @group Advanced
    #[napi]
    pub fn write(&mut self, buf: Buffer, write_config: Option<WriteConfig>) -> Result<bool> {
        let buf = buf.to_vec();
        let options = write_config.unwrap_or_default();
        self.inner
            .write(
                &buf,
                options.ask_no_ack.unwrap_or_default(),
                options.start_tx.unwrap_or(true),
            )
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Read data from the radio's RX FIFO.
    ///
    /// Use {@link RF24.available} to determine if there is data ready to read from the RX FIFO.
    ///
    /// @param len - An optional number of bytes to read from the FIFO. This is capped at `32`.
    /// If not specified, then the length of the next available payload is used (which automatically
    /// respects if dynamic payloads are enabled).
    ///
    /// Use {@link RF24.dynamicPayloads} for dynamically sized
    /// payload or {@link RF24.payloadLength} for statically sized
    /// payloads.
    ///
    /// @group Basic
    #[napi]
    pub fn read(&mut self, len: Option<u8>) -> Result<Buffer> {
        let len = self
            .inner
            .read(&mut self.read_buf, len)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))?;
        Ok(Buffer::from(&self.read_buf[0..len as usize]))
    }

    /// A blocking function to resend a failed payload in the TX FIFO.
    ///
    /// This is similar to {@link RF24.send} but specifically for
    /// failed transmissions.
    ///
    /// @group Basic
    #[napi]
    pub fn resend(&mut self) -> Result<bool> {
        self.inner
            .resend()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// A non-blocking function to restart a failed transmission.
    ///
    /// This is a helper function to {@link RF24.resend}.
    /// Use {@link RF24.update} and
    /// {@link RF24.getStatusFlags} to determine if
    /// retransmission was successful.
    ///
    /// @group Advanced
    #[napi]
    pub fn rewrite(&mut self) -> Result<()> {
        self.inner
            .rewrite()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Get the Automatic Retry Count (ARC) of attempts made during the last transmission.
    ///
    /// This resets with every new transmission. The returned value is meaningless if the
    /// auto-ack feature is disabled.
    ///
    /// Use {@link RF24.setAutoRetries} to configure the
    /// automatic retries feature.
    ///
    /// @group Advanced
    #[napi]
    pub fn get_last_arc(&mut self) -> Result<u8> {
        self.inner
            .get_last_arc()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Is this radio a nRF24L01+ variant?
    ///
    /// The bool that this attribute returns is only valid _after_ calling
    /// {@link RF24.begin}.
    ///
    /// @group Configuration
    #[napi(getter)]
    pub fn is_plus_variant(&self) -> bool {
        self.inner.is_plus_variant()
    }

    /// Was the Received Power Detection (RPD) trigger?
    ///
    /// This flag is asserted during an RX session (after a mandatory 130 microseconds
    /// duration) if a signal stronger than -64 dBm was detected.
    ///
    /// Note that if a payload was placed in RX mode, then that means
    /// the signal used to transmit that payload was stronger than either
    ///
    /// * -82 dBm in 2 Mbps {@link DataRate}
    /// * -85 dBm in 1 Mbps {@link DataRate}
    /// * -94 dBm in 250 Kbps {@link DataRate}
    ///
    /// Sensitivity may vary based of the radio's model and manufacturer.
    /// The information above is stated in the nRF24L01+ datasheet.
    ///
    /// @group Advanced
    #[napi(getter)]
    pub fn rpd(&mut self) -> Result<bool> {
        self.inner
            .rpd()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Start a constant carrier wave
    ///
    /// This functionality is meant for hardware tests (in conjunction with {@link RF24.rpd}).
    /// Typically, this behavior is required by government agencies to enforce regional restrictions.
    ///
    /// @param level - The Power Amplitude level to use when transmitting.
    /// @param channel - The channel (radio's frequency) used to transmit.
    /// The channel should not be changed while transmitting because it can cause
    /// undefined behavior.
    ///
    /// @group Advanced
    #[napi]
    pub fn start_carrier_wave(&mut self, level: PaLevel, channel: u8) -> Result<()> {
        self.inner
            .start_carrier_wave(level.into_inner(), channel)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Stop the constant carrier wave started via {@link RF24.startCarrierWave}.
    ///
    /// This function leaves the radio in a configuration that may be undesired or
    /// unexpected because of the setup involved in {@link RF24.startCarrierWave}.
    /// The {@link PaLevel} and `channel` passed to {@link RF24.startCarrierWave} are
    /// still set.
    /// If {@link RF24.isPlusVariant} returns `true`, the following features are all disabled:
    ///
    /// - auto-ack
    /// - CRC
    /// - auto-retry
    ///
    /// @group Advanced
    #[napi]
    pub fn stop_carrier_wave(&mut self) -> Result<()> {
        self.inner
            .stop_carrier_wave()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Enable or disable the LNA feature.
    ///
    /// This is enabled by default (regardless of chip variant).
    /// See {@link PaLevel} for effective behavior.
    ///
    /// On nRF24L01+ modules with a builtin antenna, this feature is always enabled.
    /// For clone's and module's with a separate PA/LNA circuit (external antenna),
    /// this function may not behave exactly as expected. Consult the radio module's
    /// manufacturer.
    ///
    /// @group Configuration
    #[napi]
    pub fn set_lna(
        &mut self,
        #[napi(ts_arg_type = "boolean | number")] enable: JsNumber,
    ) -> Result<()> {
        self.inner
            .set_lna(coerce_to_bool(Some(enable), true)?)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Enable or disable the custom ACK payloads attached to auto-ack packets.
    ///
    /// > [!IMPORTANT]
    /// > This feature requires dynamically sized payloads.
    /// > This attribute will enable {@link RF24.dynamicPayloads}
    /// > automatically when needed. This attribute will not disable
    /// > {@link RF24.dynamicPayloads}.
    ///
    /// @group Configuration
    #[napi(setter, js_name = "ackPayloads")]
    pub fn set_ack_payloads(
        &mut self,
        #[napi(ts_arg_type = "boolean | number")] enable: JsNumber,
    ) -> Result<()> {
        self.inner
            .set_ack_payloads(coerce_to_bool(Some(enable), false)?)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// @group Configuration
    #[napi(getter, js_name = "ackPayloads")]
    pub fn get_ack_payloads(&self) -> bool {
        self.inner.get_ack_payloads()
    }

    /// Enable or disable the auto-ack feature for all pipes.
    ///
    /// > [!NOTE]
    /// > This feature requires CRC to be enabled.
    /// > See {@link RF24.crcLength} for more detail.
    ///
    /// @group Configuration
    #[napi]
    pub fn set_auto_ack(
        &mut self,
        #[napi(ts_arg_type = "boolean | number")] enable: JsNumber,
    ) -> Result<()> {
        self.inner
            .set_auto_ack(coerce_to_bool(Some(enable), false)?)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Enable or disable the auto-ack feature for a specified `pipe`.
    ///
    /// @group Configuration
    #[napi]
    pub fn set_auto_ack_pipe(&mut self, enable: JsNumber, pipe: u8) -> Result<()> {
        self.inner
            .set_auto_ack_pipe(coerce_to_bool(Some(enable), false)?, pipe)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Allow disabling the auto-ack feature for individual payloads.
    ///
    /// @param enable - Setting this to `true` will allow the `askNoAck` parameter to
    /// take effect. See {@link RF24.send} and
    /// {@link WriteConfig.askNoAck} for more detail.
    ///
    /// @group Configuration
    #[napi]
    pub fn allow_ask_no_ack(
        &mut self,
        #[napi(ts_arg_type = "boolean | number")] enable: JsNumber,
    ) -> Result<()> {
        self.inner
            .allow_ask_no_ack(coerce_to_bool(Some(enable), false)?)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Upload a given ACK packet's payload (`buf`) into the radio's TX FIFO.
    ///
    /// This feature requires {@link RF24.ackPayloads}
    /// to be enabled.
    ///
    /// @param pipe - The pipe number that (when data is received) will be responded
    /// with the given payload (`buf`).
    /// @param buf - The payload to attach to the auto-ack packet when responding to
    /// data received on specified `pipe`.
    ///
    /// @returns A boolean value that describes if the payload was successfully uploaded
    /// to the TX FIFO. Remember, the TX FIFO only has 3 levels ("slots").
    ///
    /// @group Advanced
    #[napi]
    pub fn write_ack_payload(&mut self, pipe: u8, buf: Buffer) -> Result<bool> {
        let buf = buf.to_vec();
        self.inner
            .write_ack_payload(pipe, &buf)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Configure the automatic retry feature.
    ///
    /// This feature is part of the auto-ack feature, thus the auto-ack feature is
    /// required for this function to have any effect.
    ///
    /// @param delay - This value is clamped to the range [0, 15]. This value is
    /// translated to microseconds with the formula `250 + (delay * 250) = microseconds`.
    /// Meaning, the effective range of `delay` is [250, 4000].
    /// @param count - The number of attempt to retransmit when no ACK packet was
    /// received (after transmitting). This value is clamped to the range [0, 15].
    ///
    /// @group Configuration
    #[napi]
    pub fn set_auto_retries(&mut self, delay: u8, count: u8) -> Result<()> {
        self.inner
            .set_auto_retries(delay, count)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Set the channel (frequency) that the radio uses to transmit and receive.
    ///
    /// @param channel - This value is clamped to the range [0, 125].
    ///
    /// This value can be roughly translated into a frequency with the formula:
    /// ```text
    /// frequency (in Hz) = channel + 2400
    /// ```
    ///
    /// @group Basic
    #[napi(setter, js_name = "channel")]
    pub fn set_channel(&mut self, channel: u8) -> Result<()> {
        self.inner
            .set_channel(channel)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// @group Basic
    #[napi(getter, js_name = "channel")]
    pub fn get_channel(&mut self) -> Result<u8> {
        self.inner
            .get_channel()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// @group Configuration
    #[napi(getter, js_name = "crcLength")]
    pub fn get_crc_length(&mut self) -> Result<CrcLength> {
        self.inner
            .get_crc_length()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
            .map(CrcLength::from_inner)
    }

    /// Get/set the {@link CrcLength} used for all outgoing and incoming
    /// transmissions.
    ///
    /// > [!NOTE]
    /// > If disabled ({@link CrcLength.Disabled})
    /// > while auto-ack feature is enabled, then this function's returned value does not reflect
    /// > the fact that CRC is forcefully enabled by the radio's firmware (needed by the
    /// > auto-ack feature).
    ///
    /// @group Configuration
    #[napi(setter, js_name = "crcLength")]
    pub fn set_crc_length(&mut self, crc_length: CrcLength) -> Result<()> {
        self.inner
            .set_crc_length(crc_length.into_inner())
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// @group Configuration
    #[napi(getter, js_name = "dataRate")]
    pub fn get_data_rate(&mut self) -> Result<DataRate> {
        self.inner
            .get_data_rate()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
            .map(DataRate::from_inner)
    }

    /// Get/set the {@link DataRate} used for all incoming and outgoing transmissions.
    ///
    /// @group Configuration
    #[napi(setter, js_name = "dataRate")]
    pub fn set_data_rate(&mut self, data_rate: DataRate) -> Result<()> {
        self.inner
            .set_data_rate(data_rate.into_inner())
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Is there a payload available in the RX FIFO?
    ///
    /// Use {@link RF24.read} to get the payload data.
    ///
    /// @group Basic
    #[napi]
    pub fn available(&mut self) -> Result<bool> {
        self.inner
            .available()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Similar to {@link RF24.available} but also returns the
    /// pipe that received the next available payload.
    ///
    /// @group Basic
    #[napi]
    pub fn available_pipe(&mut self) -> Result<AvailablePipe> {
        let mut pipe = 15;
        let result = self
            .inner
            .available_pipe(&mut pipe)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))?;
        Ok(AvailablePipe {
            available: result,
            pipe,
        })
    }

    /// Discard all 3 levels of the radio's RX FIFO.
    ///
    /// @group Advanced
    #[napi]
    pub fn flush_rx(&mut self) -> Result<()> {
        self.inner
            .flush_rx()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Discard all 3 levels of the radio's TX FIFO.
    ///
    /// This is automatically called by {@link RF24.asTx}
    /// when ACK payloads are enabled (via {@link RF24.ackPayloads}).
    ///
    /// @group Advanced
    #[napi]
    pub fn flush_tx(&mut self) -> Result<()> {
        self.inner
            .flush_tx()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Get the state of the specified FIFO.
    ///
    /// @param aboutTx - True returns data about the TX FIFO.
    /// False returns data about the RX FIFO.
    ///
    /// @group Advanced
    #[napi]
    pub fn get_fifo_state(
        &mut self,
        #[napi(ts_arg_type = "boolean | number")] about_tx: JsNumber,
    ) -> Result<FifoState> {
        self.inner
            .get_fifo_state(coerce_to_bool(Some(about_tx), false)?)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
            .map(FifoState::from_inner)
    }

    /// @group Configuration
    #[napi(getter, js_name = "paLevel")]
    pub fn get_pa_level(&mut self) -> Result<PaLevel> {
        self.inner
            .get_pa_level()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
            .map(PaLevel::from_inner)
    }

    /// Get/set the Power Amplitude (PA) level used for all transmissions (including
    /// auto ack packet).
    ///
    /// @param paLevel - The {@link PaLevel} to use.
    ///
    /// @group Configuration
    #[napi(setter, js_name = "paLevel")]
    pub fn set_pa_level(&mut self, pa_level: PaLevel) -> Result<()> {
        self.inner
            .set_pa_level(pa_level.into_inner())
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Get/set the statically sized payload length.
    ///
    /// This configuration is not used if dynamic payloads are enabled.
    /// Use {@link RF24.getDynamicPayloadLength}
    /// instead if dynamically sized payloads are enabled (via
    /// {@link RF24.dynamicPayloads}).
    ///
    /// @group Configuration
    #[napi(setter, js_name = "payloadLength")]
    pub fn set_payload_length(&mut self, length: u8) -> Result<()> {
        self.inner
            .set_payload_length(length)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// @group Configuration
    #[napi(getter, js_name = "payloadLength")]
    pub fn get_payload_length(&mut self) -> Result<u8> {
        self.inner
            .get_payload_length()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Enable or disable the dynamically sized payloads feature.
    ///
    /// @param enable - If set to `true`, the statically sized payload length (set via
    /// {@link RF24.payloadLength}) are not used.
    ///
    /// @group Configuration
    #[napi(setter, js_name = "dynamicPayloads")]
    pub fn set_dynamic_payloads(
        &mut self,
        #[napi(ts_arg_type = "boolean | number")] enable: JsNumber,
    ) -> Result<()> {
        self.inner
            .set_dynamic_payloads(coerce_to_bool(Some(enable), false)?)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// @group Configuration
    #[napi(getter, js_name = "dynamicPayloads")]
    pub fn get_dynamic_payloads(&self) -> bool {
        self.inner.get_dynamic_payloads()
    }

    /// Get the length of the next available payload in the RX FIFO.
    ///
    /// If dynamically sized payloads are not enabled (via
    /// {@link RF24.dynamicPayloads}), then use {@link RF24.payloadLength}.
    ///
    /// @group Advanced
    #[napi]
    pub fn get_dynamic_payload_length(&mut self) -> Result<u8> {
        self.inner
            .get_dynamic_payload_length()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Open a specific pipe for receiving from the given address.
    ///
    /// It is highly recommended to avoid using pipe 0 to receive because it is also
    /// used to transmit automatic acknowledgements.
    ///
    /// > [!NOTE]
    /// > Only pipes 0 and 1 actually use up to 5 bytes of the given address.
    /// > Pipes 2 - 5 only use the first byte of the given address and last 4
    /// > bytes of the address set to pipe 1.
    ///
    /// @param pipe - The pipe number to receive data. This must be in range [0, 5],
    /// otherwise this function does nothing.
    /// @param address - The address to receive data from.
    ///
    /// @group Basic
    #[napi]
    pub fn open_rx_pipe(&mut self, pipe: u8, address: Buffer) -> Result<()> {
        let address = address.to_vec();
        self.inner
            .open_rx_pipe(pipe, &address)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Close the specified pipe from receiving transmissions.
    ///
    /// Use {@link RF24.openRxPipe} to set the address for a
    /// specific pipe.
    ///
    /// @param pipe - The pipe to close. This must be in range [0, 5], otherwise this function
    /// does nothing.
    ///
    /// @group Basic
    #[napi]
    pub fn close_rx_pipe(&mut self, pipe: u8) -> Result<()> {
        self.inner
            .close_rx_pipe(pipe)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Set the address length (applied to all pipes).
    ///
    /// @param length - The address length is clamped to the range [2, 5].
    ///
    /// @group Configuration
    #[napi(setter, js_name = "addressLength")]
    pub fn set_address_length(&mut self, length: u8) -> Result<()> {
        self.inner
            .set_address_length(length)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// @group Configuration
    #[napi(getter, js_name = "addressLength")]
    pub fn get_address_length(&mut self) -> Result<u8> {
        self.inner
            .get_address_length()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// @group Configuration
    #[napi(getter, js_name = "power")]
    pub fn is_powered(&self) -> bool {
        self.inner.is_powered()
    }

    /// Control the radio's powered level.
    ///
    /// This is just a convenience attribute that calls {@link RF24.powerUp}
    /// or {@link RF24.powerDown}.
    ///
    /// Use {@link RF24.isRx} to determine if the radio is in RX or TX mode.
    ///
    /// @group Configuration
    #[napi(setter, js_name = "power")]
    pub fn set_power(&mut self, enable: JsNumber) -> Result<()> {
        if coerce_to_bool(Some(enable), true)? {
            self.power_up(None)
        } else {
            self.power_down()
        }
    }

    /// Power Down the radio.
    ///
    /// No transmissions can be received when the radio is powered down.
    ///
    /// @group Configuration
    #[napi]
    pub fn power_down(&mut self) -> Result<()> {
        self.inner
            .power_down()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Power up the radio.
    ///
    /// @param delay - The number of nanoseconds to wait for the radio to finish
    /// powering up. If not specified, the default wait time defaults to 5 milliseconds.
    ///
    /// @group Configuration
    #[napi]
    pub fn power_up(&mut self, delay: Option<u32>) -> Result<()> {
        self.inner
            .power_up(delay)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// @group Configuration
    #[napi(getter, js_name = "txDelay")]
    pub fn get_tx_delay(&self) -> u32 {
        self.inner.tx_delay
    }

    /// The driver will delay for this duration (32 bit unsigned int of microseconds)
    /// when {@link RF24.asTx} is called.
    ///
    /// If the auto-ack feature is disabled, then this can be set as low as 0.
    /// If the auto-ack feature is enabled, then set to 100 microseconds minimum on
    /// generally faster devices (like RPi).
    ///
    /// This value cannot be negative.
    ///
    /// Since this value can be optimized per the radio's data rate, this value is
    /// automatically adjusted when changing {@link RF24.dataRate}.
    /// If setting this to a custom value be sure, to set it *after*
    /// changing the radio's data rate.
    ///
    /// > [!WARNING]
    /// > If set to 0, then the concurrent outgoing ACK packet (when auto-ack is enabled)
    /// > may fail to transmit when exiting RX mode with {@link RF24.asTx}.
    ///
    /// @group Configuration
    #[napi(setter, js_name = "txDelay")]
    pub fn set_tx_delay(&mut self, value: u32) {
        self.inner.tx_delay = value;
    }

    /// Configure the IRQ pin to reflect the specified {@link StatusFlags}.
    ///
    /// @param flags - If no value is given, then all flags are reflected by the IRQ pin.
    ///
    /// @group Configuration
    #[napi]
    pub fn set_status_flags(&mut self, flags: Option<StatusFlags>) -> Result<()> {
        let flags = flags.unwrap_or(StatusFlags {
            rx_dr: Some(true),
            tx_ds: Some(true),
            tx_df: Some(true),
        });
        self.inner
            .set_status_flags(flags.into_inner())
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Reset the specified {@link StatusFlags}.
    ///
    /// @param flags - If no value is given, then all flags are reset.
    ///
    /// @group Advanced
    #[napi]
    pub fn clear_status_flags(&mut self, flags: Option<StatusFlags>) -> Result<()> {
        let flags = flags.unwrap_or(StatusFlags {
            rx_dr: Some(true),
            tx_ds: Some(true),
            tx_df: Some(true),
        });
        self.inner
            .clear_status_flags(flags.into_inner())
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Update the cached value of Status flags.
    ///
    /// Use {@link RF24.getStatusFlags} to get the updated values.
    ///
    /// @group Advanced
    #[napi]
    pub fn update(&mut self) -> Result<()> {
        self.inner
            .update()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Get the current state of the {@link StatusFlags}.
    ///
    /// > [!NOTE]
    /// > This function simply returns the value of the flags that was cached
    /// > from the last SPI transaction. It does not actually update the values
    /// > (from the radio) before returning them.
    /// >
    /// > Use {@link RF24.update} to update them first.
    ///
    /// @group Advanced
    #[napi]
    pub fn get_status_flags(&mut self) -> StatusFlags {
        let mut flags = rf24::StatusFlags::default();
        self.inner.get_status_flags(&mut flags);
        StatusFlags::from_inner(flags)
    }

    /// Print helpful debug information to stdout.
    ///
    /// @group Configuration
    #[napi]
    pub fn print_details(&mut self) -> Result<()> {
        self.inner
            .print_details()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }
}
