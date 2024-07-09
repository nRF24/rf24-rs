use crate::{radio::EsbPayloadLength, Nrf24Error, RF24};
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

    fn get_dynamic_payload_length(&mut self) -> Result<usize, Self::PayloadLengthErrorType> {
        if !self._dynamic_payloads_enabled {
            return Ok(0);
        }
        self.spi_read(1, commands::R_RX_PL_WID)?;
        if self._buf[1] > 32 {
            return Err(Nrf24Error::BinaryCorruption);
        }
        Ok(self._buf[1] as usize)
    }
}
