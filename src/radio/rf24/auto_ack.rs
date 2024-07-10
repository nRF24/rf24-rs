use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};

use crate::radio::{
    prelude::{EsbAutoAck, EsbPayloadLength},
    Nrf24Error, RF24,
};

use super::{commands, mnemonics, registers};

impl<SPI, DO, DELAY> EsbAutoAck for RF24<SPI, DO, DELAY>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    type AutoAckErrorType = Nrf24Error<SPI::Error, DO::Error>;

    fn allow_ack_payloads(&mut self, enable: bool) -> Result<(), Self::AutoAckErrorType> {
        if self._ack_payloads_enabled != enable {
            if enable {
                self.spi_read(1, registers::FEATURE)?;
                let mut reg_val = self._buf[1] | mnemonics::EN_ACK_PAY | mnemonics::EN_DPL;
                self.spi_write_byte(registers::FEATURE, reg_val)?;

                // Enable dynamic payload on pipes 0 & 1
                self.spi_read(1, registers::DYNPD)?;
                reg_val = self._buf[1] | 3;
                self.spi_write_byte(registers::DYNPD, reg_val)?;
                self._dynamic_payloads_enabled = true;
            } else {
                // disable ack payloads (leave dynamic payload features as is)
                self.spi_read(1, registers::FEATURE)?;
                let reg_val = self._buf[1] & !mnemonics::EN_ACK_PAY;
                self.spi_write_byte(registers::FEATURE, reg_val)?;
            }
            self._ack_payloads_enabled = enable;
        }
        Ok(())
    }

    fn set_auto_ack(&mut self, enable: bool) -> Result<(), Nrf24Error<<SPI>::Error, <DO>::Error>> {
        self.spi_write_byte(registers::EN_AA, 0x3F * enable as u8)?;
        // accommodate ACK payloads feature
        if !enable && self._ack_payloads_enabled {
            self.set_dynamic_payloads(false)?;
        }
        Ok(())
    }

    fn set_auto_ack_pipe(&mut self, enable: bool, pipe: u8) -> Result<(), Self::AutoAckErrorType> {
        if pipe > 5 {
            return Ok(());
        }
        self.spi_read(1, registers::EN_AA)?;
        let mask = 1 << pipe;
        if !enable && self._ack_payloads_enabled && pipe == 0 {
            self.allow_ack_payloads(enable)?;
        }
        let reg_val = self._buf[1] & !mask | (mask * enable as u8);
        self.spi_write_byte(registers::EN_AA, reg_val)
    }

    fn allow_ask_no_ack(&mut self, enable: bool) -> Result<(), Self::AutoAckErrorType> {
        self.spi_read(1, registers::FEATURE)?;
        self.spi_write_byte(registers::FEATURE, self._buf[1] & !1 | enable as u8)
    }

    fn write_ack_payload(&mut self, pipe: u8, buf: &[u8]) -> Result<bool, Self::AutoAckErrorType> {
        if self._ack_payloads_enabled && pipe <= 5 {
            let len = {
                let buf_len = buf.len();
                if buf_len > 32 {
                    32usize
                } else {
                    buf_len
                }
            };
            self.spi_write_buf(commands::W_ACK_PAYLOAD | pipe, &buf[..len])?;
            return Ok(0 == self._status & 1);
        }
        Ok(false)
    }

    fn set_auto_retries(&mut self, delay: u8, count: u8) -> Result<(), Self::AutoAckErrorType> {
        let out = {
            if count > 15 {
                15
            } else {
                count
            }
        } | ({
            if delay > 15 {
                15
            } else {
                delay
            }
        } << 4);
        self.spi_write_byte(registers::SETUP_RETR, out)
    }
}
