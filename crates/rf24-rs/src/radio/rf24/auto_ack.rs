use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};

use crate::radio::{prelude::EsbAutoAck, Nrf24Error, RF24};

use super::{commands, registers, Feature};

impl<SPI, DO, DELAY> EsbAutoAck for RF24<SPI, DO, DELAY>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    type AutoAckErrorType = Nrf24Error<SPI::Error, DO::Error>;

    fn set_ack_payloads(&mut self, enable: bool) -> Result<(), Self::AutoAckErrorType> {
        if self._feature.ack_payloads() != enable {
            self.spi_read(1, registers::FEATURE)?;
            self._feature =
                Feature::from_bits(self._feature.into_bits() & !Feature::REG_MASK | self._buf[1])
                    .with_ack_payloads(enable);
            self.spi_write_byte(
                registers::FEATURE,
                self._feature.into_bits() & Feature::REG_MASK,
            )?;

            if enable {
                // Enable dynamic payload on all pipes
                self.spi_write_byte(registers::DYNPD, 0x3F)?;
            }
            // else disable ack payloads, but leave dynamic payload features as is
        }
        Ok(())
    }

    fn get_ack_payloads(&self) -> bool {
        self._feature.ack_payloads()
    }

    fn set_auto_ack(&mut self, enable: bool) -> Result<(), Self::AutoAckErrorType> {
        self.spi_write_byte(registers::EN_AA, 0x3F * enable as u8)?;
        // accommodate ACK payloads feature
        if !enable && self._feature.ack_payloads() {
            self.set_ack_payloads(false)?;
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
        if !enable && self._feature.ack_payloads() && pipe == 0 {
            self.set_ack_payloads(enable)?;
        }
        self.spi_write_byte(registers::EN_AA, reg_val & !mask | (mask * enable as u8))
    }

    fn allow_ask_no_ack(&mut self, enable: bool) -> Result<(), Self::AutoAckErrorType> {
        self.spi_read(1, registers::FEATURE)?;
        self.spi_write_byte(registers::FEATURE, self._buf[1] & !1 | enable as u8)
    }

    fn write_ack_payload(&mut self, pipe: u8, buf: &[u8]) -> Result<bool, Self::AutoAckErrorType> {
        if self._feature.ack_payloads() && pipe <= 5 {
            let len = buf.len().min(32);
            self.spi_write_buf(commands::W_ACK_PAYLOAD | pipe, &buf[..len])?;
            return Ok(!self._status.tx_full());
        }
        Ok(false)
    }

    fn set_auto_retries(&mut self, delay: u8, count: u8) -> Result<(), Self::AutoAckErrorType> {
        self.spi_write_byte(registers::SETUP_RETR, count.min(15) | (delay.min(15) << 4))
    }
}

/////////////////////////////////////////////////////////////////////////////////
/// unit tests
#[cfg(test)]
mod test {
    extern crate std;
    use super::{commands, registers, EsbAutoAck};
    use crate::{radio::prelude::EsbPayloadLength, spi_test_expects, test::mk_radio};
    use embedded_hal_mock::eh1::spi::Transaction as SpiTransaction;
    use std::vec;

    const EN_ACK_PAY: u8 = 1 << 1;
    const EN_DPL: u8 = 1 << 2;

