use anyhow::{anyhow, Error, Result};
use linux_embedded_hal::{
    gpio_cdev::{chips, Chip, LineRequestFlags},
    spidev::{SpiModeFlags, SpidevOptions},
    CdevPin, Delay, SpidevDevice,
};

pub struct BoardHardware {
    pub spi: SpidevDevice,
    pub ce_pin: CdevPin,
    #[allow(dead_code)]
    gpio: Chip,
    pub delay: Delay,
}

impl BoardHardware {
    pub fn new(dev_gpio_chip: u8, ce_pin: u32, dev_spi_bus: u8, cs_pin: u8) -> Result<Self> {
        // get the desired "dev/gpiochip{dev_gpio_chip}"
        let mut dev_gpio = chips()?
            .find(|chip| {
                if let Ok(chip) = chip {
                    if chip.path().ends_with(dev_gpio_chip.to_string()) {
                        return true;
                    }
                }
                false
            })
            .ok_or(anyhow!(
                "Could not find specified dev/gpiochip{dev_gpio_chip} for this system."
            ))??;
        let ce_line = dev_gpio
            .get_line(ce_pin)
            .map_err(|_| anyhow!("GPIO{ce_pin} is unavailable"))?;
        let ce_line_handle = ce_line
            .request(LineRequestFlags::OUTPUT, 0, "rf24-rs")
            .map_err(Error::from)?;
        let ce_pin = CdevPin::new(ce_line_handle).map_err(Error::from)?;

        let mut spi =
            SpidevDevice::open(format!("/dev/spidev{dev_spi_bus}.{cs_pin}")).map_err(|_| {
                anyhow!(
                "SPI bus {dev_spi_bus} with CS pin option {cs_pin} is not available in this system"
            )
            })?;
        let config = SpidevOptions::new()
            .max_speed_hz(10000000)
            .mode(SpiModeFlags::SPI_MODE_0)
            .bits_per_word(8)
            .build();
        spi.configure(&config).map_err(Error::from)?;

        Ok(BoardHardware {
            spi,
            ce_pin,
            gpio: dev_gpio,
            delay: Delay,
        })
    }

    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Result<Self> {
        Self::new(
            option_env!("RF24_EXAMPLE_GPIO_CHIP")
                .unwrap_or("0")
                .parse()?,
            22,
            0,
            0,
        )
    }
}
