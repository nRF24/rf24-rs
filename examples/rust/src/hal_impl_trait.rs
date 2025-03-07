
/// A set of common functions that are guaranteed to be implemented by the hardware-specific implementations.
pub trait HardwareImpl {
    fn new() -> Result<Self>;

    fn default_ce_pin(&self) -> Result<impl OutputPin>;

    fn default_spi_device(&self) -> Result<impl SpiDevice>;

    fn default_irq_pin(&self) -> Result<impl InputPin>;
}