    #[test]
    pub fn allow_ack_payloads() {
        let mut ack_buf = [0x55; 3];
        let valid_pipe = 2;
        ack_buf[0] = commands::W_ACK_PAYLOAD | valid_pipe;

        let spi_expectations = spi_test_expects![
            // enable ACK payloads
            // read/write FEATURE register
            (vec![registers::FEATURE, 0u8], vec![0xEu8, 0u8]),
            (
                vec![
                    registers::FEATURE | commands::W_REGISTER,
                    EN_ACK_PAY | EN_DPL,
                ],
                vec![0xEu8, 0u8],
            ),
            // write DYNPD register
            (
                vec![registers::DYNPD | commands::W_REGISTER, 0x3Fu8],
                vec![0xEu8, 0u8],
            ),
            // write_ack_payload()
            (ack_buf.to_vec(), vec![0u8; 3]),
            // write_ack_payload() again but with TX FIFO as full
            (ack_buf.to_vec(), vec![1u8; 3]),
            // read EN_AA register value
            (vec![registers::EN_AA, 1u8], vec![0u8, 0x3Fu8]),
            // disable ACK payloads in FEATURE register
            (
                vec![registers::FEATURE, 0x3Fu8],
                vec![0u8, EN_ACK_PAY | EN_DPL | 1]
            ),
            (
                vec![registers::FEATURE | commands::W_REGISTER, EN_DPL | 1],
                vec![0xEu8, 0u8],
            ),
            // set EN_AA register with pipe 0 disabled
            (
                vec![registers::EN_AA | commands::W_REGISTER, 0x3Eu8],
                vec![0xEu8, 0u8],
            ),
            // read EN_AA register value
            (vec![registers::EN_AA, 0u8], vec![0u8, 0x3Eu8]),
            // set EN_AA register with pipes 0 and 1 disabled
            (
                vec![registers::EN_AA | commands::W_REGISTER, 0x3Cu8],
                vec![0xEu8, 0u8],
            ),
        ];
        let mocks = mk_radio(&[], &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        radio.set_ack_payloads(true).unwrap();
        // do again for region coverage (should result in Ok non-op)
        radio.set_ack_payloads(true).unwrap();
        let buf = &ack_buf[1..3];
        // write ACK payload to invalid pipe (results in Ok non-op)
        assert!(!radio.write_ack_payload(9, buf).unwrap());
        // write ACK payload to valid pipe (test will also mark TX FIFO as full)
        assert!(radio.write_ack_payload(valid_pipe, buf).unwrap());
        // write ACK payload to valid pipe with TX FIFO full
        assert!(!radio.write_ack_payload(valid_pipe, buf).unwrap());
        // disable invalid pipe number (results in Ok non-op)
        radio.set_auto_ack_pipe(false, 9).unwrap();
        // disable auto-ack on pipe 0 (also disables ack_payloads)
        radio.set_auto_ack_pipe(false, 0).unwrap();
        // disable pipe 1 for region coverage
        radio.set_auto_ack_pipe(false, 1).unwrap();
        spi.done();
        ce_pin.done();
    }

    #[test]
    pub fn set_auto_ack() {
        let spi_expectations = spi_test_expects![
            // enable ACK payloads
            // read/write FEATURE register
            (vec![registers::FEATURE, 0u8], vec![0xEu8, 0u8]),
            (
                vec![
                    registers::FEATURE | commands::W_REGISTER,
                    EN_ACK_PAY | EN_DPL,
                ],
                vec![0xEu8, 0u8],
            ),
            // write DYNPD register
            (
                vec![registers::DYNPD | commands::W_REGISTER, 0x3Fu8],
                vec![0xEu8, 0u8],
            ),
            // write EN_AA register value
            (
                vec![registers::EN_AA | commands::W_REGISTER, 0u8],
                vec![0xEu8, 0u8],
            ),
            // disable ACK payloads in FEATURE register
            (
                vec![registers::FEATURE, 0u8],
                vec![0u8, EN_ACK_PAY | EN_DPL]
            ),
            (
                vec![registers::FEATURE | commands::W_REGISTER, EN_DPL],
                vec![0xEu8, 0u8],
            ),
            // read RX_PL_WID
            (vec![commands::R_RX_PL_WID, 0u8], vec![0xEu8, 32]),
        ];
        let mocks = mk_radio(&[], &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        radio.set_ack_payloads(true).unwrap();
        assert!(radio.get_ack_payloads());
        radio.set_auto_ack(false).unwrap();
        assert_eq!(radio.get_dynamic_payload_length().unwrap(), 32u8);
        spi.done();
        ce_pin.done();
    }

    #[test]
    pub fn allow_ask_no_ack() {
        let spi_expectations = spi_test_expects![
            // disable EN_DYN_ACK flag in FEATURE register
            (vec![registers::FEATURE, 0u8], vec![0u8, EN_ACK_PAY]),
            (
                vec![registers::FEATURE | commands::W_REGISTER, EN_ACK_PAY | 1],
                vec![0xEu8, 0u8],
            ),
        ];
        let mocks = mk_radio(&[], &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        radio.allow_ask_no_ack(true).unwrap();
        spi.done();
        ce_pin.done();
    }
}
