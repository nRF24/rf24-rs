use super::{data_rate::set_tx_delay, registers, Feature, Nrf24Error, RF24};
use crate::{
    radio::{
        prelude::{EsbChannel, EsbFifo, EsbInit, EsbPayloadLength, EsbPipe, EsbPower, EsbStatus},
        RadioConfig,
    },
    StatusFlags,
};
use embedded_hal::{delay::DelayNs, digital::OutputPin, spi::SpiDevice};

impl<SPI, DO, DELAY> EsbInit for RF24<SPI, DO, DELAY>
where
    SPI: SpiDevice,
    DO: OutputPin,
    DELAY: DelayNs,
{
    type ConfigErrorType = Nrf24Error<SPI::Error, DO::Error>;

    /// Initialize the radio's hardware using the [`SpiDevice`] and [`OutputPin`] given
    /// to [`RF24::new()`].
    fn init(&mut self) -> Result<(), Self::ConfigErrorType> {
        // Must allow the radio time to settle else configuration bits will not necessarily stick.
        // This is actually only required following power up but some settling time also appears to
        // be required after resets too. For full coverage, we'll always assume the worst.
        // Enabling 16b CRC is by far the most obvious case if the wrong timing is used - or skipped.
        // Technically we require 4.5ms + 14us as a worst case. We'll just call it 5ms for good measure.
        // WARNING: Delay is based on P-variant whereby non-P *may* require different timing.
        self._delay_impl.delay_ns(5000000);

        self.power_down()?;
        self.spi_read(1, registers::CONFIG)?;
        if self._buf[1] != self._config_reg.into_bits() {
            return Err(Nrf24Error::BinaryCorruption);
        }

        // detect if is a plus variant & use old toggle features command accordingly
        self.spi_read(1, registers::FEATURE)?;
        let before_toggle = self._buf[1];
        self.toggle_features()?;
        self.spi_read(1, registers::FEATURE)?;
        let after_toggle = self._buf[1];
        self._feature
            .set_is_plus_variant(before_toggle == after_toggle);
        if after_toggle < before_toggle {
            // FEATURE register is disabled on non-plus variants until `toggle_features()` is used.
            // MCU may have reset without triggering a power-on-reset in radio.
            self.toggle_features()?;
        }
        self.with_config(&RadioConfig::default())
    }

    fn with_config(&mut self, config: &RadioConfig) -> Result<(), Self::ConfigErrorType> {
        self.clear_status_flags(StatusFlags::new())?;
        self.power_down()?;

        // Flush buffers
        self.flush_rx()?;
        self.flush_tx()?;

        self.set_address_length(config.address_length())?;

        self.spi_write_byte(registers::SETUP_RETR, config.auto_retries.into_bits())?;
        self.spi_write_byte(registers::EN_AA, config.auto_ack())?;
        self._feature = Feature::from_bits(
            self._feature.into_bits() & !Feature::REG_MASK
                | (config.feature.into_bits() & Feature::REG_MASK),
        );
        self.spi_write_byte(registers::DYNPD, 0x3F * (config.dynamic_payloads() as u8))?;
        self.spi_write_byte(
            registers::FEATURE,
            self._feature.into_bits() & Feature::REG_MASK,
        )?;

        let setup_rf_reg_val = config.setup_rf_aw.into_bits() & 0x27u8;
        self.spi_write_byte(registers::RF_SETUP, setup_rf_reg_val)?;
        self.tx_delay = set_tx_delay(config.data_rate());

        let mut address = [0; 5];
        for pipe in 0..6 {
            config.rx_address(pipe, &mut address);
            self.open_rx_pipe(pipe, &address)?;
            // we need to set the pipe addresses before closing the pipe
            // because pipe 1 address is reused for pipes 2-5
            if !config.is_rx_pipe_enabled(pipe) {
                self.close_rx_pipe(pipe)?;
            }
        }
        config.tx_address(&mut address);
        self.open_tx_pipe(&address)?;

        self.set_payload_length(config.payload_length())?;

        self.set_channel(config.channel())?;

        // Set CONFIG register:
        //      Set all IRQ events on IRQ pin
        //      Set CRC length
        //      Power up
        //      Enable PTX
        // Do not write CE high so radio will remain in standby-I mode.
        // PTX should use only 22uA of power in standby-I mode.
        self._config_reg = config.config_reg.with_power(true);
        self.spi_write_byte(registers::CONFIG, self._config_reg.into_bits())
    }
}

