use crate::radio::{prelude::EsbPayloadLength, Nrf24Error, RF24};
use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};

use super::{commands, registers, Feature};

impl<SPI, DO, DELAY> EsbPayloadLength for RF24<SPI, DO, DELAY>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    type PayloadLengthErrorType = Nrf24Error<SPI::Error, DO::Error>;

    fn set_payload_length(&mut self, length: u8) -> Result<(), Self::PayloadLengthErrorType> {
        let len = length.clamp(1, 32);
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
        self._feature =
            Feature::from_bits(self._feature.into_bits() & !Feature::REG_MASK | self._buf[1])
                .with_dynamic_payloads(enable);
        self.spi_write_byte(
            registers::FEATURE,
            self._feature.into_bits() & Feature::REG_MASK,
        )?;
        self.spi_write_byte(registers::DYNPD, 0x3F * enable as u8)?;
        Ok(())
    }

    fn get_dynamic_payloads(&self) -> bool {
        self._feature.dynamic_payloads()
    }

    fn get_dynamic_payload_length(&mut self) -> Result<u8, Self::PayloadLengthErrorType> {
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
    use crate::radio::prelude::{EsbAutoAck, EsbPayloadLength};
    use crate::radio::Nrf24Error;
    use crate::spi_test_expects;

    use super::{commands, registers, RF24};
    use embedded_hal_mock::eh1::delay::NoopDelay;
    use embedded_hal_mock::eh1::digital::Mock as PinMock;
    use embedded_hal_mock::eh1::spi::{Mock as SpiMock, Transaction as SpiTransaction};
    use std::vec;

    const EN_ACK_PAY: u8 = 1 << 1;
    const EN_DPL: u8 = 1 << 2;

    #[test]
    fn dynamic_payloads() {
        let pin_expectations = [];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

        let spi_expectations = spi_test_expects![
            // set_dynamic_payloads(true)
            (vec![registers::FEATURE, 0u8], vec![0xEu8, 0u8],),
            (
                vec![registers::FEATURE | commands::W_REGISTER, EN_DPL],
                vec![0xEu8, 0u8],
            ),
            (
                vec![registers::DYNPD | commands::W_REGISTER, 0x3Fu8],
                vec![0xEu8, 0],
            ),
            // read dynamic payload length invalid value
            (vec![commands::R_RX_PL_WID, 0u8], vec![0xEu8, 0xFFu8]),
            // read dynamic payload length valid value
            (vec![commands::R_RX_PL_WID, 0xFFu8], vec![0xEu8, 32u8]),
            // set_dynamic_payloads(false)
            (
                vec![registers::FEATURE, 32u8],
                vec![0xEu8, EN_ACK_PAY | EN_DPL],
            ),
            (
                vec![registers::FEATURE | commands::W_REGISTER, 0u8],
                vec![0xEu8, 0u8],
            ),
            (
                vec![registers::DYNPD | commands::W_REGISTER, 0u8],
                vec![0xEu8, 0],
            ),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        radio.set_dynamic_payloads(true).unwrap();
        assert!(radio.get_dynamic_payloads());
        assert_eq!(
            radio.get_dynamic_payload_length(),
            Err(Nrf24Error::BinaryCorruption)
        );
        assert_eq!(radio.get_dynamic_payload_length().unwrap(), 32u8);
        radio.set_dynamic_payloads(false).unwrap();
        assert!(!radio.get_dynamic_payloads());
        assert!(!radio.get_ack_payloads());
        spi_mock.done();
        pin_mock.done();
    }

    #[test]
    pub fn set_payload_length() {
        // Create pin
        let pin_expectations = [];
        let mut pin_mock = PinMock::new(&pin_expectations);

        // create delay fn
        let delay_mock = NoopDelay::new();

        let spi_expectations = spi_test_expects![
            // set payload length to 32 bytes on all pipes
            (
                vec![registers::RX_PW_P0 | commands::W_REGISTER, 32u8],
                vec![0xEu8, 0u8]
            ),
            (
                vec![(registers::RX_PW_P0 + 1) | commands::W_REGISTER, 32u8],
                vec![0xEu8, 0u8]
            ),
            (
                vec![(registers::RX_PW_P0 + 2) | commands::W_REGISTER, 32u8],
                vec![0xEu8, 0u8]
            ),
            (
                vec![(registers::RX_PW_P0 + 3) | commands::W_REGISTER, 32u8],
                vec![0xEu8, 0u8]
            ),
            (
                vec![(registers::RX_PW_P0 + 4) | commands::W_REGISTER, 32u8],
                vec![0xEu8, 0u8]
            ),
            (
                vec![(registers::RX_PW_P0 + 5) | commands::W_REGISTER, 32u8],
                vec![0xEu8, 0u8]
            ),
            // get payload length for all pipe 0 (because all pipes will use the same static length)
            (vec![registers::RX_PW_P0, 0u8], vec![0xEu8, 32u8]),
        ];
        let mut spi_mock = SpiMock::new(&spi_expectations);
        let mut radio = RF24::new(pin_mock.clone(), spi_mock.clone(), delay_mock);
        radio.set_payload_length(76).unwrap();
        assert_eq!(radio.get_payload_length().unwrap(), 32u8);
        spi_mock.done();
        pin_mock.done();
    }
}
