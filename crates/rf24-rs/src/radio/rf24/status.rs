use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};

use crate::{
    radio::{prelude::EsbStatus, Nrf24Error, RF24},
    types::StatusFlags,
};

use super::{commands, registers, Config};

impl<SPI, DO, DELAY> EsbStatus for RF24<SPI, DO, DELAY>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    type StatusErrorType = Nrf24Error<SPI::Error, DO::Error>;

    fn set_status_flags(&mut self, flags: StatusFlags) -> Result<(), Self::StatusErrorType> {
        self.spi_read(1, registers::CONFIG)?;
        self._config_reg =
            Config::from_bits(self._buf[1] & 0x0F | (!flags.into_bits() & StatusFlags::IRQ_MASK));
        self.spi_write_byte(registers::CONFIG, self._config_reg.into_bits())
    }

    fn clear_status_flags(&mut self, flags: StatusFlags) -> Result<(), Self::StatusErrorType> {
        self.spi_write_byte(registers::STATUS, flags.into_bits() & StatusFlags::IRQ_MASK)
    }

    fn update(&mut self) -> Result<(), Self::StatusErrorType> {
        self.spi_read(0, commands::NOP)
    }

    fn get_status_flags(&self, flags: &mut StatusFlags) {
        *flags = self._status;
    }
}

/////////////////////////////////////////////////////////////////////////////////
/// unit tests
#[cfg(test)]
mod test {
    extern crate std;
    use crate::radio::prelude::EsbStatus;
    use crate::radio::rf24::commands;
    use crate::spi_test_expects;
    use crate::types::StatusFlags;

    use super::{registers, RF24};
    use embedded_hal_mock::eh1::delay::NoopDelay;
    use embedded_hal_mock::eh1::digital::Mock as PinMock;
    use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
    use std::vec;

    #[test]
    pub fn what_happened() {
        // Create pin
        let pin_expectations = [];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

        let spi_expectations = spi_test_expects![
            // get the RF_SETUP register value for each possible result
            (vec![commands::NOP], vec![0x70u8]),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        radio.update().unwrap();
        let mut flags = StatusFlags::default();
        radio.get_status_flags(&mut flags);
        assert!(flags.rx_dr());
        assert!(flags.tx_ds());
        assert!(flags.tx_df());
        spi_mock.done();
        pin_mock.done();
    }

    #[test]
    pub fn set_status_flags() {
        // Create pin
        let pin_expectations = [];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

        let spi_expectations = spi_test_expects![
            // read the CONFIG register value
            (vec![registers::CONFIG, 0u8], vec![0xEu8, 0xFu8]),
            // set the CONFIG register value to disable all IRQ events
            (
                vec![registers::CONFIG | commands::W_REGISTER, 0x7Fu8],
                vec![0xEu8, 0u8],
            ),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        radio.set_status_flags(StatusFlags::default()).unwrap();
        spi_mock.done();
        pin_mock.done();
    }
}
