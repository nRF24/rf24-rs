use anyhow::{anyhow, Error, Result};
pub use linux_embedded_hal::{
    gpio_cdev::{chips, Chip, LineRequestFlags},
    spidev::{SpiModeFlags, SpidevOptions},
    CdevPin, Delay, SpidevDevice,
};

pub struct BoardHardware {
    gpio: Chip,
}

#[cfg(feature = "linux")]
extern crate std;
#[cfg(feature = "linux")]
use std::{format, string::ToString};
#[cfg(feature = "linux")]
pub use std::{print, println};

impl BoardHardware {
    pub fn new(dev_gpio_chip: u8) -> Result<Self> {
        // get the desired "/dev/gpiochip{dev_gpio_chip}"
        let dev_gpio = chips()?
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
            .ok_or(anyhow!(
                "Could not find specified dev/gpiochip{dev_gpio_chip} for this system."
            ))??;

        Ok(BoardHardware { gpio: dev_gpio })
    }

    #[allow(
        clippy::should_implement_trait,
        reason = "Default trait does not support `-> Result<Self>`"
    )]
    pub fn default() -> Result<Self> {
        let result = Self::new(4);
        if result.is_err() {
            return Self::new(0);
        }
        result
    }

    pub fn get_spi_device(bus: u8, cs: u8) -> Result<SpidevDevice> {
        let mut spi = SpidevDevice::open(format!("/dev/spidev{bus}.{cs}")).map_err(|_| {
            anyhow!("SPI bus {bus} with CS pin option {cs} is not available in this system")
        })?;
        let config = SpidevOptions::new()
            .max_speed_hz(10000000)
            .mode(SpiModeFlags::SPI_MODE_0)
            .bits_per_word(8)
            .build();
        spi.configure(&config).map_err(Error::from)?;
        Ok(spi)
    }

    pub fn default_spi_device() -> Result<SpidevDevice> {
        BoardHardware::get_spi_device(0, 0)
    }

    pub fn get_ce_pin(&mut self, ce_pin: u32) -> Result<CdevPin> {
        let ce_line = self
            .gpio
            .get_line(ce_pin)
            .map_err(|_| anyhow!("GPIO{ce_pin} is unavailable"))?;
        let ce_line_handle = ce_line
            .request(LineRequestFlags::OUTPUT, 0, "rf24-rs")
            .map_err(Error::from)?;
        CdevPin::new(ce_line_handle).map_err(Error::from)
    }

    pub fn default_ce_pin(&mut self) -> Result<CdevPin> {
        self.get_ce_pin(22)
    }

    pub fn get_irq_pin(&mut self, ce_pin: u32) -> Result<CdevPin> {
        let irq_line = self
            .gpio
            .get_line(ce_pin)
            .map_err(|_| anyhow!("GPIO{ce_pin} is unavailable"))?;
        let irq_line_handle = irq_line
            .request(LineRequestFlags::INPUT, 0, "rf24-rs")
            .map_err(Error::from)?;
        CdevPin::new(irq_line_handle).map_err(Error::from)
    }

    pub fn default_irq_pin(&mut self) -> Result<CdevPin> {
        self.get_irq_pin(24)
    }
}
