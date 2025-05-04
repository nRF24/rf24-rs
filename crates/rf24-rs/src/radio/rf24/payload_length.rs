use crate::radio::{prelude::EsbPayloadLength, Nrf24Error, RF24};
use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};

use super::{commands, registers, Feature};

impl<SPI, DO, DELAY> EsbPayloadLength for RF24<SPI, DO, DELAY>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    fn set_payload_length(&mut self, length: u8) -> Result<(), Self::Error> {
        let len = length.clamp(1, 32);
        for i in 0..6 {
            self.spi_write_byte(registers::RX_PW_P0 + i, len)?;
        }
        self.payload_length = len;
        Ok(())
    }

    fn get_payload_length(&mut self) -> Result<u8, Self::Error> {
        self.spi_read(1, registers::RX_PW_P0)?;
        Ok(self.buf[1])
    }

    fn set_dynamic_payloads(&mut self, enable: bool) -> Result<(), Self::Error> {
        self.spi_read(1, registers::FEATURE)?;
        self.feature =
            Feature::from_bits(self.feature.into_bits() & !Feature::REG_MASK | self.buf[1])
                .with_dynamic_payloads(enable);
        self.spi_write_byte(
            registers::FEATURE,
            self.feature.into_bits() & Feature::REG_MASK,
        )?;
        self.spi_write_byte(registers::DYNPD, 0x3F * enable as u8)?;
        Ok(())
    }

    fn get_dynamic_payloads(&self) -> bool {
        self.feature.dynamic_payloads()
    }

    fn get_dynamic_payload_length(&mut self) -> Result<u8, Self::Error> {
        self.spi_read(1, commands::R_RX_PL_WID)?;
        if self.buf[1] > 32 {
            return Err(Nrf24Error::BinaryCorruption);
        }
        Ok(self.buf[1])
    }
}

/////////////////////////////////////////////////////////////////////////////////
/// unit tests
#[cfg(test)]
mod test {
    extern crate std;
    use super::{commands, registers, EsbPayloadLength, Nrf24Error};
    use crate::{radio::prelude::EsbAutoAck, spi_test_expects, test::mk_radio};
    use embedded_hal_mock::eh1::spi::Transaction as SpiTransaction;
    use std::vec;

    const EN_ACK_PAY: u8 = 1 << 1;
    const EN_DPL: u8 = 1 << 2;

    #[test]
    fn dynamic_payloads() {
        let spi_expectations = spi_test_expects![
            // set_dynamic_payloads(true)
            (vec![registers::FEATURE, 0], vec![0xEu8, 0],),
            (
                vec![registers::FEATURE | commands::W_REGISTER, EN_DPL],
                vec![0xEu8, 0],
            ),
            (
                vec![registers::DYNPD | commands::W_REGISTER, 0x3F],
                vec![0xEu8, 0],
            ),
            // read dynamic payload length invalid value
            (vec![commands::R_RX_PL_WID, 0], vec![0xEu8, 0xFF]),
            // read dynamic payload length valid value
            (vec![commands::R_RX_PL_WID, 0xFF], vec![0xEu8, 32]),
            // set_dynamic_payloads(false)
            (
                vec![registers::FEATURE, 32],
                vec![0xEu8, EN_ACK_PAY | EN_DPL],
            ),
            (
                vec![registers::FEATURE | commands::W_REGISTER, 0],
                vec![0xEu8, 0],
            ),
            (
                vec![registers::DYNPD | commands::W_REGISTER, 0],
                vec![0xEu8, 0],
            ),
        ];

        let mocks = mk_radio(&[], &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        radio.set_dynamic_payloads(true).unwrap();
        assert!(radio.get_dynamic_payloads());
        assert_eq!(
            radio.get_dynamic_payload_length(),
            Err(Nrf24Error::BinaryCorruption)
        );
        assert_eq!(radio.get_dynamic_payload_length().unwrap(), 32);
        radio.set_dynamic_payloads(false).unwrap();
        assert!(!radio.get_dynamic_payloads());
        assert!(!radio.get_ack_payloads());
        spi.done();
        ce_pin.done();
    }

    #[test]
    pub fn set_payload_length() {
        let mut spi_expectations = vec![];
        for pipe in 0..6 {
            spi_expectations.extend(spi_test_expects![
                // set payload length to 32 bytes on all pipes
                (
                    vec![(registers::RX_PW_P0 + pipe) | commands::W_REGISTER, 32],
                    vec![0xEu8, 0]
                ),
            ]);
        }
        spi_expectations.extend(spi_test_expects![
            // get payload length for all pipe 0 (because all pipes will use the same static length)
            (vec![registers::RX_PW_P0, 0], vec![0xEu8, 32]),
        ]);

        let mocks = mk_radio(&[], &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        radio.set_payload_length(76).unwrap();
        assert_eq!(radio.get_payload_length().unwrap(), 32);
        spi.done();
        ce_pin.done();
    }
}
