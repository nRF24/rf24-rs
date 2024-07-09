use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};

use crate::{radio::EsbStatus, Nrf24Error, RF24};

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
        let mut new_config = self._buf[1] & (3 << 4);
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
        let mut new_config = 0;
        if rx_dr {
            new_config |= mnemonics::MASK_RX_DR;
        }
        if tx_ds {
            new_config |= mnemonics::MASK_TX_DS;
        }
        if tx_df {
            new_config |= mnemonics::MASK_MAX_RT;
        }
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
