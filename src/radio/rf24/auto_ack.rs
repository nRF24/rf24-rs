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

    fn set_auto_ack(&mut self, enable: bool) -> Result<(), Self::AutoAckErrorType> {
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
        let reg_val = self._buf[1];
        if !enable && self._ack_payloads_enabled && pipe == 0 {
            self.allow_ack_payloads(enable)?;
        }
        self.spi_write_byte(registers::EN_AA, reg_val & !mask | (mask * enable as u8))
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

/////////////////////////////////////////////////////////////////////////////////
/// unit tests
#[cfg(test)]
mod test {
    extern crate std;
    use crate::radio::prelude::{EsbAutoAck, EsbPayloadLength};
    use crate::radio::Nrf24Error;

    use super::{commands, mnemonics, registers, RF24};
    use embedded_hal_mock::eh1::delay::NoopDelay;
    use embedded_hal_mock::eh1::digital::Mock as PinMock;
    use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
    use std::vec;

    #[test]
    pub fn allow_ack_payloads() {
        // Create pin
        let pin_expectations = [];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();
        let mut ack_buf = [0x55; 3];
        ack_buf[0] = commands::W_ACK_PAYLOAD | 2;

        let spi_expectations = [
            // enable ACK payloads
            // read/write FEATURE register
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::FEATURE, 0u8], vec![0xEu8, 0u8]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![
                    registers::FEATURE | commands::W_REGISTER,
                    mnemonics::EN_ACK_PAY | mnemonics::EN_DPL,
                ],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
            // read/write DYNPD register
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::DYNPD, 0u8], vec![0xEu8, 0u8]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::DYNPD | commands::W_REGISTER, 3u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
            // write_ack_payload()
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(ack_buf.to_vec(), vec![0u8; 3]),
            SpiTransaction::transaction_end(),
            // read dynamic payload length invalid value
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![commands::R_RX_PL_WID, 0u8],
                vec![0xEu8, 0xFFu8],
            ),
            SpiTransaction::transaction_end(),
            // read dynamic payload length valid value
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![commands::R_RX_PL_WID, 0xFFu8],
                vec![0xEu8, 32u8],
            ),
            SpiTransaction::transaction_end(),
            // read EN_AA register value
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::EN_AA, 32u8], vec![0u8, 0x3Fu8]),
            SpiTransaction::transaction_end(),
            // disable ACK payloads in FEATURE register
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::FEATURE, 0x3Fu8], vec![0u8, 3u8]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::FEATURE | commands::W_REGISTER, 1u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
            // set EN_AA register with pipe 0 disabled
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::EN_AA | commands::W_REGISTER, 0x3Eu8],
                vec![0xEu8, 0x3Fu8],
            ),
            SpiTransaction::transaction_end(),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        radio.allow_ack_payloads(true).unwrap();
        let buf = [0x55; 2];
        assert!(!radio.write_ack_payload(9, &buf).unwrap());
        assert!(radio.write_ack_payload(2, &buf).unwrap());
        assert_eq!(
            radio.get_dynamic_payload_length(),
            Err(Nrf24Error::BinaryCorruption)
        );
        assert_eq!(radio.get_dynamic_payload_length().unwrap(), 32u8);
        radio.set_auto_ack_pipe(false, 9).unwrap();
        radio.set_auto_ack_pipe(false, 0).unwrap();
        spi_mock.done();
        pin_mock.done();
    }

    #[test]
    pub fn set_auto_ack() {
        // Create pin
        let pin_expectations = [];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

        let spi_expectations = [
            // enable ACK payloads
            // read/write FEATURE register
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::FEATURE, 0u8], vec![0xEu8, 0u8]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![
                    registers::FEATURE | commands::W_REGISTER,
                    mnemonics::EN_ACK_PAY | mnemonics::EN_DPL,
                ],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
            // read/write DYNPD register
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::DYNPD, 0u8], vec![0xEu8, 0u8]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::DYNPD | commands::W_REGISTER, 3u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
            // write EN_AA register value
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::EN_AA | commands::W_REGISTER, 0u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
            // disable ACK payloads in FEATURE register
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::FEATURE, 0u8],
                vec![0u8, mnemonics::EN_DPL],
            ),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::FEATURE | commands::W_REGISTER, 0u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
            // clear DYNPD register
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::DYNPD | commands::W_REGISTER, 0u8],
                vec![0xEu8, 0x3Fu8],
            ),
            SpiTransaction::transaction_end(),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        radio.allow_ack_payloads(true).unwrap();
        radio.set_auto_ack(false).unwrap();
        assert_eq!(radio.get_dynamic_payload_length().unwrap(), 0u8);
        spi_mock.done();
        pin_mock.done();
    }

    #[test]
    pub fn allow_ask_no_ack() {
        // Create pin
        let pin_expectations = [];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

        let spi_expectations = [
            // disable EN_DYN_ACK flag in FEATURE register
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(vec![registers::FEATURE, 0u8], vec![0u8, 2u8]),
            SpiTransaction::transaction_end(),
            SpiTransaction::transaction_start(),
            SpiTransaction::transfer_in_place(
                vec![registers::FEATURE | commands::W_REGISTER, 3u8],
                vec![0xEu8, 0u8],
            ),
            SpiTransaction::transaction_end(),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        radio.allow_ask_no_ack(true).unwrap();
        spi_mock.done();
        pin_mock.done();
    }
}
