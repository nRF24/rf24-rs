#![doc(
    html_logo_url = "https://raw.githubusercontent.com/nRF24/rf24-rs/main/docs/src/images/logo-square.png"
)]
#![doc(html_favicon_url = "https://github.com/nRF24/rf24-rs/raw/main/docs/src/images/favicon.ico")]
#![doc = include_str!("../README.md")]
#![no_std]

mod radio;
pub use radio::{ble_config, FakeBle, BLE_CHANNEL};

pub mod data_manipulation;

pub mod services;

#[cfg(test)]
mod test {
    use embedded_hal_mock::eh1::{
        delay::NoopDelay,
        digital::{Mock as PinMock, Transaction as PinTransaction},
        spi::{Mock as SpiMock, Transaction as SpiTransaction},
    };
    use rf24::radio::RF24;

    /// Takes an indefinite repetition of a tuple of 2 vectors: `(expected_data, response_data)`
    /// and generates an array of `SpiTransaction`s.
    ///
    /// NOTE: This macro is only used to generate code in unit tests (for this crate only).
    #[macro_export]
    macro_rules! spi_test_expects {
        ($( ($expected:expr , $response:expr $(,)? ) , ) + ) => {
            [
                $(
                    SpiTransaction::transaction_start(),
                    SpiTransaction::transfer_in_place($expected, $response),
                    SpiTransaction::transaction_end(),
                )*
            ]
        }
    }

    /// A tuple struct to encapsulate objects used to mock [`RF24`],
    pub struct MockRadio(
        pub RF24<SpiMock<u8>, PinMock, NoopDelay>,
        pub SpiMock<u8>,
        pub PinMock,
    );

    /// Create a mock objects using the given expectations.
    ///
    /// The `spi_expectations` parameter
    pub fn mk_radio(
        ce_expectations: &[PinTransaction],
        spi_expectations: &[SpiTransaction<u8>],
    ) -> MockRadio {
        let spi = SpiMock::new(spi_expectations);
        let ce_pin = PinMock::new(ce_expectations);
        let delay_impl = NoopDelay;
        let radio = RF24::new(ce_pin.clone(), spi.clone(), delay_impl);
        MockRadio(radio, spi, ce_pin)
    }
}
