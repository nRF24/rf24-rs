#![cfg(target_os = "linux")]

use crate::enums::{
    AvailablePipe, HardwareConfig, NodeCrcLength, NodeDataRate, NodeFifoState, NodePaLevel,
    NodeStatusFlags, WriteConfig,
};
use linux_embedded_hal::{
    gpio_cdev::{chips, LineRequestFlags},
    spidev::{SpiModeFlags, SpidevOptions},
    CdevPin, Delay, SpidevDevice,
};
use napi::{bindgen_prelude::Buffer, Error, Result, Status};

use rf24_rs::radio::{prelude::*, RF24};
use rf24_rs::StatusFlags;

#[napi(js_name = "RF24")]
pub struct NodeRF24 {
    inner: RF24<SpidevDevice, CdevPin, Delay>,
    read_buf: [u8; 32],
}

#[napi]
impl NodeRF24 {
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
                    println!("{:?}", chip.path());
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
            inner: RF24::new(ce_pin, spi, Delay),
            read_buf: [0u8; 32],
        })
    }

    #[napi]
    pub fn begin(&mut self) -> Result<()> {
        self.inner
            .init()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi(getter)]
    pub fn is_listening(&self) -> bool {
        self.inner.is_listening()
    }

    #[napi]
    pub fn start_listening(&mut self) -> Result<()> {
        self.inner
            .start_listening()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn stop_listening(&mut self) -> Result<()> {
        self.inner
            .stop_listening()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn send(&mut self, buf: Buffer, ask_no_ack: Option<bool>) -> Result<bool> {
        let buf = buf.to_vec();
        self.inner
            .send(&buf, ask_no_ack.unwrap_or_default())
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

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

    #[napi]
    pub fn read(&mut self, len: Option<u8>) -> Result<Buffer> {
        let len = self
            .inner
            .read(&mut self.read_buf, len)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))?;
        Ok(Buffer::from(&self.read_buf[0..len as usize]))
    }

    #[napi]
    pub fn resend(&mut self) -> Result<bool> {
        self.inner
            .resend()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn rewrite(&mut self) -> Result<()> {
        self.inner
            .rewrite()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn get_last_arc(&mut self) -> Result<u8> {
        self.inner
            .get_last_arc()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi(getter)]
    pub fn is_plus_variant(&self) -> bool {
        self.inner.is_plus_variant()
    }

    #[napi(getter)]
    pub fn rpd(&mut self) -> Result<bool> {
        self.inner
            .test_rpd()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn start_carrier_wave(&mut self, level: NodePaLevel, channel: u8) -> Result<()> {
        self.inner
            .start_carrier_wave(level.into_inner(), channel)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn stop_carrier_wave(&mut self) -> Result<()> {
        self.inner
            .stop_carrier_wave()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn set_lna(&mut self, enable: bool) -> Result<()> {
        self.inner
            .set_lna(enable)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn allow_ack_payloads(&mut self, enable: bool) -> Result<()> {
        self.inner
            .allow_ack_payloads(enable)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn set_auto_ack(&mut self, enable: bool) -> Result<()> {
        self.inner
            .set_auto_ack(enable)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn set_auto_ack_pipe(&mut self, enable: bool, pipe: u8) -> Result<()> {
        self.inner
            .set_auto_ack_pipe(enable, pipe)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn allow_ask_no_ack(&mut self, enable: bool) -> Result<()> {
        self.inner
            .allow_ask_no_ack(enable)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn write_ack_payload(&mut self, pipe: u8, buf: Buffer) -> Result<bool> {
        let buf = buf.to_vec();
        self.inner
            .write_ack_payload(pipe, &buf)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn set_auto_retries(&mut self, delay: u8, count: u8) -> Result<()> {
        self.inner
            .set_auto_retries(delay, count)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn set_channel(&mut self, channel: u8) -> Result<()> {
        self.inner
            .set_channel(channel)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn get_channel(&mut self) -> Result<u8> {
        self.inner
            .get_channel()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn get_crc_length(&mut self) -> Result<NodeCrcLength> {
        self.inner
            .get_crc_length()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
            .map(|e| NodeCrcLength::from_inner(e))
    }

    #[napi]
    pub fn set_crc_length(&mut self, crc_length: NodeCrcLength) -> Result<()> {
        self.inner
            .set_crc_length(crc_length.into_inner())
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn get_data_rate(&mut self) -> Result<NodeDataRate> {
        self.inner
            .get_data_rate()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
            .map(|e| NodeDataRate::from_inner(e))
    }

    #[napi]
    pub fn set_data_rate(&mut self, data_rate: NodeDataRate) -> Result<()> {
        self.inner
            .set_data_rate(data_rate.into_inner())
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn available(&mut self) -> Result<bool> {
        self.inner
            .available()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

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

    /// Use this to discard all 3 layers in the radio's RX FIFO.
    #[napi]
    pub fn flush_rx(&mut self) -> Result<()> {
        self.inner
            .flush_rx()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// Use this to discard all 3 layers in the radio's TX FIFO.
    #[napi]
    pub fn flush_tx(&mut self) -> Result<()> {
        self.inner
            .flush_tx()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn get_fifo_state(&mut self, about_tx: bool) -> Result<NodeFifoState> {
        self.inner
            .get_fifo_state(about_tx)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
            .map(|e| NodeFifoState::from_inner(e))
    }

    #[napi]
    pub fn get_pa_level(&mut self) -> Result<NodePaLevel> {
        self.inner
            .get_pa_level()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
            .map(|e| NodePaLevel::from_inner(e))
    }

    #[napi]
    pub fn set_pa_level(&mut self, pa_level: NodePaLevel) -> Result<()> {
        self.inner
            .set_pa_level(pa_level.into_inner())
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn set_payload_length(&mut self, length: u8) -> Result<()> {
        self.inner
            .set_payload_length(length)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn get_payload_length(&mut self) -> Result<u8> {
        self.inner
            .get_payload_length()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn set_dynamic_payloads(&mut self, enable: bool) -> Result<()> {
        self.inner
            .set_dynamic_payloads(enable)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn get_dynamic_payload_length(&mut self) -> Result<u8> {
        self.inner
            .get_dynamic_payload_length()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn open_rx_pipe(&mut self, pipe: u8, address: Buffer) -> Result<()> {
        let address = address.to_vec();
        self.inner
            .open_rx_pipe(pipe, &address)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn open_tx_pipe(&mut self, address: Buffer) -> Result<()> {
        let address = address.to_vec();
        self.inner
            .open_tx_pipe(&address)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    /// If the given `pipe` number is  not in range [0, 5], then this function does nothing.
    #[napi]
    pub fn close_rx_pipe(&mut self, pipe: u8) -> Result<()> {
        self.inner
            .close_rx_pipe(pipe)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn set_address_length(&mut self, length: u8) -> Result<()> {
        self.inner
            .set_address_length(length)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn get_address_length(&mut self) -> Result<u8> {
        self.inner
            .get_address_length()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi(getter)]
    pub fn is_powered(&self) -> bool {
        self.inner.is_powered()
    }

    #[napi]
    pub fn power_down(&mut self) -> Result<()> {
        self.inner
            .power_down()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn power_up(&mut self, delay: Option<u32>) -> Result<()> {
        self.inner
            .power_up(delay)
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn set_status_flags(&mut self, flags: Option<NodeStatusFlags>) -> Result<()> {
        let flags = flags.unwrap_or(NodeStatusFlags {
            rx_dr: Some(true),
            tx_ds: Some(true),
            tx_df: Some(true),
        });
        self.inner
            .set_status_flags(Some(flags.into_inner()))
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn clear_status_flags(&mut self, flags: Option<NodeStatusFlags>) -> Result<()> {
        let flags = flags.unwrap_or(NodeStatusFlags {
            rx_dr: Some(true),
            tx_ds: Some(true),
            tx_df: Some(true),
        });
        self.inner
            .clear_status_flags(Some(flags.into_inner()))
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn update(&mut self) -> Result<()> {
        self.inner
            .update()
            .map_err(|e| Error::new(Status::GenericFailure, format!("{e:?}")))
    }

    #[napi]
    pub fn get_status_flags(&mut self) -> NodeStatusFlags {
        let mut flags = StatusFlags::default();
        self.inner.get_status_flags(&mut flags);
        NodeStatusFlags::from_inner(flags)
    }
}
