use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};

use crate::radio::{prelude::EsbStatus, Nrf24Error, RF24};

use super::{commands, mnemonics, registers};

impl<SPI, DO, DELAY> EsbStatus for RF24<SPI, DO, DELAY>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    type StatusErrorType = Nrf24Error<SPI::Error, DO::Error>;

    /// Configure which status flags trigger the radio's IRQ pin.
    ///
    /// The supported interrupt events correspond to the parameters:
    /// - `rx_dr` means "RX Data Ready"
    /// - `tx_ds` means "TX Data Sent"
    /// - `tx_df` means "TX Data Failed" to send
    ///
    /// Set any parameter to `false` to have the IRQ pin ignore the corresponding event.
    /// By default, all events are enabled and will trigger the IRQ pin, a behavior
    /// equivalent to `set_status_flags(true, true, true)`.
    fn set_status_flags(
        &mut self,
        rx_dr: bool,
        tx_ds: bool,
        tx_df: bool,
    ) -> Result<(), Self::StatusErrorType> {
        self.spi_read(1, registers::CONFIG)?;
        let mut new_config = self._buf[1] & !(3 << 4);
        if !rx_dr {
            new_config |= mnemonics::MASK_RX_DR;
        }
        if !tx_ds {
            new_config |= mnemonics::MASK_TX_DS;
        }
        if !tx_df {
            new_config |= mnemonics::MASK_MAX_RT;
        }
        self.spi_write_byte(registers::CONFIG, new_config)
    }

    /// Clear the radio's IRQ status flags
    ///
    /// This needs to be done when the event has been handled.
    ///
    /// The supported interrupt events correspond to the parameters:
    /// - `rx_dr` means "RX Data Ready"
    /// - `tx_ds` means "TX Data Sent"
    /// - `tx_df` means "TX Data Failed" to send
    ///
    /// Set any parameter to `true` to clear the corresponding interrupt event.
    /// Setting a parameter to `false` will leave the corresponding status flag untouched.
    /// This means that the IRQ pin can remain active (LOW) when multiple events occurred
    /// but only flag was cleared.
    fn clear_status_flags(
        &mut self,
        rx_dr: bool,
        tx_ds: bool,
        tx_df: bool,
    ) -> Result<(), Self::StatusErrorType> {
        let new_config = (mnemonics::MASK_RX_DR * rx_dr as u8)
            | (mnemonics::MASK_TX_DS * tx_ds as u8)
            | (mnemonics::MASK_MAX_RT * tx_df as u8);
        self.spi_write_byte(registers::STATUS, new_config)
    }

    fn update(&mut self) -> Result<(), Self::StatusErrorType> {
        self.spi_read(0, commands::NOP)
    }

    fn get_status_flags(
        &mut self,
        rx_dr: &mut Option<bool>,
        tx_ds: &mut Option<bool>,
        tx_df: &mut Option<bool>,
    ) -> Result<(), Self::StatusErrorType> {
        if let Some(f) = rx_dr {
            *f = self._status & mnemonics::MASK_RX_DR > 0;
        }
        if let Some(f) = tx_ds {
            *f = self._status & mnemonics::MASK_TX_DS > 0;
        }
        if let Some(f) = tx_df {
            *f = self._status & mnemonics::MASK_MAX_RT > 0;
        }
        Ok(())
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
        let mut rx_dr = Some(false);
        let mut tx_ds = Some(false);
        let mut tx_df = Some(false);
        radio
            .get_status_flags(&mut rx_dr, &mut tx_ds, &mut tx_df)
            .unwrap();
        assert!(rx_dr.is_some_and(|rv| rv));
        assert!(tx_ds.is_some_and(|rv| rv));
        assert!(tx_df.is_some_and(|rv| rv));
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

        let spi_expectations = [
            // read the CONFIG register value
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::CONFIG, 0u8], vec![0xEu8, 0xFu8]),
            SpiTransaction::transaction_end(),
            // set the CONFIG register value to disable all IRQ events
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::CONFIG | commands::W_REGISTER, 0x7Fu8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        radio.set_status_flags(false, false, false).unwrap();
        spi_mock.done();
        pin_mock.done();
    }
}