/////////////////////////////////////////////////////////////////////////////////
/// unit tests
#[cfg(test)]
mod test {
    extern crate std;
    use super::{registers, EsbInit};
    use crate::{radio::rf24::commands, spi_test_expects, test::mk_radio, DataRate, PaLevel};
    use embedded_hal_mock::eh1::{
        digital::{State as PinState, Transaction as PinTransaction},
        spi::Transaction as SpiTransaction,
    };
    use std::vec;

    pub fn init_parametrized(corrupted_binary: bool, is_plus_variant: bool, no_por: bool) {
        let mut ce_expectations = [PinTransaction::set(PinState::Low)].to_vec();
        let mut spi_expectations = spi_test_expects![
            // power_down()
            (
                vec![registers::CONFIG | commands::W_REGISTER, 0xCu8],
                vec![0xEu8, 0u8],
            ),
        ]
        .to_vec();

        // read back CONFIG register to verify SPI lines are working
        if corrupted_binary {
            spi_expectations.extend(spi_test_expects![(
                vec![registers::CONFIG, 0u8],
                vec![0xFF, 0xFF]
            ),]);
            // !!! expectations stop here if emulating corrupted_binary
        } else {
            spi_expectations.extend(spi_test_expects![(
                vec![registers::CONFIG, 0u8],
                vec![0xEu8, 0xCu8]
            ),]);
            ce_expectations.extend([PinTransaction::set(PinState::Low)]);

            // check for plus_variant
            spi_expectations.extend(spi_test_expects![
                // read FEATURE register
                (
                    vec![registers::FEATURE, 0xCu8],
                    vec![0xEu8, if no_por { 5u8 } else { 0u8 }],
                ),
                // toggle_features()
                (vec![commands::ACTIVATE, 0x73u8], vec![0xEu8, 0u8]),
            ]);
            if is_plus_variant {
                // mocking a plus variant
                spi_expectations.extend(spi_test_expects![
                    // read FEATURE register
                    (vec![registers::FEATURE, 0u8], vec![0xEu8, 0u8]),
                ]);
            } else {
                // mocking a non-plus variant
                spi_expectations.extend(spi_test_expects![
                    // read FEATURE register
                    (
                        vec![registers::FEATURE, 0u8],
                        vec![0xEu8, if no_por { 0u8 } else { 5u8 }]
                    ),
                ]);
                if no_por {
                    spi_expectations.extend(spi_test_expects![
                        // toggle_features()
                        (vec![commands::ACTIVATE, 0x73u8], vec![0xEu8, 0u8]),
                    ]);
                }
            }

            spi_expectations.extend(spi_test_expects![
                // clear_status_flags()
                (
                    vec![registers::STATUS | commands::W_REGISTER, 0x70u8],
                    vec![0xEu8, 0u8],
                ),
                // power_down()
                (
                    vec![registers::CONFIG | commands::W_REGISTER, 0xCu8],
                    vec![0xEu8, 0u8],
                ),
                // flush_rx()
                (vec![commands::FLUSH_RX], vec![0xEu8]),
                // flush_tx()
                (vec![commands::FLUSH_TX], vec![0xEu8]),
                // set_address_length()
                (
                    vec![registers::SETUP_AW | commands::W_REGISTER, 3u8],
                    vec![0xEu8, 0u8],
                ),
                // set_auto_retries()
                (
                    vec![registers::SETUP_RETR | commands::W_REGISTER, 0x5fu8],
                    vec![0xEu8, 0u8],
                ),
                // write auto-ack register
                (
                    vec![registers::EN_AA | commands::W_REGISTER, 0x3Fu8],
                    vec![0xEu8, 0u8],
                ),
                // write dynamic payloads register
                (
                    vec![registers::DYNPD | commands::W_REGISTER, 0u8],
                    vec![0xEu8, 0u8],
                ),
                // write FEATURE register
                (
                    vec![registers::FEATURE | commands::W_REGISTER, 0u8],
                    vec![0xEu8, 0u8],
                ),
                // write data rate && PA level register
                (
                    vec![
                        registers::RF_SETUP | commands::W_REGISTER,
                        DataRate::Mbps1.into_bits() | PaLevel::Max.into_bits() | 1
                    ],
                    vec![0xEu8, 0u8],
                ),
            ]);
            for (pipe, addr) in [0xE7, 0xC2].iter().enumerate() {
                spi_expectations.extend(spi_test_expects![
                    // set RX address for pipe
                    (
                        vec![
                            (registers::RX_ADDR_P0 + pipe as u8) | commands::W_REGISTER,
                            *addr,
                            *addr,
                            *addr,
                            *addr,
                            *addr
                        ],
                        vec![0xEu8, 0, 0, 0, 0, 0],
                    ),
                    // enable RX pipe
                    (vec![registers::EN_RXADDR, 0], vec![0xEu8, 0]),
                    (
                        vec![registers::EN_RXADDR | commands::W_REGISTER, 1 << pipe],
                        vec![0xEu8, 0],
                    ),
                ]);
                if pipe == 0 {
                    // close pipe 0
                    spi_expectations.extend(spi_test_expects![
                        (vec![registers::EN_RXADDR, 0], vec![0xEu8, 1]),
                        (
                            vec![registers::EN_RXADDR | commands::W_REGISTER, 0],
                            vec![0xEu8, 0],
                        ),
                    ]);
                }
            }
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
                    // enable RX pipe
                    (vec![registers::EN_RXADDR, 0], vec![0xEu8, 0]),
                    (
                        vec![registers::EN_RXADDR | commands::W_REGISTER, 1 << (pipe + 2)],
                        vec![0xEu8, 0],
                    ),
                    // close RX pipe
                    (vec![registers::EN_RXADDR, 0], vec![0xEu8, 1 << (pipe + 2)]),
                    (
                        vec![registers::EN_RXADDR | commands::W_REGISTER, 0],
                        vec![0xEu8, 0],
                    ),
                ]);
            }
            spi_expectations.extend(spi_test_expects![
                // set TX address for pipe 0
                (
                    vec![
                        registers::TX_ADDR | commands::W_REGISTER,
                        0xE7,
                        0xE7,
                        0xE7,
                        0xE7,
                        0xE7
                    ],
                    vec![0xEu8, 0, 0, 0, 0, 0],
                ),
                // set RX address for pipe 0
                (
                    vec![
                        registers::RX_ADDR_P0 | commands::W_REGISTER,
                        0xE7,
                        0xE7,
                        0xE7,
                        0xE7,
                        0xE7
                    ],
                    vec![0xEu8, 0, 0, 0, 0, 0],
                ),
                // set payload length to 32 bytes on all pipes
                (
                    vec![registers::RX_PW_P0 | commands::W_REGISTER, 32u8],
                    vec![0xEu8, 0u8],
                ),
                (
                    vec![(registers::RX_PW_P0 + 1) | commands::W_REGISTER, 32u8],
                    vec![0xEu8, 0u8],
                ),
                (
                    vec![(registers::RX_PW_P0 + 2) | commands::W_REGISTER, 32u8],
                    vec![0xEu8, 0u8],
                ),
                (
                    vec![(registers::RX_PW_P0 + 3) | commands::W_REGISTER, 32u8],
                    vec![0xEu8, 0u8],
                ),
                (
                    vec![(registers::RX_PW_P0 + 4) | commands::W_REGISTER, 32u8],
                    vec![0xEu8, 0u8],
                ),
                (
                    vec![(registers::RX_PW_P0 + 5) | commands::W_REGISTER, 32u8],
                    vec![0xEu8, 0u8],
                ),
                // set_channel()
                (
                    vec![registers::RF_CH | commands::W_REGISTER, 76u8],
                    vec![0xEu8, 0u8],
                ),
                // write CONFIG register (configure CRC, power, and IRQ events)
                (
                    vec![registers::CONFIG | commands::W_REGISTER, 0xEu8],
                    vec![0xEu8, 0u8],
                ),
            ]);
        }

        let mocks = mk_radio(&ce_expectations, &spi_expectations);
        let (mut radio, mut spi, mut ce_pin) = (mocks.0, mocks.1, mocks.2);
        let result = radio.init();
        if corrupted_binary {
            assert!(result.is_err());
        } else {
            assert!(result.is_ok());
        }
        assert_eq!(radio.is_plus_variant(), is_plus_variant);
        spi.done();
        ce_pin.done();
    }

    #[test]
    fn init_bin_corrupt() {
        init_parametrized(true, true, false);
    }

    #[test]
    fn init_plus_variant() {
        init_parametrized(false, true, false);
    }

    #[test]
    fn init_non_plus_variant() {
        init_parametrized(false, false, false);
    }

    #[test]
    fn init_non_plus_variant_no_por() {
        init_parametrized(false, false, true);
    }
}
