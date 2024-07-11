use crate::radio::{prelude::EsbPayloadLength, Nrf24Error, RF24};
use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};

use super::{commands, mnemonics, registers};

impl<SPI, DO, DELAY> EsbPayloadLength for RF24<SPI, DO, DELAY>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    type PayloadLengthErrorType = Nrf24Error<SPI::Error, DO::Error>;

    fn set_payload_length(&mut self, length: u8) -> Result<(), Self::PayloadLengthErrorType> {
        let len = {
            if length > 32 {
                32
            } else {
                length
            }
        };
        for i in 0..6 {
            self.spi_write_byte(registers::RX_PW_P0 + i, len)?;
        }
        self._payload_length = len;
        Ok(())
    }

    fn get_payload_length(&mut self) -> Result<u8, Self::PayloadLengthErrorType> {
        self.spi_read(1, registers::RX_PW_P0)?;
        Ok(self._buf[1])
    }

    fn set_dynamic_payloads(&mut self, enable: bool) -> Result<(), Self::PayloadLengthErrorType> {
        self.spi_read(1, registers::FEATURE)?;
        let reg_val = self._buf[1];
        if enable != (reg_val & mnemonics::EN_DPL == mnemonics::EN_DPL) {
            self.spi_write_byte(
                registers::FEATURE,
                reg_val & !mnemonics::EN_DPL | (mnemonics::EN_DPL * enable as u8),
            )?;
        }
        self.spi_write_byte(registers::DYNPD, 0x3F * enable as u8)?;
        self._dynamic_payloads_enabled = enable;
        Ok(())
    }

    fn get_dynamic_payload_length(&mut self) -> Result<u8, Self::PayloadLengthErrorType> {
        if !self._dynamic_payloads_enabled {
            return Ok(0);
        }
        self.spi_read(1, commands::R_RX_PL_WID)?;
        if self._buf[1] > 32 {
            return Err(Nrf24Error::BinaryCorruption);
        }
        Ok(self._buf[1])
    }
}

/////////////////////////////////////////////////////////////////////////////////
/// unit tests
#[cfg(test)]
mod test {
    extern crate std;
    use crate::radio::prelude::EsbPayloadLength;

    use super::{commands, registers, RF24};
    use embedded_hal_mock::eh1::delay::NoopDelay;
    use embedded_hal_mock::eh1::digital::Mock as PinMock;
    use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
    use std::vec;

    // dynamic payloads are already tested in EsbAutoAck trait as
    // these features' behaviors are interdependent.

    #[test]
    pub fn set_payload_length() {
        // Create pin
        let pin_expectations = [];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

        let spi_expectations = [
            // set payload length to 32 bytes on all pipes
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::RX_PW_P0 + 0 | commands::W_REGISTER, 32u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::RX_PW_P0 + 1 | commands::W_REGISTER, 32u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::RX_PW_P0 + 2 | commands::W_REGISTER, 32u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::RX_PW_P0 + 3 | commands::W_REGISTER, 32u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::RX_PW_P0 + 4 | commands::W_REGISTER, 32u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::RX_PW_P0 + 5 | commands::W_REGISTER, 32u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
            // get payload length for all pipe 0 (because all pipes will use the same static length)
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::RX_PW_P0 + 0, 0u8],
                vec![0xEu8, 32u8],
            ),
            SpiTransaction::transaction_end(),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        radio.set_payload_length(76).unwrap();
        assert_eq!(radio.get_payload_length().unwrap(), 32u8);
        spi_mock.done();
        pin_mock.done();
    }
}
