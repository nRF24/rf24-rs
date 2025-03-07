#![cfg(feature = "rp2040")]
use crate::hal_impl_trait::HardwareImpl;
use anyhow::Result;

use core::cell::RefCell;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::peripherals::{PIN_10, PIN_25, SPI1};
use embassy_rp::spi::{Blocking, Config, Spi};
use embassy_rp::Peripherals;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::blocking_mutex::Mutex;

pub struct BoardHardware<'b> {
    peri: Peripherals,
    spi_bus_mutex: Mutex<NoopRawMutex, RefCell<Spi<'b, SPI1, Blocking>>>,
}

impl BoardHardware<'_> {
    pub fn new() -> Self {
        let peri = embassy_rp::init(Default::default());

        let clk = peri.PIN_10;
        let mosi = peri.PIN_11;
        let miso = peri.PIN_12;
        let mut spi_config = Config::default();
        spi_config.frequency = 10_000_000;
        let spi = Spi::new_blocking(peri.SPI1, clk, mosi, miso, spi_config);
        let spi_bus_mutex: Mutex<NoopRawMutex, RefCell<_>> = Mutex::new(RefCell::new(spi));

        BoardHardware {
            peri,
            spi_bus_mutex,
        }
    }
}

impl HardwareImpl for BoardHardware {
    fn new() -> Result<Self>;

    fn default_ce_pin(&self) -> Result<impl OutputPin> {
        let ce = self.peri.PIN_9;
        let ce_pin = Output::new(ce, Level::Low);
        Ok(ce_pin)
    }

    fn default_spi_device(&self) -> Result<impl SpiDevice> {
        let cs = self.peri.PIN_25;
        let cs_pin = Output::new(cs, Level::High);
        let spi_device = SpiDevice::new(&self.spi_bus_mutex, cs_pin);
        Ok(spi_device)
    }

    fn default_irq_pin(&self) -> Result<impl InputPin> {
        let irq = self.peri.PIN_13;
        let irq_pin = Input::new(irq, Pull::None);
        Ok(irq_pin)
    }
}
