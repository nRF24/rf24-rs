use super::{commands, data_rate::set_tx_delay, registers, Feature, Nrf24Error, RF24};
use crate::{
    radio::{
        prelude::{EsbChannel, EsbFifo, EsbInit, EsbPayloadLength, EsbPipe, EsbPower, EsbStatus},
        RadioConfig,
    },
    StatusFlags,
};
use embedded_hal::{
    delay::DelayNs,
    digital::{Error, OutputPin},
    spi::SpiDevice,
};

impl<SPI, DO, DELAY> EsbInit for RF24<SPI, DO, DELAY>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    /// Initialize the radio's hardware using the [`SpiDevice`] and [`OutputPin`] given
    /// to [`RF24::new()`].
    fn init(&mut self) -> Result<(), Self::Error> {
        // Must allow the radio time to settle else configuration bits will not necessarily stick.
        // This is actually only required following power up but some settling time also appears to
        // be required after resets too. For full coverage, we'll always assume the worst.
        // Enabling 16b CRC is by far the most obvious case if the wrong timing is used - or skipped.
        // Technically we require 4.5ms + 14us as a worst case. We'll just call it 5ms for good measure.
        // WARNING: Delay is based on P-variant whereby non-P *may* require different timing.
        self.delay_impl.delay_ns(5000000);

        self.power_down()?;
        self.spi_read(1, registers::CONFIG)?;
        if self.buf[1] != self.config_reg.into_bits() {
            return Err(Nrf24Error::BinaryCorruption);
        }

        // detect if is a plus variant & use old toggle features command accordingly
        self.spi_read(1, registers::FEATURE)?;
        let before_toggle = self.buf[1];
        self.toggle_features()?;
        self.spi_read(1, registers::FEATURE)?;
        let after_toggle = self.buf[1];
        self.feature
            .set_is_plus_variant(before_toggle == after_toggle);
        if after_toggle < before_toggle {
            // FEATURE register is disabled on non-plus variants until `toggle_features()` is used.
            // MCU may have reset without triggering a power-on-reset in radio.
            self.toggle_features()?;
        }
        self.with_config(&RadioConfig::default())
    }

    fn with_config(&mut self, config: &RadioConfig) -> Result<(), Self::Error> {
        // Set CONFIG register:
        //      Set all IRQ events on IRQ pin
        //      Set CRC length
        //      Power up
        //      Enable PTX
        // Do not write CE high so radio will remain in standby-I mode.
        // PTX should use only 22uA of power in standby-I mode.
        self.config_reg = config.config_reg.with_power(true);
        self.ce_pin.set_low().map_err(|e| e.kind())?; // Guarantee CE is low on powerDown
        self.clear_status_flags(StatusFlags::new())?;

        // Flush buffers
        self.flush_rx()?;
        self.flush_tx()?;

        let addr_len = config.address_length();
        self.set_address_length(addr_len)?;

        self.spi_write_byte(registers::SETUP_RETR, config.auto_retries.into_bits())?;
        self.spi_write_byte(registers::EN_AA, config.auto_ack())?;
        self.feature = Feature::from_bits(
            self.feature.into_bits() & !Feature::REG_MASK
                | (config.feature.into_bits() & Feature::REG_MASK),
        );
        self.spi_write_byte(registers::DYNPD, 0x3F * (config.dynamic_payloads() as u8))?;
        self.spi_write_byte(
            registers::FEATURE,
            self.feature.into_bits() & Feature::REG_MASK,
        )?;

        let setup_rf_reg_val = config.setup_rf_aw.into_bits() & 0x27u8;
        self.spi_write_byte(registers::RF_SETUP, setup_rf_reg_val)?;
        self.tx_delay = set_tx_delay(config.data_rate());

        // setup RX addresses
        if config.is_rx_pipe_enabled(0) {
            self.pipe0_rx_addr = Some(config.pipes.pipe0);
        }
        self.spi_write_buf(registers::RX_ADDR_P0 + 1, &config.pipes.pipe1)?;
        for pipe in 2..6 {
            self.spi_write_byte(
                registers::RX_ADDR_P0 + pipe,
                config.pipes.subsequent_pipe_prefixes[pipe as usize - 2],
            )?;
        }

        // setup TX address
        config.tx_address(&mut self.tx_address);
        // use `spi_transfer()` to avoid multiple borrows of self (`spi_write_buf()` and `tx_address`)
        for reg in [registers::TX_ADDR, registers::RX_ADDR_P0] {
            self.buf[0] = reg | commands::W_REGISTER;
            self.buf[1..addr_len as usize + 1]
                .copy_from_slice(&self.tx_address[0..addr_len as usize]);
            self.spi_transfer(addr_len + 1)?;
        }

        // open all RX pipes; enable pipe 0 for TX mode
        self.spi_write_byte(registers::EN_RXADDR, config.pipes.rx_pipes_enabled | 1)?;

        self.set_payload_length(config.payload_length())?;

        self.set_channel(config.channel())?;

        self.spi_write_byte(registers::CONFIG, self.config_reg.into_bits())
    }
}

