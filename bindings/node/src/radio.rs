#![cfg(target_os = "linux")]

use crate::types::{
    AvailablePipe, CrcLength, DataRate, FifoState, HardwareConfig, PaLevel, StatusFlags,
    WriteConfig,
};
use linux_embedded_hal::{
    gpio_cdev::{chips, LineRequestFlags},
    spidev::{SpiModeFlags, SpidevOptions},
    CdevPin, Delay, SpidevDevice,
};
use napi::{bindgen_prelude::Buffer, Error, Result, Status};

use rf24::radio::prelude::*;

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
    /// @group Basic
    #[napi]
    pub fn begin(&mut self) -> Result<()> {
        self.inner
            .init()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Is the radio in active RX mode?
    ///
    /// @group Basic
    #[napi(getter)]
    pub fn is_listening(&self) -> bool {
        self.inner.is_listening()
    }

    /// Put the radio into active RX mode.
    ///
    /// > [!WARNING]
    /// > Do not call {@link RF24.send} while in active RX mode because (internally in rust)
    /// > that _will_ cause an infinite loop.
    ///
    /// @group Basic
    #[napi]
    pub fn start_listening(&mut self) -> Result<()> {
        self.inner
            .start_listening()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Deactivates active RX mode and puts the radio into an inactive TX mode.
    ///
    /// The datasheet recommends idling the radio in an inactive TX mode.
    ///
    /// @group Basic
    #[napi]
    pub fn stop_listening(&mut self) -> Result<()> {
        self.inner
            .stop_listening()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Blocking function that loads a given `buf` into the TX FIFO, waits for a response
    /// (if auto-ack is enabled), then returns a Boolean describing success.
    ///
    /// @param buf - The buffer of bytes to transmit.
    /// @param askNoAck - A flag to disable the auto-ack feature for the given payload in `buf`.
    /// This has no effect if auto-ack is disabled or
    /// {@link RF24.allowAskNoAck | `RF24.allowAskNoAck()`} is not enabled.
    ///
    /// @returns A boolean that describes if transmission is successful or not.
    /// This will always return true if auto-ack is disabled.
    ///
    /// @group Basic
    #[napi]
    pub fn send(&mut self, buf: Buffer, ask_no_ack: Option<bool>) -> Result<bool> {
        let buf = buf.to_vec();
        self.inner
            .send(&buf, ask_no_ack.unwrap_or_default())
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// A non-blocking function that uploads a given `buf` to the radio's TX FIFO.
    ///
    /// This is a helper function to {@link RF24.send | `RF24.send()`}.
    /// Use this in combination with {@link RF24.update | `RF24.update()`} and
    /// {@link RF24.getStatusFlags | `RF24.getStatusFlags()`}
    /// to determine if transmission was successful.
    ///
    /// @param buf - The buffer of bytes to load into the TX FIFO.
    ///
    /// @returns A Boolean that describes if the given `buf` was successfully loaded
    /// into the TX FIFO.
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
    /// Use {@link RF24.available | `RF24.available()`} to determine if there is data ready to read from the RX FIFO.
    ///
    /// @param len - An optional number of bytes to read from the FIFO. This is capped at `32`.
    /// If not specified, then the length of the next available payload is used (which automatically
    /// respects if dynamic payloads are enabled).
    ///
    /// Use {@link RF24.setDynamicPayloads | `RF24.setDynamicPayloads()`} for dynamically sized
    /// payload or {@link RF24.setPayloadLength | `RF24.setPayloadLength()`} for statically sized
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
    /// This is similar to {@link RF24.send | `RF24.send`} but specifically for
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
    /// This is a helper function to {@link RF24.resend | `RF24.resend()`}.
    /// Use {@link RF24.update | `RF24.update()`} and
    /// {@link RF24.getStatusFlags | `RF24.getStatusFlags()`} to determine if
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
    /// Use {@link RF24.setAutoRetries | `RF24.setAutoRetries()`} to configure the
    /// automatic retries feature.
    ///
    /// @group Advanced
    #[napi]
    pub fn get_last_arc(&mut self) -> Result<u8> {
        self.inner
            .get_last_arc()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// A property that describes if the radio is a nRF24L01+ or not.
    ///
    /// @group Configuration
    #[napi(getter)]
    pub fn is_plus_variant(&self) -> bool {
        self.inner.is_plus_variant()
    }

    /// A property that describes the radio's Received Power Detection (RPD).
    ///
    /// This is reset upon entering RX mode and is only set if the radio detects a
    /// signal if strength -64 dBm or greater (actual threshold may vary depending
    /// on radio model).
    ///
    /// @group Advanced
    #[napi(getter)]
    pub fn rpd(&mut self) -> Result<bool> {
        self.inner
            .test_rpd()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Start a constant carrier wave on the given `channel` using the specified
    /// power amplitude `level`.
    ///
    /// This functionality is only useful for testing the radio hardware works as a
    /// transmitter.
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

    /// Stop transmitting the constant carrier wave.
    ///
    /// {@link RF24.startCarrierWave | `RF24.startCarrierWave()`} should be called before
    /// this function.
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
    /// On nRF24L01+ modules with a builtin antenna, this feature is always enabled.
    /// For clone's and module's with a separate PA/LNA circuit (external antenna),
    /// this function may not behave exactly as expected. Consult the radio module's
    /// manufacturer.
    ///
    /// @group Configuration
    #[napi]
    pub fn set_lna(&mut self, enable: bool) -> Result<()> {
        self.inner
            .set_lna(enable)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Enable or disable the custom ACK payloads attached to auto-ack packets.
    ///
    /// > [!IMPORTANT]
    /// > This feature requires dynamically sized payloads.
    /// > Use {@link RF24.setDynamicPayloads | `RF24.setDynamicPayloads(true)`}
    /// > to enable dynamically sized payloads.
    ///
    /// @group Configuration
    #[napi]
    pub fn allow_ack_payloads(&mut self, enable: bool) -> Result<()> {
        self.inner
            .allow_ack_payloads(enable)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Enable or disable the auto-ack feature for all pipes.
    ///
    /// > [!NOTE]
    /// > This feature requires CRC to be enabled.
    /// > See {@link RF24.setCrcLength | `RF24.setCrcLength()`} for more detail.
    ///
    /// @group Configuration
    #[napi]
    pub fn set_auto_ack(&mut self, enable: bool) -> Result<()> {
        self.inner
            .set_auto_ack(enable)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Enable or disable the auto-ack feature for a specified `pipe`.
    ///
    /// @group Configuration
    #[napi]
    pub fn set_auto_ack_pipe(&mut self, enable: bool, pipe: u8) -> Result<()> {
        self.inner
            .set_auto_ack_pipe(enable, pipe)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Allow disabling the auto-ack feature for individual payloads.
    ///
    /// @param enable - Setting this to `true` will allow the `askNoAck` parameter to
    /// take effect. See {@link RF24.send | `RF24.send()`} and
    /// {@link WriteConfig.askNoAck | `WriteConfig.askNoAck`} for more detail.
    ///
    /// @group Configuration
    #[napi]
    pub fn allow_ask_no_ack(&mut self, enable: bool) -> Result<()> {
        self.inner
            .allow_ask_no_ack(enable)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Upload a given ACK packet's payload (`buf`) into the radio's TX FIFO.
    ///
    /// This feature requires {@link RF24.allowAckPayloads | `RF24.allowAckPayloads()`}
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
    /// @param channel - The channel must be in range [0, 125], otherwise this
    /// function does nothing. This value can be roughly translated into frequency
    /// by adding its value to 2400 (`channel + 2400 = frequency in Hz`).
    ///
    /// @group Basic
    #[napi]
    pub fn set_channel(&mut self, channel: u8) -> Result<()> {
        self.inner
            .set_channel(channel)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Get the currently configured channel.
    ///
    /// @group Basic
    #[napi]
    pub fn get_channel(&mut self) -> Result<u8> {
        self.inner
            .get_channel()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Get the {@link CrcLength | `CrcLength`} used for all outgoing and incoming
    /// transmissions.
    ///
    /// > [!NOTE]
    /// > If disabled (with {@link RF24.setCrcLength | `RF24.setCrcLength(CrcLength.Disabled)`})
    /// > while auto-ack feature is disabled, then this function's returned value does not reflect
    /// > the fact that CRC is forcefully enabled by the radio's firmware (needed by the
    /// > auto-ack feature).
    ///
    /// @group Configuration
    #[napi]
    pub fn get_crc_length(&mut self) -> Result<CrcLength> {
        self.inner
            .get_crc_length()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
            .map(|e| CrcLength::from_inner(e))
    }

    /// Set the {@link CrcLength | `CrcLength`} used for all outgoing and incoming transmissions.
    ///
    /// > [!IMPORTANT]
    /// > Because CRC is required for the auto-ack feature, the radio's firmware will forcefully
    /// > enable CRC even if the user explicitly disables it (using this function).
    ///
    /// @group Configuration
    #[napi]
    pub fn set_crc_length(&mut self, crc_length: CrcLength) -> Result<()> {
        self.inner
            .set_crc_length(crc_length.into_inner())
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Get the {@link DataRate | `DataRate`} used for all incoming and outgoing transmissions.
    ///
    /// @group Configuration
    #[napi]
    pub fn get_data_rate(&mut self) -> Result<DataRate> {
        self.inner
            .get_data_rate()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
            .map(|e| DataRate::from_inner(e))
    }

    /// Set the {@link DataRate | `DataRate`} used for all incoming and outgoing transmissions.
    ///
    /// @group Configuration
    #[napi]
    pub fn set_data_rate(&mut self, data_rate: DataRate) -> Result<()> {
        self.inner
            .set_data_rate(data_rate.into_inner())
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Is there a payload available in the RX FIFO?
    ///
    /// Use {@link RF24.read | `RF24.read()`} to get the payload data.
    ///
    /// @group Basic
    #[napi]
    pub fn available(&mut self) -> Result<bool> {
        self.inner
            .available()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Similar to {@link RF24.available | `RF24.available()`} but also returns the
    /// pipe that received the next available payload.
    ///
    /// @group Basic
    #[napi]
    pub fn available_pipe(&mut self) -> Result<AvailablePipe> {
        let mut pipe = Some(0u8);
        let result = self
            .inner
            .available_pipe(&mut pipe)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))?;
        Ok(AvailablePipe {
            available: result,
            pipe: pipe.expect("`pipe` should be a number"),
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
    pub fn get_fifo_state(&mut self, about_tx: bool) -> Result<FifoState> {
        self.inner
            .get_fifo_state(about_tx)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
            .map(|e| FifoState::from_inner(e))
    }

    /// Get the currently configured Power Amplitude (PA) level.
    ///
    /// @group Configuration
    #[napi]
    pub fn get_pa_level(&mut self) -> Result<PaLevel> {
        self.inner
            .get_pa_level()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
            .map(|e| PaLevel::from_inner(e))
    }

    /// Set the Power Amplitude (PA) level used for all transmissions (including
    /// auto ack packet).
    ///
    /// @param paLevel - The {@link PaLevel | `PaLevel`} to use.
    ///
    /// @group Configuration
    #[napi]
    pub fn set_pa_level(&mut self, pa_level: PaLevel) -> Result<()> {
        self.inner
            .set_pa_level(pa_level.into_inner())
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Set the statically sized payload length.
    ///
    /// This configuration is not used if dynamic payloads are enabled.
    ///
    /// @group Configuration
    #[napi]
    pub fn set_payload_length(&mut self, length: u8) -> Result<()> {
        self.inner
            .set_payload_length(length)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Get the currently configured length of statically sized payloads.
    ///
    /// Use {@link RF24.getDynamicPayloadLength | `RF24.getDynamicPayloadLength()`}
    /// instead if dynamically sized payloads are enabled (via
    /// {@link RF24.setDynamicPayloads | `RF24.setDynamicPayloads()`}).
    ///
    /// @group Configuration
    #[napi]
    pub fn get_payload_length(&mut self) -> Result<u8> {
        self.inner
            .get_payload_length()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Enable or disable the dynamically sized payloads feature.
    ///
    /// @param enable - If set to `true`, the statically sized payloads (set via
    /// {@link RF24.setPayloadLength | `RF24.setPayloadLength()`}) are not used.
    ///
    /// @group Configuration
    #[napi]
    pub fn set_dynamic_payloads(&mut self, enable: bool) -> Result<()> {
        self.inner
            .set_dynamic_payloads(enable)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Get the length of the next available payload in the RX FIFO.
    ///
    /// If dynamically sized payloads are not enabled (via
    /// {@link RF24.setDynamicPayloads | `RF24.setDynamicPayloads()`}),
    /// then use {@link RF24.getPayloadLength | `RF24.getPayloadLength()`}.
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
    /// It is highly recommended to avoid using pip 0 to receive because it is also
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

    /// Set the address used for transmitting on pipe 0.
    ///
    /// Only pipe 0 can be used for transmitting. It is highly recommended to
    /// avoid using pipe 0 to receive because of this.
    ///
    /// @param address - The address to receive data from.
    ///
    /// @group Basic
    #[napi]
    pub fn open_tx_pipe(&mut self, address: Buffer) -> Result<()> {
        let address = address.to_vec();
        self.inner
            .open_tx_pipe(&address)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Close the specified pipe from receiving transmissions.
    ///
    /// Use {@link RF24.openRxPipe | `RF24.openRxPipe()`} to set the address for a
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
    /// @param length - The address length is only allowed to be in range [2, 5].
    ///
    /// @group Configuration
    #[napi]
    pub fn set_address_length(&mut self, length: u8) -> Result<()> {
        self.inner
            .set_address_length(length)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Get the current configured address length (applied to all pipes).
    ///
    /// @group Configuration
    #[napi]
    pub fn get_address_length(&mut self) -> Result<u8> {
        self.inner
            .get_address_length()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Is the radio powered up?
    ///
    /// Use {@link RF24.isListening | `RF24.isListening`} to determine if
    /// the radio is in RX or TX mode.
    ///
    /// @group Configuration
    #[napi(getter)]
    pub fn is_powered(&self) -> bool {
        self.inner.is_powered()
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

    /// Configure the IRQ pin to reflect the specified {@link StatusFlags | `StatusFlags`}.
    ///
    /// If no parameter value is given, then all flags are are reflected by the IRQ pin.
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
            .set_status_flags(Some(flags.into_inner()))
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Reset the specified {@link StatusFlags | `StatusFlags`}.
    ///
    /// If no parameter value is given, then all flags are reset.
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
            .clear_status_flags(Some(flags.into_inner()))
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Update the cached value of Status flags.
    ///
    /// Use {@link RF24.getStatusFlags | `RF24.getStatusFlags`} to get the updated values.
    ///
    /// @group Advanced
    #[napi]
    pub fn update(&mut self) -> Result<()> {
        self.inner
            .update()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Get the current state of the {@link StatusFlags | `StatusFlags`}.
    ///
    /// > [!NOTE]
    /// > This function simply returns the value of the flags that was cached
    /// > from the last SPI transaction. It does not actually update the values
    /// > (from the radio) before returning them.
    /// >
    /// > Use {@link RF24.update | `RF24.update`} to update them first.
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
