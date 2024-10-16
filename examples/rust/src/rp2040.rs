use core::cell::RefCell;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{PIN_10, PIN_25, SPI1};
use embassy_rp::spi::{Blocking, Config, Spi};
use embassy_rp::Peripherals;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::blocking_mutex::Mutex;

pub struct BoardHardware<'b> {
    peri: Peripherals,
    spi_bus_mutex: Mutex<NoopRawMutex, RefCell<Spi<'b, SPI1, Blocking>>>,
    pub spi_device: SpiDevice<'b, NoopRawMutex, Spi<'b, SPI1, Blocking>, Output<'b, PIN_10>>,
    pub ce_pin: Output<'b, PIN_25>,
}

impl BoardHardware<'_> {
    pub fn new() -> Self {
        let peri = embassy_rp::init(Default::default());

        let ce = peri.PIN_9;
        let ce_pin = Output::new(ce, Level::Low);
        
        let clk = peri.PIN_10;
        let mosi = peri.PIN_11;
        let miso = peri.PIN_12;
        let mut spi_config = Config::default();
        spi_config.frequency = 10_000_000;
        let spi = Spi::new_blocking(peri.SPI1, clk, mosi, miso, spi_config);
        let spi_bus_mutex: Mutex<NoopRawMutex, RefCell<_>> = Mutex::new(RefCell::new(spi));
        let cs = peri.PIN_25;
        let cs_pin = Output::new(cs, Level::High);
        let spi_device = SpiDevice::new(&spi_bus_mutex, cs_pin);

        BoardHardware {
            peri,
            spi_bus_mutex,
            spi_device,
            ce_pin,
        }
    }
}