/////////////////////////////////////////////////////////////////////////////////
/// unit tests
#[cfg(test)]
mod test {
    extern crate std;
    use super::{registers, EsbInit};
    use crate::{
        radio::{rf24::commands, RadioConfig},
        spi_test_expects,
        test::mk_radio,
        DataRate, PaLevel,
    };
    use embedded_hal_mock::eh1::{
        digital::{State as PinState, Transaction as PinTransaction},
        spi::Transaction as SpiTransaction,
    };
    use std::vec;

    #[derive(Default)]
    struct InitParams {
        corrupted_binary: bool,
        is_plus_variant: bool,
        no_por: bool,
        is_p0_rx: bool,
    }
    fn init_parametrized(test_params: InitParams) {
        let mut ce_expectations = [PinTransaction::set(PinState::Low)].to_vec();
        let mut spi_expectations = vec![];
        if !test_params.is_p0_rx {
            // power_down()
            spi_expectations.extend(spi_test_expects![(
                vec![registers::CONFIG | commands::W_REGISTER, 0xC],
                vec![0xEu8, 0],
            ),]);
            if test_params.corrupted_binary {
                spi_expectations.extend(spi_test_expects![(
                    vec![registers::CONFIG, 0],
                    vec![0xFF, 0xFF]
                ),]);
                // !!! expectations stop here if emulating corrupted_binary
            } else {
                spi_expectations.extend(spi_test_expects![(
                    vec![registers::CONFIG, 0],
                    vec![0xEu8, 0xC]
                ),]);
                ce_expectations.extend([PinTransaction::set(PinState::Low)]);

                // check for plus_variant
                spi_expectations.extend(spi_test_expects![
                    // read FEATURE register
                    (
                        vec![registers::FEATURE, 0xC],
                        vec![0xEu8, if test_params.no_por { 5 } else { 0 }],
                    ),
                    // toggle_features()
                    (vec![commands::ACTIVATE, 0x73], vec![0xEu8, 0]),
                ]);
                if test_params.is_plus_variant {
                    // mocking a plus variant
                    spi_expectations.extend(spi_test_expects![
                        // read FEATURE register
                        (vec![registers::FEATURE, 0], vec![0xEu8, 0]),
                    ]);
                } else {
                    // mocking a non-plus variant
                    spi_expectations.extend(spi_test_expects![
                        // read FEATURE register
                        (
                            vec![registers::FEATURE, 0],
                            vec![0xEu8, if test_params.no_por { 0 } else { 5 }]
                        ),
                    ]);
                    if test_params.no_por {
                        spi_expectations.extend(spi_test_expects![
                            // toggle_features()
                            (vec![commands::ACTIVATE, 0x73], vec![0xEu8, 0]),
                        ]);
                    }
                }
            }
        }

        if !test_params.corrupted_binary {
            // begin with_config()
            spi_expectations.extend(spi_test_expects![
                // clear_status_flags()
                (
                    vec![registers::STATUS | commands::W_REGISTER, 0x70],
                    vec![0xEu8, 0],
                ),
                // flush_rx()
                (vec![commands::FLUSH_RX], vec![0xEu8]),
                // flush_tx()
                (vec![commands::FLUSH_TX], vec![0xEu8]),
                // set_address_length()
                (
                    vec![registers::SETUP_AW | commands::W_REGISTER, 3],
                    vec![0xEu8, 0],
                ),
                // set_auto_retries()
                (
                    vec![registers::SETUP_RETR | commands::W_REGISTER, 0x5F],
                    vec![0xEu8, 0],
                ),
                // write auto-ack register
                (
                    vec![registers::EN_AA | commands::W_REGISTER, 0x3F],
                    vec![0xEu8, 0],
                ),
                // write dynamic payloads register
                (
                    vec![registers::DYNPD | commands::W_REGISTER, 0],
                    vec![0xEu8, 0],
                ),
                // write FEATURE register
                (
                    vec![registers::FEATURE | commands::W_REGISTER, 0],
                    vec![0xEu8, 0],
                ),
                // write data rate && PA level register
                (
                    vec![
                        registers::RF_SETUP | commands::W_REGISTER,
                        DataRate::Mbps1.into_bits() | PaLevel::Max.into_bits() | 1
                    ],
                    vec![0xEu8, 0],
                ),
                // set RX address for pipe 1
                (
                    vec![
                        (registers::RX_ADDR_P0 + 1) | commands::W_REGISTER,
                        0xC2,
                        0xC2,
                        0xC2,
                        0xC2,
                        0xC2
                    ],
                    vec![0xEu8, 0, 0, 0, 0, 0],
                ),
            ]);
            for (pipe, addr) in [0xC3, 0xC4, 0xC5, 0xC6].iter().enumerate() {
                spi_expectations.extend(spi_test_expects![
                    // set RX address for pipe
                    (
                        vec![
                            (registers::RX_ADDR_P0 + 2 + pipe as u8) | commands::W_REGISTER,
                            *addr,
                        ],
                        vec![0xEu8, 0],
                    ),
                ]);
            }
            // set TX address for pipe 0 and as RX address for pipe 0
            for reg in [registers::TX_ADDR, registers::RX_ADDR_P0] {
                spi_expectations.extend(spi_test_expects![(
                    vec![reg | commands::W_REGISTER, 0xE7, 0xE7, 0xE7, 0xE7, 0xE7],
                    vec![0xEu8, 0, 0, 0, 0, 0],
                ),]);
            }
            // open RX pipe 1. Also pipe 0 for TX (regardless of auto-ack)
            spi_expectations.extend(spi_test_expects![(
                vec![registers::EN_RXADDR | commands::W_REGISTER, 3],
                vec![0xEu8, 0],
            ),]);
            // set payload length to 32 bytes on all pipes
            for pipe in 0..6 {
                spi_expectations.extend(spi_test_expects![(
                    vec![(registers::RX_PW_P0 + pipe) | commands::W_REGISTER, 32],
                    vec![0xEu8, 0],
                ),]);
            }
            spi_expectations.extend(spi_test_expects![
                // set_channel()
                (
                    vec![registers::RF_CH | commands::W_REGISTER, 76],
                    vec![0xEu8, 0],
                ),
                // write CONFIG register (configure CRC, power, and IRQ events)
                (
                    vec![registers::CONFIG | commands::W_REGISTER, 0xE],
                    vec![0xEu8, 0],
                ),
            ]);
        }

        let mocks = mk_radio(&ce_expectations, &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        let result = if !test_params.is_p0_rx {
            radio.init()
        } else {
            radio.with_config(&RadioConfig::default().with_rx_address(0, &[0xE7; 5]))
        };
        if test_params.corrupted_binary {
            assert!(result.is_err());
        } else {
            assert!(result.is_ok());
        }
        if !test_params.is_p0_rx {
            assert_eq!(radio.is_plus_variant(), test_params.is_plus_variant);
        }
        spi.done();
        ce_pin.done();
    }

    #[test]
    fn init_bin_corrupt() {
        init_parametrized(InitParams {
            corrupted_binary: true,
            is_plus_variant: true,
            ..Default::default()
        });
    }

    #[test]
    fn init_plus_variant() {
        init_parametrized(InitParams {
            is_plus_variant: true,
            ..Default::default()
        });
    }

    #[test]
    fn init_non_plus_variant() {
        init_parametrized(InitParams::default());
    }

    #[test]
    fn init_non_plus_variant_no_por() {
        init_parametrized(InitParams {
            no_por: true,
            ..Default::default()
        });
    }

    #[test]
    fn init_with_pipe0_rx() {
        init_parametrized(InitParams {
            is_p0_rx: true,
            ..Default::default()
        });
    }
}
